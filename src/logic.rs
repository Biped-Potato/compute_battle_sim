use std::borrow::Cow;

use bevy::prelude::*;
use bevy::render::{
    render_graph::{self},
    render_resource::*,
    renderer::{RenderContext, RenderDevice},
};

use crate::{SimulationUniforms, SIZE_X, SIZE_Y, WORKGROUP_SIZE};
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
pub struct LogicBindGroup(BindGroup);

pub fn prepare_bind_group(
    mut commands: Commands,
    pipeline: Res<LogicPipeline>,
    simulation_uniforms: Res<SimulationUniforms>,
    render_device: Res<RenderDevice>,
) {
    let mut byte_buffer = Vec::new();
    let mut buffer = encase::StorageBuffer::new(&mut byte_buffer);
    buffer.write(&simulation_uniforms.units).unwrap();

    let storage = render_device.create_buffer_with_data(&BufferInitDescriptor {
        label: None,
        usage: BufferUsages::COPY_DST | BufferUsages::STORAGE | BufferUsages::COPY_SRC,
        contents: buffer.into_inner(),
    });

    let bind_group = render_device.create_bind_group(
        None,
        &pipeline.texture_bind_group_layout,
        &[BindGroupEntry {
            binding: 0,
            resource: BindingResource::Buffer(storage.as_entire_buffer_binding()),
        }],
    );
    commands.insert_resource(LogicBindGroup(bind_group));
}

#[derive(Resource)]
pub struct LogicPipeline {
    texture_bind_group_layout: BindGroupLayout,
    update_pipeline: CachedComputePipelineId,
}

impl FromWorld for LogicPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let texture_bind_group_layout = render_device.create_bind_group_layout(
            "LogicUniforms",
            &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        );
        let shader = world.load_asset(SHADER_ASSET_PATH);
        let pipeline_cache = world.resource::<PipelineCache>();
        let update_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: None,
            layout: vec![texture_bind_group_layout.clone()],
            push_constant_ranges: Vec::new(),
            shader,
            shader_defs: vec![],
            entry_point: Cow::from("update"),
            zero_initialize_workgroup_memory: false,
        });

        LogicPipeline {
            texture_bind_group_layout,
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

        let mut pass = render_context
            .command_encoder()
            .begin_compute_pass(&ComputePassDescriptor::default());

        // select the pipeline based on the current state
        match self.state {
            LogicState::Loading => {}
            LogicState::Update => {
                let update_pipeline = pipeline_cache
                    .get_compute_pipeline(pipeline.update_pipeline)
                    .unwrap();
                pass.set_bind_group(0, bind_group, &[]);
                pass.set_pipeline(update_pipeline);

                pass.dispatch_workgroups((SIZE_X*SIZE_Y)/ WORKGROUP_SIZE, 1, 1);
            }
        }

        Ok(())
    }
}
