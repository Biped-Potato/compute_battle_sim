struct Unit {
    position: vec2<f32>,
}

@group(0) @binding(0)
var<storage, read_write> units: array<Unit>;

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


@compute @workgroup_size(16, 1, 1)
fn update(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let index = i32(invocation_id.x); 
    units[index] = Unit(
        vec2<f32>(units[index].position.x + 0.1,units[index].position.y)
    ); 
}