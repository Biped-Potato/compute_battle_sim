struct Unit {
    position: vec2<f32>,
    velocity : vec2<f32>,
}

struct UniformData{
    dimensions : vec2<f32>,
    unit_count : i32
}

@group(0) @binding(0)
var<storage, read_write> units: array<Unit>;

@group(0) @binding(1)
var texture: texture_storage_2d<rgba8unorm, read_write>;

@group(0) @binding(2)
var<uniform> uniform_data : UniformData;

fn hash(value: u32) -> u32 {
    var state = value;
    state = state ^ 2747636419u;
    state = state * 2654435769u;
    state = state ^ state >> 16u;
    state = state * 2654435769u;
    state = state ^ state >> 16u;
    state = state * 2654435769u;
    return state;
}

fn randomFloat(value: u32) -> f32 {
    return f32(hash(value)) / 4294967295.0;
}

@compute @workgroup_size(16, 16, 1)
fn clear(@builtin(global_invocation_id) invocation_id: vec3<u32>,@builtin(num_workgroups) num_workgroups: vec3<u32>) {
    let location = vec2<i32>(i32(invocation_id.x), i32(invocation_id.y));
    textureStore(texture, location, vec4<f32>(0.0,0.0,0.0,0.0));
}


@compute @workgroup_size(16, 1, 1)
fn render(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let position = units[i32(invocation_id.x)].position+uniform_data.dimensions/2.;
    var color : vec4<f32> = vec4f(0.0,0.0,1.0,1.0);
    if(i32(invocation_id.x) > uniform_data.unit_count/2){
        color = vec4f(1.0,0.0,0.0,1.0);
    }
    

    textureStore(texture, vec2<i32>(i32(position.x),i32(position.y)), color);
}