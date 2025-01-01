use bevy::{prelude::*, render::render_resource::ShaderType};
#[derive(ShaderType, Default, Clone, Copy)]
pub struct Unit {
    pub position: Vec2,
    pub velocity: Vec2,
    pub hash_id: i32,
    pub start_index: i32,
}
