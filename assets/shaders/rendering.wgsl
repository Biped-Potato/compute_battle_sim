struct Unit {
    position: vec2<f32>,
    velocity : vec2<f32>,
    hash_id : i32,
    start_index : i32,
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
}

@group(0) @binding(0)
var<storage, read_write> units: array<Unit>;

@group(0) @binding(1)
var texture: texture_storage_2d<rgba8unorm, read_write>;

@group(0) @binding(2)
var<uniform> uniform_data : UniformData;

const workgroup_s = 32;

@compute @workgroup_size(workgroup_s, workgroup_s, 1)
fn clear(@builtin(global_invocation_id) invocation_id: vec3<u32>,@builtin(num_workgroups) num_workgroups: vec3<u32>) {
    let location = vec2<i32>(i32(invocation_id.x), i32(invocation_id.y));
    textureStore(texture, location, vec4<f32>(0.0,0.0,0.0,0.0));
}


@compute @workgroup_size(workgroup_s, 1, 1)
fn render(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let original_pos = units[i32(invocation_id.x)].position;
    let position = (original_pos)/uniform_data.camera_zoom+uniform_data.dimensions/2.+ uniform_data.camera_position;
    let strength = 1.0;
    let seed = f32(units[i32(invocation_id.x)].hash_id);
    let r = fract(sin(seed*12.012489)*1.1234);
    let g = fract(cos(seed*1.193817)*3.08923971);
    let b = fract(sin(seed*3.942812)*1.9831205);
    var color : vec4<f32> = vec4f(r,g,b,strength);

    if(abs(original_pos.x) > f32(uniform_data.grid_size) * f32(uniform_data.grid_width) * 0.5 || abs(original_pos.y) > f32(uniform_data.grid_size) * f32(uniform_data.grid_height) * 0.5) {
        color = vec4f(1.0,0.0,0.0,1.0);
    }
    
    textureStore(texture, vec2<i32>(i32(position.x),i32(position.y)), color);
}