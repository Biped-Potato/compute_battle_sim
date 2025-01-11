struct Unit {
    position: vec2<f32>,
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
}


@group(0) @binding(0)
var<storage, read_write> units: array<Unit>;

@group(0) @binding(1)
var<storage, read_write> indices : array<u32>;

@group(0) @binding(2)
var<uniform> uniform_data : UniformData;

const targeting_factor : f32 = 0.01;

const avoid_factor : f32 = 0.05;

const visible_range : f32 = 10.0;
const protected_range : f32 = 4.0;

const max_speed = 0.5;

const workgroup_s = 32;


const offsets = array(
    vec2<i32>(-1, 1), vec2<i32>(0, 1), vec2<i32>(1, 1),
    vec2<i32>(-1, 0), vec2<i32>(0, 0), vec2<i32>(1, 0),
    vec2<i32>(-1,-1), vec2<i32>(0,-1), vec2<i32>(1,-1),
);

fn dimensionalize(offset : vec2<i32>) -> i32{
    return offset.x - offset.y*uniform_data.grid_width;
}
fn compute_hash_id(position : vec2<f32>) -> i32{
    let x = i32((position.x / f32(uniform_data.grid_size)) + (f32(uniform_data.grid_width)/2.0));
    let y = i32((position.y / f32(uniform_data.grid_size)) + (f32(uniform_data.grid_height)/2.0));
    return x + (y*uniform_data.grid_width);
}

@compute @workgroup_size(workgroup_s, 1, 1)
fn hash(@builtin(global_invocation_id) invocation_id: vec3<u32>){
    let index = i32(invocation_id.x); 
    units[index].hash_id = compute_hash_id(units[index].position);
}

@compute @workgroup_size(workgroup_s, 1, 1)
fn hash_indices(@builtin(global_invocation_id) invocation_id: vec3<u32>){
    var prev_key : i32 = 0;
    let index = u32(invocation_id.x);
    let key = units[index].hash_id;
    if (index == 0){
        prev_key = -1;
    }
    else {
        prev_key = units[index - 1].hash_id;
    }
    if (prev_key != key){
        indices[key] = index;
    }
}

@compute @workgroup_size(workgroup_s, 1, 1)
fn sort(@builtin(global_invocation_id) invocation_id: vec3<u32>){

    let idx_start = i32(invocation_id.x);

    let half_step = uniform_data.step/2;

    let low = (idx_start/half_step) * uniform_data.step + (idx_start % half_step);
            
    let direction = ((low/uniform_data.level) + 1)%2;

    compare(
        u32(low),
        u32(low + half_step),
        direction,
    );

}


fn compare(a: u32, b: u32, direction: i32) {
    var e : i32 = 0;
    if (units[a].hash_id > units[b].hash_id){
        e = 1;
    }
    if direction == e {
        let temp = units[a];
        units[a] = units[b];
        units[b] = temp;
    }
}
@compute @workgroup_size(workgroup_s, 1, 1)
fn update(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let index = i32(invocation_id.x); 
    var position : vec2<f32> = units[index].position;
    var velocity : vec2<f32> = units[index].velocity;
    let hash_id = units[index].hash_id;

    //targeting
    velocity += normalize(vec2<f32>(0.0,0.0)-position)*targeting_factor;
    //separation
    var close_dx : f32 = 0.0;
    var close_dy : f32 = 0.0;

    var neighboring_boids : f32 = 0.0;


    
    for(var j = 0;j<9;j++){
        let new_id = hash_id+dimensionalize(offsets[j]);

        let start_index = indices[new_id];
        if (new_id < 0 || new_id > uniform_data.unit_count) {
            continue;
        }
        for(var i = i32(start_index); i < uniform_data.unit_count; i++) {

            if(new_id != units[i].hash_id){
                break;
            }
            if (i != index){
                let o_position = units[i].position;
                let offset = position - o_position;
                let dist = length(offset);
                if (dist < protected_range){
                    let norm = normalize(offset);
                    let avoid = norm * avoid_factor * (protected_range/dist);
                    velocity += avoid;
                }
            }
        }
    }

    velocity = normalize(velocity) * clamp(length(velocity),-max_speed,max_speed);
    
    position += velocity;
    
    units[index].position = position;
    units[index].velocity = velocity;
    
}