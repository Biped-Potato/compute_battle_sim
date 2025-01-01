use bevy::{prelude::*, render::{render_graph::{self, RenderLabel}, render_resource::{encase, BindGroupEntry, BindingResource, BufferInitDescriptor, BufferUsages, CachedPipelineState, ComputePassDescriptor, PipelineCache}, renderer::{RenderContext, RenderDevice}}};

use crate::{logic::{LogicBindGroup, LogicPipeline}, UniformData, UnitBuffer, COUNT};

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
pub struct SortLabel(pub i32);

const SHADER_ASSET_PATH: &str = "shaders/logic.wgsl";

const WORKGROUP_SIZE : u32 = 16;
pub enum SortState{
    Loading,
    Update,
}

pub struct SortNode {
    pub state : SortState,
    pub level: i32,
    pub step: i32,
}

impl render_graph::Node for SortNode {
    fn update(&mut self, world: &mut World) {
        let pipeline = world.resource::<LogicPipeline>();
        let pipeline_cache = world.resource::<PipelineCache>();

        // if the corresponding pipeline has loaded, transition to the next stage
        match self.state {
            SortState::Loading => {
                match pipeline_cache.get_compute_pipeline_state(pipeline.sort_pipeline) {
                    CachedPipelineState::Ok(_) => {
                        self.state = SortState::Update;
                    }
                    CachedPipelineState::Err(err) => {
                        panic!("Initializing assets/{SHADER_ASSET_PATH}:\n{err}")
                    }
                    _ => {}
                }
            }
            SortState::Update => {
                self.state = SortState::Update;
            }
        }
    }

    fn run(
        &self,
        _graph: &mut render_graph::RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), render_graph::NodeRunError> {
        let pipeline_cache = world.resource::<PipelineCache>();
        let pipeline = world.resource::<LogicPipeline>();
        let unit_buffer = world.resource::<UnitBuffer>();

        let render_device = world.resource::<RenderDevice>();

        let mut pass = render_context
            .command_encoder()
            .begin_compute_pass(&ComputePassDescriptor::default());

        let uniform_data = UniformData {
            dimensions: Vec2::new(0.,0.),
            unit_count: COUNT as i32,
            level : self.level,
            step : self.step,
        };

        let mut byte_buffer = Vec::new();
        let mut buffer = encase::StorageBuffer::new(&mut byte_buffer);
        buffer.write(&uniform_data).unwrap();

        let uniform = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: None,
            usage: BufferUsages::COPY_DST | BufferUsages::UNIFORM | BufferUsages::COPY_SRC,
            contents: buffer.into_inner(),
        });

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
                    resource: BindingResource::Buffer(uniform.as_entire_buffer_binding()),
                },
            ],
        );

        // select the pipeline based on the current state
        match self.state {
            SortState::Loading => {}
            SortState::Update => {
                

                let sort_pipeline = pipeline_cache
                    .get_compute_pipeline(pipeline.sort_pipeline)
                    .unwrap();

                pass.set_bind_group(0, &bind_group, &[]);
                pass.set_pipeline(sort_pipeline);

                pass.dispatch_workgroups((COUNT as u32)/(2*WORKGROUP_SIZE), 1, 1);
            }
        }

        Ok(())
    }
}
