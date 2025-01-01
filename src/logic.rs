use std::borrow::Cow;

use bevy::prelude::*;
use bevy::render::{
    render_graph::{self},
    render_resource::*,
    renderer::{RenderContext, RenderDevice},
};

use crate::{IndicesBuffer, SimulationUniformBuffer, UniformData, UnitBuffer, COUNT, GRID_SIZE, SIZE, WORKGROUP_SIZE};
const SHADER_ASSET_PATH: &str = "shaders/logic.wgsl";

pub enum LogicState {
    Loading,
    Update,
}

pub struct LogicNode {
    state: LogicState,
}

impl Default for LogicNode {
    fn default() -> Self {
        Self {
            state: LogicState::Loading,
        }
    }
}

#[derive(Resource)]
pub struct LogicBindGroup(pub BindGroup);

pub fn prepare_bind_group(
    mut commands: Commands,
    pipeline: Res<LogicPipeline>,
    unit_buffer: Res<UnitBuffer>,
    uniform_buffer: Res<SimulationUniformBuffer>,
    indices_buffer : Res<IndicesBuffer>,
    render_device: Res<RenderDevice>,
) {
    let bind_group = render_device.create_bind_group(
        None,
        &pipeline.texture_bind_group_layout,
        &[
            BindGroupEntry {
                binding: 0,
                resource: BindingResource::Buffer(unit_buffer.0[0].as_entire_buffer_binding()),
            },
            BindGroupEntry {
                binding: 1,
                resource: BindingResource::Buffer(indices_buffer.0[0].as_entire_buffer_binding()),
            },
            BindGroupEntry {
                binding : 2,
                resource : BindingResource::Buffer(uniform_buffer.0[0].as_entire_buffer_binding()),
            }
        ],
    );
    commands.insert_resource(LogicBindGroup(bind_group));
}

#[derive(Resource)]
pub struct LogicPipeline {
    pub texture_bind_group_layout: BindGroupLayout,
    pub sort_pipeline: CachedComputePipelineId,
    pub hash_pipeline: CachedComputePipelineId,
    pub hash_indices_pipeline: CachedComputePipelineId,
    pub update_pipeline: CachedComputePipelineId,
}

impl FromWorld for LogicPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let texture_bind_group_layout = render_device.create_bind_group_layout(
            "LogicUniforms",
            &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        );
        let shader = world.load_asset(SHADER_ASSET_PATH);
        let pipeline_cache = world.resource::<PipelineCache>();
        let update_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: None,
            layout: vec![texture_bind_group_layout.clone()],
            push_constant_ranges: Vec::new(),
            shader: shader.clone(),
            shader_defs: vec![],
            entry_point: Cow::from("update"),
            zero_initialize_workgroup_memory: false,
        });

        let sort_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: None,
            layout: vec![texture_bind_group_layout.clone()],
            push_constant_ranges: Vec::new(),
            shader: shader.clone(),
            shader_defs: vec![],
            entry_point: Cow::from("sort"),
            zero_initialize_workgroup_memory: false,
        });
        let hash_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: None,
            layout: vec![texture_bind_group_layout.clone()],
            push_constant_ranges: Vec::new(),
            shader: shader.clone(),
            shader_defs: vec![],
            entry_point: Cow::from("hash"),
            zero_initialize_workgroup_memory: false,
        });
        let hash_indices_pipeline =
            pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
                label: None,
                layout: vec![texture_bind_group_layout.clone()],
                push_constant_ranges: Vec::new(),
                shader,
                shader_defs: vec![],
                entry_point: Cow::from("hash_indices"),
                zero_initialize_workgroup_memory: false,
            });
        LogicPipeline {
            texture_bind_group_layout,
            sort_pipeline,
            hash_pipeline,
            hash_indices_pipeline,
            update_pipeline,
        }
    }
}

impl render_graph::Node for LogicNode {
    fn update(&mut self, world: &mut World) {
        let pipeline = world.resource::<LogicPipeline>();
        let pipeline_cache = world.resource::<PipelineCache>();

        // if the corresponding pipeline has loaded, transition to the next stage
        match self.state {
            LogicState::Loading => {
                match pipeline_cache.get_compute_pipeline_state(pipeline.update_pipeline) {
                    CachedPipelineState::Ok(_) => {
                        self.state = LogicState::Update;
                    }
                    CachedPipelineState::Err(err) => {
                        panic!("Initializing assets/{SHADER_ASSET_PATH}:\n{err}")
                    }
                    _ => {}
                }
            }
            LogicState::Update => {
                self.state = LogicState::Update;
            }
        }
    }

