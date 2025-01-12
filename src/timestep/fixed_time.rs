use bevy::prelude::*;

#[derive(Resource)]
pub struct FixedTimestep {
    pub accumulater: f32,
    pub timestep: f32,
    pub current_time: f32,
    pub alpha: f32,
    pub time: f32,
}

impl Default for FixedTimestep {
    fn default() -> Self {
        Self {
            accumulater: 0.0,
            timestep: 1.0 / 8.0,
            current_time: 0.0,
            time: 0.0,
            alpha: 0.0,
        }
    }
}
