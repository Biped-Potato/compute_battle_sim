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
        Render, RenderApp, RenderSet,
    },
};
use logic::{LogicNode, LogicPipeline};
use rendering::{RenderNode, RenderingPipeline};
use unit::Unit;

pub mod logic;
pub mod rendering;
pub mod unit;

const DISPLAY_FACTOR: u32 = 1;
const SIZE: (u32, u32) = (1920 / DISPLAY_FACTOR, 1080 / DISPLAY_FACTOR);
const WORKGROUP_SIZE: u32 = 16;
const SIZE_X : u32 = 100;
const SIZE_Y : u32 = 100;
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
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, set_texture)
        .run();
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

    for x in 0..SIZE_X {
        for y in 0..SIZE_Y {
            units.push(Unit { position: Vec2::new(x as f32, y as f32)*5.0 });
        }
    }
    commands.insert_resource(SimulationUniforms {
        render_texture: image,
        units: units,
    });
}

fn set_texture(images: Res<SimulationUniforms>, mut sprite: Single<&mut Sprite>) {
    sprite.image = images.render_texture.clone_weak();
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
            (logic::prepare_bind_group,rendering::prepare_bind_group).in_set(RenderSet::PrepareBindGroups),
        );

        let mut render_graph = render_app.world_mut().resource_mut::<RenderGraph>();
        render_graph.add_node(LogicLabel, LogicNode::default());
        render_graph.add_node(RenderingLabel, RenderNode::default());

        render_graph.add_node_edge(RenderingLabel, LogicLabel);
        render_graph.add_node_edge(LogicLabel, bevy::render::graph::CameraDriverLabel);
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
