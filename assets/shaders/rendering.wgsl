struct Unit {
    previous_state : vec2<f32>,
    current_state : vec2<f32>,
    velocity : vec2<f32>,
    hash_id : i32,
}

struct UniformData{
    dimensions : vec2<f32>,
    unit_count : i32,
    level : i32,
    step : i32,
    grid_size : i32,
    grid_width : i32,
    grid_height : i32,
    camera_zoom : f32,
    camera_position : vec2<f32>,
    alpha : f32,
}

@group(0) @binding(0)
var<storage, read_write> units: array<Unit>;

@group(0) @binding(1)
var texture: texture_storage_2d<rgba8unorm, read_write>;

@group(0) @binding(2)
var<uniform> uniform_data : UniformData;

const workgroup_s = 256;

@compute @workgroup_size(32, 32, 1)
fn clear(@builtin(global_invocation_id) invocation_id: vec3<u32>,@builtin(num_workgroups) num_workgroups: vec3<u32>) {
    let location = vec2<i32>(i32(invocation_id.x), i32(invocation_id.y));
    textureStore(texture, location, vec4<f32>(0.0,0.0,0.0,0.0));
}


@compute @workgroup_size(workgroup_s, 1, 1)
fn render(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let current_state = units[i32(invocation_id.x)].current_state;
    let previous_state = units[i32(invocation_id.x)].previous_state;
    let pos = current_state * uniform_data.alpha + previous_state * (1.0 - uniform_data.alpha);
    let screen_position = (pos+uniform_data.camera_position)/uniform_data.camera_zoom + uniform_data.dimensions/2.;
    var color : vec4<f32> = vec4f(1.0,1.0,1.0,1.0);
    
    textureStore(texture, vec2<i32>(i32(screen_position.x),i32(screen_position.y)), color);
}