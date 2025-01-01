//! A compute shader that simulates Conway's Game of Life.
//!
//! Compute shaders use the GPU for computing arbitrary information, that may be independent of what
//! is rendered to the screen.

use bevy::{
    core_pipeline::oit::resolve::node, prelude::*, render::{
        extract_resource::{ExtractResource, ExtractResourcePlugin},
        render_asset::RenderAssetUsages,
        render_graph::{RenderGraph, RenderLabel},
        render_resource::*,
        renderer::RenderDevice,
        Render, RenderApp, RenderSet,
    }
};
use extra::fps_counter::FPSTextPlugin;
use logic::{LogicNode, LogicPipeline};
use rendering::{RenderNode, RenderingPipeline};

use rand::{thread_rng, Rng};

use sort::sort::{SortLabel, SortNode};
use unit::Unit;

pub mod extra;
pub mod logic;
pub mod rendering;
pub mod sort;
pub mod unit;

const DISPLAY_FACTOR: u32 = 1;
const SIZE: (u32, u32) = (1920 / DISPLAY_FACTOR, 1072 / DISPLAY_FACTOR);
const WORKGROUP_SIZE: u32 = 16;
const SIZE_X: u32 = 100;
const SIZE_Y: u32 = 100;
const COUNT : i32 = nearest_base(SIZE_X as i32*SIZE_Y as i32,2);
fn main() {
    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugins((
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        resolution: (
                            (SIZE.0 * DISPLAY_FACTOR) as f32,
                            (SIZE.1 * DISPLAY_FACTOR) as f32,
                        )
                            .into(),
                        mode: bevy::window::WindowMode::BorderlessFullscreen(
                            MonitorSelection::Primary,
                        ),
                        // uncomment for unthrottled FPS
                        // present_mode: bevy::window::PresentMode::AutoNoVsync,
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
            SimulationComputePlugin,
            FPSTextPlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, set_texture)
        .run();
}
const fn nearest_base(input: i32, base: i32) -> i32 {
    let num = 2_i32.pow(base as u32);
    if input > num {
        return nearest_base(input, base + 1);
    }
    return num;
}
fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let mut image = Image::new_fill(
        Extent3d {
            width: SIZE.0,
            height: SIZE.1,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[0, 0, 0, 255],
        TextureFormat::Rgba8Unorm,
        RenderAssetUsages::RENDER_WORLD,
    );
    image.texture_descriptor.usage =
        TextureUsages::COPY_DST | TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING;
    let image = images.add(image);

    commands.spawn((
        Sprite {
            image: image.clone(),
            custom_size: Some(Vec2::new(SIZE.0 as f32, SIZE.1 as f32)),
            ..default()
        },
        Transform::from_scale(Vec3::splat(DISPLAY_FACTOR as f32)),
    ));
    commands.spawn(Camera2d);

    let mut units = Vec::new();
    let mut rand = thread_rng();
    println!("{}",COUNT);
    for i in 0..COUNT {
        units.push(Unit {
            position: Vec2::new(
                rand.gen_range(-((SIZE.0 / 2) as f32)..((SIZE.0 / 2) as f32)),
                rand.gen_range(-((SIZE.1 / 2) as f32)..((SIZE.1 / 2) as f32)),
            ),
            velocity: Vec2::new(0., 0.),
        });
    }
    commands.insert_resource(SimulationUniforms {
        render_texture: image,
        units: units,
    });
}
#[derive(Resource, Default, Deref)]
pub struct UnitBuffer(Vec<Buffer>);
#[derive(Resource, Default, Deref)]
pub struct SimulationUniformBuffer(Vec<Buffer>);

#[derive(Clone, ShaderType)]
pub struct UniformData {
    pub dimensions: Vec2,
    pub unit_count: i32,
    //for bitonic sort
    pub level  : i32,
    pub step : i32,
}
fn create_buffers(
    simulation_uniforms: Res<SimulationUniforms>,
    render_device: Res<RenderDevice>,
    mut unit_buffer: ResMut<UnitBuffer>,
    mut uniform_buffer: ResMut<SimulationUniformBuffer>,
) {
    if unit_buffer.0.len() == 0 {
        let mut byte_buffer = Vec::new();
        let mut buffer = encase::StorageBuffer::new(&mut byte_buffer);
        buffer.write(&simulation_uniforms.units).unwrap();

        let storage = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: None,
            usage: BufferUsages::COPY_DST | BufferUsages::STORAGE | BufferUsages::COPY_SRC,
            contents: buffer.into_inner(),
        });
        unit_buffer.0.push(storage);

        let uniform_data = UniformData {
            dimensions: Vec2::new(SIZE.0 as f32, SIZE.1 as f32),
            unit_count: COUNT as i32,
            level : 1,
            step : 1,
        };

        let mut byte_buffer = Vec::new();
        let mut buffer = encase::StorageBuffer::new(&mut byte_buffer);
        buffer.write(&uniform_data).unwrap();

        let uniform = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: None,
            usage: BufferUsages::COPY_DST | BufferUsages::UNIFORM | BufferUsages::COPY_SRC,
            contents: buffer.into_inner(),
        });
        uniform_buffer.0.push(uniform);
    }
}
fn set_texture(images: Res<SimulationUniforms>, sprite: Single<&mut Sprite>) {
    //sprite.image = images.render_texture.clone_weak();
}

pub struct SimulationComputePlugin;

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
pub struct LogicLabel;

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
pub struct RenderingLabel;

impl Plugin for SimulationComputePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ExtractResourcePlugin::<SimulationUniforms>::default());
        let render_app = app.sub_app_mut(RenderApp);
        render_app.add_systems(
            Render,
            (
                create_buffers,
                logic::prepare_bind_group.after(create_buffers),
                rendering::prepare_bind_group.after(create_buffers),
            )
                .in_set(RenderSet::PrepareBindGroups),
        );
        render_app.init_resource::<UnitBuffer>();
        render_app.init_resource::<SimulationUniformBuffer>();

        let mut render_graph = render_app.world_mut().resource_mut::<RenderGraph>();

        render_graph.add_node(LogicLabel, LogicNode::default());
        render_graph.add_node(RenderingLabel, RenderNode::default());

        render_graph.add_node_edge(LogicLabel, RenderingLabel);
        render_graph.add_node_edge(RenderingLabel, bevy::render::graph::CameraDriverLabel);
        
        let num = COUNT.ilog(2) as i32;
        let mut node_id = 0;
        let mut sort_label = SortLabel(0);

        for pass in 1..=num {
            let level = 2_i32.pow(pass as u32);
            for pass_exp in (1..=pass).rev() {
                let step = 2_i32.pow(pass_exp as u32);
                sort_label = SortLabel(node_id);
                render_graph.add_node(
                    sort_label.clone(),
                    SortNode {
                        state : sort::sort::SortState::Loading,
                        level: level,
                        step: step,
                    },
                );
                if node_id == 0 {
                    render_graph.add_node_edge(sort_label.clone(), LogicLabel);
                }
                else{
                    render_graph.add_node_edge(sort_label.clone(),  SortLabel(node_id-1));
                }
                node_id+=1;
            }
        }
        

        
    }

    fn finish(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);
        render_app.init_resource::<LogicPipeline>();
        render_app.init_resource::<RenderingPipeline>();
    }
}

#[derive(Resource, Clone, ExtractResource)]
pub struct SimulationUniforms {
    render_texture: Handle<Image>,
    units: Vec<Unit>,
}
