use bevy::{input::mouse::{MouseScrollUnit, MouseWheel}, prelude::*};

use crate::SimulationUniforms;


const SCROLL_SPEED : f32 = 100.0;
const MOVE_SPEED : f32 = 10.0;
pub struct CameraControlsPlugin;
impl Plugin for CameraControlsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_camera);
    }
}
fn update_camera(
    time : Res<Time>,
    keys : Res<ButtonInput<KeyCode>>,
    mut evr_scroll: EventReader<MouseWheel>,
    mut uniform_data : ResMut<SimulationUniforms>,
){
    if uniform_data.data.is_some(){
        for ev in evr_scroll.read() {
            match ev.unit {
                MouseScrollUnit::Line => {
                    uniform_data.data.as_mut().unwrap().camera_zoom += SCROLL_SPEED * ev.y;
                }
                MouseScrollUnit::Pixel => {
                    uniform_data.data.as_mut().unwrap().camera_zoom += SCROLL_SPEED * ev.y;
                }
            }
        }
        if keys.pressed(KeyCode::KeyW) {
            uniform_data.data.as_mut().unwrap().camera_position.y += MOVE_SPEED * time.delta_secs();
        }
        if keys.pressed(KeyCode::KeyS) {
            uniform_data.data.as_mut().unwrap().camera_position.y -= MOVE_SPEED * time.delta_secs();
        }
        if keys.pressed(KeyCode::KeyD) {
            uniform_data.data.as_mut().unwrap().camera_position.x += MOVE_SPEED * time.delta_secs();
        }
        if keys.pressed(KeyCode::KeyA) {
            uniform_data.data.as_mut().unwrap().camera_position.x -= MOVE_SPEED * time.delta_secs();
        }
    }
}