//! A compute shader that simulates Conway's Game of Life.
//!
//! Compute shaders use the GPU for computing arbitrary information, that may be independent of what
//! is rendered to the screen.

use bevy::{
    prelude::*,
    render::{
        extract_resource::{ExtractResource, ExtractResourcePlugin},
        render_asset::RenderAssetUsages,
        render_graph::{RenderGraph, RenderLabel},
        render_resource::*,
        renderer::RenderDevice,
        Render, RenderApp, RenderSet,
    },
};
use extra::stats::StatsPlugin;
use helpers::camera_controls::CameraControlsPlugin;
use logic::{LogicNode, LogicPipeline};
use rendering::{RenderNode, RenderingPipeline};

use rand::{thread_rng, Rng};

use timestep::fixed_time::FixedTimestep;
use unit::Unit;

pub mod extra;
pub mod helpers;
pub mod logic;
pub mod rendering;
pub mod timestep;
pub mod unit;

const DISPLAY_FACTOR: u32 = 1;
const SIZE: (u32, u32) = (1920 / DISPLAY_FACTOR, 1088 / DISPLAY_FACTOR);
const WORKGROUP_SIZE: u32 = 256;
const SIZE_X: u32 = 1000000;
const SIZE_Y: u32 = 1;
const COUNT: i32 = nearest_base(SIZE_X as i32 * SIZE_Y as i32, 2);
const GRID_SIZE: i32 = 5;
const WORLD_SIZE: (i32, i32) = (1920 * 3, 1080 * 3);
const HASH_SIZE: (i32, i32) = (WORLD_SIZE.0 / GRID_SIZE, WORLD_SIZE.1 / GRID_SIZE);

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
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
            SimulationComputePlugin,
            StatsPlugin,
            CameraControlsPlugin,
        ))
        .add_systems(Update, exit_on_esc)
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
fn exit_on_esc(mut writer: EventWriter<AppExit>, input: Res<ButtonInput<KeyCode>>) {
    if input.pressed(KeyCode::Escape) {
        writer.send(AppExit::Success);
    }
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
    println!("{}", COUNT);
    for i in 0..COUNT {
        let mut position = Vec2::new(
            rand.gen_range(-(WORLD_SIZE.0 as f32 * 0.47)..(-20.0)),
            rand.gen_range(-((WORLD_SIZE.1 / 2) as f32)..((WORLD_SIZE.1 / 2) as f32)) * 0.47 * 2.0,
        );

        if i > COUNT / 2 {
            position = Vec2::new(
                rand.gen_range((20.0)..(WORLD_SIZE.0 as f32 * 0.47)),
                rand.gen_range(-((WORLD_SIZE.1 / 2) as f32)..((WORLD_SIZE.1 / 2) as f32)) * 0.47 * 2.0,
            );
        }
        units.push(Unit {
            hash_id: -1,
            attack_id: -1,
            previous_state: position,
            current_state: position,
            velocity: Vec2::ZERO,
            id: i,
            health : 4,
        });
    }
    let width = (WORLD_SIZE.0 as f32 / GRID_SIZE as f32) as i32;
    let height = (WORLD_SIZE.1 as f32 / GRID_SIZE as f32) as i32;
    let uniform_data = UniformData {
        dimensions: Vec2::new(SIZE.0 as f32, SIZE.1 as f32),
        unit_count: COUNT as i32,
        level: 1,
        step: 1,
        grid_size: GRID_SIZE,
        grid_width: width,
        grid_height: height,
        camera_zoom: 0.25,
        camera_position: Vec2::ZERO,
        alpha: 0.0,
    };

    commands.insert_resource(SimulationUniforms {
        render_texture: image,
        units: units,
        data: Some(uniform_data),
    });
}
#[derive(Resource, Default, Deref)]
pub struct UnitBuffer(Vec<Buffer>);
#[derive(Resource, Default, Deref)]
pub struct SimulationUniformBuffer(Vec<Buffer>);

#[derive(Resource, Default, Deref)]
pub struct IndicesBuffer(Vec<Buffer>);
#[derive(Clone, ShaderType)]
pub struct UniformData {
    pub dimensions: Vec2,
    pub unit_count: i32,
    //for bitonic sort
    pub level: i32,
    pub step: i32,
    pub grid_size: i32,
    pub grid_width: i32,
    pub grid_height: i32,
    pub camera_zoom: f32,
    pub camera_position: Vec2,
    pub alpha: f32,
}

fn create_buffers(
    render_device: Res<RenderDevice>,
    simulation_uniforms: ResMut<SimulationUniforms>,
    mut unit_buffer: ResMut<UnitBuffer>,
    mut uniform_buffer: ResMut<SimulationUniformBuffer>,
    mut indices_buffer: ResMut<IndicesBuffer>,
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

        let mut byte_buffer = Vec::new();
        let mut buffer = encase::StorageBuffer::new(&mut byte_buffer);
        buffer
            .write(&simulation_uniforms.data.clone().unwrap())
            .unwrap();

        let uniform = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: None,
            usage: BufferUsages::COPY_DST | BufferUsages::UNIFORM | BufferUsages::COPY_SRC,
            contents: buffer.into_inner(),
        });
        uniform_buffer.0.push(uniform);

        let mut byte_buffer = Vec::new();
        let mut buffer = encase::StorageBuffer::new(&mut byte_buffer);

        buffer
            .write(&vec![-1; (HASH_SIZE.0 * HASH_SIZE.1) as usize])
            .unwrap();

        let storage = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: None,
            usage: BufferUsages::COPY_DST | BufferUsages::STORAGE | BufferUsages::COPY_SRC,
            contents: buffer.into_inner(),
        });
        indices_buffer.0.push(storage);
    }
}
fn set_texture(_images: Res<SimulationUniforms>, _sprite: Single<&mut Sprite>) {
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
        render_app.init_resource::<IndicesBuffer>();
        render_app.init_resource::<FixedTimestep>();

        let mut render_graph = render_app.world_mut().resource_mut::<RenderGraph>();

        render_graph.add_node(LogicLabel, LogicNode::default());
        render_graph.add_node(RenderingLabel, RenderNode::default());

        render_graph.add_node_edge(LogicLabel, RenderingLabel);
        render_graph.add_node_edge(RenderingLabel, bevy::render::graph::CameraDriverLabel);
    }

    fn finish(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);
        render_app.init_resource::<LogicPipeline>();
        render_app.init_resource::<RenderingPipeline>();
    }
}

#[derive(Resource, Clone)]
pub struct SimulationUniforms {
    data: Option<UniformData>,
    render_texture: Handle<Image>,
    units: Vec<Unit>,
}

impl ExtractResource for SimulationUniforms {
    type Source = SimulationUniforms;

    fn extract_resource(uniforms: &Self::Source) -> Self {
        SimulationUniforms {
            data: uniforms.data.clone(),
            render_texture: uniforms.render_texture.clone(),
            units: uniforms.units.clone(),
        }
    }
}
