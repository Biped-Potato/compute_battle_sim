use bevy::{prelude::*, render::render_resource::ShaderType};
#[derive(ShaderType, Default, Clone, Copy)]
pub struct Unit {
    pub previous_state : Vec2,
    pub current_state : Vec2,
    pub position: Vec2,
    pub velocity: Vec2,
    pub hash_id: i32,
}
