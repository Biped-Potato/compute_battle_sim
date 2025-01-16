use bevy::{prelude::*, render::render_resource::ShaderType};
#[derive(ShaderType, Default, Clone, Copy)]
pub struct Unit {
    pub previous_state: Vec2,
    pub current_state: Vec2,
    pub velocity: Vec2,
    pub hash_id: i32,
    pub attack_id: i32,
    pub id: i32,
    pub health: i32,
}
