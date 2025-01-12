use bevy::{
    input::mouse::{MouseScrollUnit, MouseWheel},
    prelude::*,
};

use crate::SimulationUniforms;

const SCROLL_SPEED: f32 = 0.1;
const MOVE_SPEED: f32 = 500.0;
pub struct CameraControlsPlugin;
impl Plugin for CameraControlsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_camera);
    }
}
fn update_camera(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut evr_scroll: EventReader<MouseWheel>,
    mut uniform_data: ResMut<SimulationUniforms>,
) {
    if uniform_data.data.is_some() {
        let data = uniform_data.data.as_mut().unwrap();
        for ev in evr_scroll.read() {
            match ev.unit {
                MouseScrollUnit::Line => {
                    data.camera_zoom -= SCROLL_SPEED * ev.y;
                }
                MouseScrollUnit::Pixel => {
                    data.camera_zoom -= SCROLL_SPEED * ev.y;
                }
            }
        }
        data.camera_zoom = f32::clamp(data.camera_zoom, 0.1, f32::MAX);
        if keys.pressed(KeyCode::KeyW) {
            data.camera_position.y +=
                MOVE_SPEED * time.delta_secs() * f32::clamp(data.camera_zoom, 1.0, f32::MAX);
        }
        if keys.pressed(KeyCode::KeyS) {
            data.camera_position.y -=
                MOVE_SPEED * time.delta_secs() * f32::clamp(data.camera_zoom, 1.0, f32::MAX);
        }
        if keys.pressed(KeyCode::KeyD) {
            data.camera_position.x -=
                MOVE_SPEED * time.delta_secs() * f32::clamp(data.camera_zoom, 1.0, f32::MAX);
        }
        if keys.pressed(KeyCode::KeyA) {
            data.camera_position.x +=
                MOVE_SPEED * time.delta_secs() * f32::clamp(data.camera_zoom, 1.0, f32::MAX);
        }
    }
}
