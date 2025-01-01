use bevy::render::render_resource::{CachedComputePipelineId, CachedPipelineState, PipelineCache};

pub fn get_pipeline_states(
    pipelines: Vec<CachedComputePipelineId>,
    cache: &PipelineCache,
    shader_path: String,
) -> bool {
    for i in 0..pipelines.len() {
        match cache.get_compute_pipeline_state(pipelines[i]) {
            CachedPipelineState::Ok(_) => {}
            CachedPipelineState::Err(err) => {
                panic!("Initializing assets/{shader_path}:\n{err}")
            }
            _ => {
                return false;
            }
        }
    }
    return true;
}
