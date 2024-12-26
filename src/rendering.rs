use std::borrow::Cow;

use bevy::prelude::*;
use bevy::render::{
    render_asset::RenderAssets,
    render_graph::{self},
    render_resource::*,
    renderer::{RenderContext, RenderDevice},
    texture::GpuImage,
};

use crate::{SimulationUniforms, SIZE, WORKGROUP_SIZE};
const SHADER_ASSET_PATH: &str = "shaders/rendering.wgsl";

pub enum RenderState {
    Loading,
    Init,
    Update,
}

pub struct RenderNode {
    state: RenderState,
}

impl Default for RenderNode {
    fn default() -> Self {
        Self {
            state: RenderState::Loading,
        }
    }
}

#[derive(Resource)]
pub struct RenderBindGroup(BindGroup);

pub fn prepare_bind_group(
    mut commands: Commands,
    pipeline: Res<RenderingPipeline>,
    gpu_images: Res<RenderAssets<GpuImage>>,
    simulation_uniforms: Res<SimulationUniforms>,
    render_device: Res<RenderDevice>,
) {
    let render_texture = gpu_images.get(&simulation_uniforms.render_texture).unwrap();
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
        &[
            BindGroupEntry {
                binding: 0,
                resource: BindingResource::Buffer(storage.as_entire_buffer_binding()),
            },
            BindGroupEntry {
                binding: 1,
                resource: BindingResource::TextureView(&render_texture.texture_view),
            },
        ],
    );
    commands.insert_resource(RenderBindGroup(bind_group));
}

#[derive(Resource)]
pub struct RenderingPipeline {
    texture_bind_group_layout: BindGroupLayout,
    init_pipeline: CachedComputePipelineId,
    update_pipeline: CachedComputePipelineId,
}

impl FromWorld for RenderingPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let texture_bind_group_layout = render_device.create_bind_group_layout(
            "RenderUniforms",
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
                    ty: BindingType::StorageTexture {
                        access: StorageTextureAccess::ReadWrite,
                        format: TextureFormat::Rgba8Unorm,
                        view_dimension: TextureViewDimension::D2,
                    },
                    count: None,
                },
            ],
        );
        let shader = world.load_asset(SHADER_ASSET_PATH);
        let pipeline_cache = world.resource::<PipelineCache>();
        let init_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: None,
            layout: vec![texture_bind_group_layout.clone()],
            push_constant_ranges: Vec::new(),
            shader: shader.clone(),
            shader_defs: vec![],
            entry_point: Cow::from("init"),
            zero_initialize_workgroup_memory: false,
        });
        let update_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: None,
            layout: vec![texture_bind_group_layout.clone()],
            push_constant_ranges: Vec::new(),
            shader,
            shader_defs: vec![],
            entry_point: Cow::from("update"),
            zero_initialize_workgroup_memory: false,
        });

        RenderingPipeline {
            texture_bind_group_layout,
            init_pipeline,
            update_pipeline,
        }
    }
}

impl render_graph::Node for RenderNode {
    fn update(&mut self, world: &mut World) {
        let pipeline = world.resource::<RenderingPipeline>();
        let pipeline_cache = world.resource::<PipelineCache>();

        // if the corresponding pipeline has loaded, transition to the next stage
        match self.state {
            RenderState::Loading => {
                match pipeline_cache.get_compute_pipeline_state(pipeline.init_pipeline) {
                    CachedPipelineState::Ok(_) => {
                        self.state = RenderState::Init;
                    }
                    CachedPipelineState::Err(err) => {
                        panic!("Initializing assets/{SHADER_ASSET_PATH}:\n{err}")
                    }
                    _ => {}
                }
            }
            RenderState::Init => {
                if let CachedPipelineState::Ok(_) =
                    pipeline_cache.get_compute_pipeline_state(pipeline.update_pipeline)
                {
                    self.state = RenderState::Update;
                }
            }
            RenderState::Update => {
                self.state = RenderState::Update;
            }
        }
    }

    fn run(
        &self,
        _graph: &mut render_graph::RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), render_graph::NodeRunError> {
        let bind_group = &world.resource::<RenderBindGroup>().0;
        let pipeline_cache = world.resource::<PipelineCache>();
        let pipeline = world.resource::<RenderingPipeline>();

        let mut pass = render_context
            .command_encoder()
            .begin_compute_pass(&ComputePassDescriptor::default());

        // select the pipeline based on the current state
        match self.state {
            RenderState::Loading => {}
            RenderState::Init => {
                let init_pipeline = pipeline_cache
                    .get_compute_pipeline(pipeline.init_pipeline)
                    .unwrap();
                pass.set_bind_group(0, bind_group, &[]);
                pass.set_pipeline(init_pipeline);
                pass.dispatch_workgroups(SIZE.0 / WORKGROUP_SIZE, SIZE.1 / WORKGROUP_SIZE, 1);
            }
            RenderState::Update => {
                let update_pipeline = pipeline_cache
                    .get_compute_pipeline(pipeline.update_pipeline)
                    .unwrap();
                pass.set_bind_group(0, bind_group, &[]);
                pass.set_pipeline(update_pipeline);

                pass.dispatch_workgroups(SIZE.0 / WORKGROUP_SIZE, SIZE.1 / WORKGROUP_SIZE, 1);
            }
        }

        Ok(())
    }
}