    fn run(
        &self,
        _graph: &mut render_graph::RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), render_graph::NodeRunError> {
        let bind_group = &world.resource::<LogicBindGroup>().0;
        let pipeline_cache = world.resource::<PipelineCache>();
        let pipeline = world.resource::<LogicPipeline>();
        let unit_buffer = world.resource::<UnitBuffer>();
        let indices_buffer = world.resource::<IndicesBuffer>();
        let render_device = world.resource::<RenderDevice>();

        // select the pipeline based on the current state
        match self.state {
            LogicState::Loading => {}
            LogicState::Update => {
                let mut pass_1 =
                    render_context
                        .command_encoder()
                        .begin_compute_pass(&ComputePassDescriptor {
                            label: Some(&"hash"),
                            ..Default::default()
                        });

                let hash_pipeline = pipeline_cache
                    .get_compute_pipeline(pipeline.hash_pipeline)
                    .unwrap();
                pass_1.set_bind_group(0, bind_group, &[]);
                pass_1.set_pipeline(hash_pipeline);

                pass_1.dispatch_workgroups((COUNT as u32) / WORKGROUP_SIZE, 1, 1);

                drop(pass_1);

                let num = COUNT.ilog(2) as i32;

                for sort_pass in 1..=num {
                    let level = 2_i32.pow(sort_pass as u32);
                    for pass_exp in (1..=sort_pass).rev() {
                        let step = 2_i32.pow(pass_exp as u32);

                        let uniform_data = UniformData {
                            dimensions: Vec2::new(SIZE.0 as f32, SIZE.1 as f32),
                            unit_count: COUNT as i32,
                            level: level,
                            step: step,
                            grid_size : GRID_SIZE,
                        };

                        let mut byte_buffer = Vec::new();
                        let mut buffer = encase::StorageBuffer::new(&mut byte_buffer);
                        buffer.write(&uniform_data).unwrap();

                        let uniform =
                            render_device.create_buffer_with_data(&BufferInitDescriptor {
                                label: None,
                                usage: BufferUsages::COPY_DST
                                    | BufferUsages::UNIFORM
                                    | BufferUsages::COPY_SRC,
                                contents: buffer.into_inner(),
                            });

                        let bind_group = render_device.create_bind_group(
                            None,
                            &pipeline.texture_bind_group_layout,
                            &[
                                BindGroupEntry {
                                    binding: 0,
                                    resource: BindingResource::Buffer(
                                        unit_buffer.0[0].as_entire_buffer_binding(),
                                    ),
                                },
                                BindGroupEntry {
                                    binding: 1,
                                    resource: BindingResource::Buffer(indices_buffer.0[0].as_entire_buffer_binding()),
                                },
                                BindGroupEntry {
                                    binding: 2,
                                    resource: BindingResource::Buffer(
                                        uniform.as_entire_buffer_binding(),
                                    ),
                                },
                            ],
                        );

                        let mut pass = render_context.command_encoder().begin_compute_pass(
                            &ComputePassDescriptor {
                                label: Some(
                                    ("level ".to_owned()
                                        + level.to_string().as_str()
                                        + " step "
                                        + step.to_string().as_str())
                                    .as_str(),
                                ),
                                ..Default::default()
                            },
                        );

                        let sort_pipeline = pipeline_cache
                            .get_compute_pipeline(pipeline.sort_pipeline)
                            .unwrap();

                        pass.set_bind_group(0, &bind_group, &[]);
                        pass.set_pipeline(sort_pipeline);

                        pass.dispatch_workgroups((COUNT as u32) / (2 * WORKGROUP_SIZE), 1, 1);
                    }
                }
                let mut pass_2 =
                    render_context
                        .command_encoder()
                        .begin_compute_pass(&ComputePassDescriptor {
                            label: Some(&"hash"),
                            ..Default::default()
                        });

                let hash_id_pipeline = pipeline_cache
                    .get_compute_pipeline(pipeline.hash_indices_pipeline)
                    .unwrap();
                pass_2.set_bind_group(0, bind_group, &[]);
                pass_2.set_pipeline(hash_id_pipeline);

                pass_2.dispatch_workgroups((COUNT as u32) / WORKGROUP_SIZE, 1, 1);


                drop(pass_2);

                let mut pass =
                    render_context
                        .command_encoder()
                        .begin_compute_pass(&ComputePassDescriptor {
                            label: Some(&"update"),
                            ..Default::default()
                        });

                let update_pipeline = pipeline_cache
                    .get_compute_pipeline(pipeline.update_pipeline)
                    .unwrap();
                pass.set_bind_group(0, bind_group, &[]);
                pass.set_pipeline(update_pipeline);

                pass.dispatch_workgroups((COUNT as u32) / WORKGROUP_SIZE, 1, 1);
            }
        }

        Ok(())
    }
}
