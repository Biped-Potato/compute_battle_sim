struct Unit {
    previous_state : vec2<f32>,
    current_state : vec2<f32>,
    velocity : vec2<f32>,
    hash_id : i32,
    attack_id : i32,
    id : i32,
    health : i32,
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
var<storage, read_write> indices : array<i32>;

@group(0) @binding(2)
var<uniform> uniform_data : UniformData;

const targeting_factor : f32 = 0.5;

const avoid_factor : f32 = 2.0;

const protected_range : f32 = 4.0;

const max_speed = 0.5;

const workgroup_s = 256;

const war_zone : f32 = 5.0;

const attack_range : f32 = 20.0;

const kill_range : f32 = 4.0;

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
    if(units[index].id == -1) {
        units[index].hash_id = -100;
    } 
    else {
        units[index].hash_id = compute_hash_id(units[index].current_state);
    }
}
@compute @workgroup_size(workgroup_s, 1, 1)
fn hash_indices(@builtin(global_invocation_id) invocation_id: vec3<u32>){
    var prev_key : i32 = 0;
    let index = i32(invocation_id.x);
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

fn get_side(id : i32) -> i32{
    if (id >= uniform_data.unit_count/2){
        return 1;
    }
    return 0;
}

@compute @workgroup_size(workgroup_s, 1, 1)
fn update(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let index = i32(invocation_id.x); 
    if(units[index].health <= 0){
        units[index].id = -1;
        return;
    }
    var current_state : vec2<f32> = units[index].current_state;
    units[index].previous_state = current_state;
    var velocity : vec2<f32> = units[index].velocity;
    let hash_id = units[index].hash_id;
    let id = units[index].id;
    let side = get_side(id);
    var closest : f32 = 1000.0;
    let attack_id = units[index].attack_id;
    var new_attack_id : i32 = -1;
    var enemy_index : i32 = -1;

    for(var j = 0;j<9;j++){
        
        let new_hash_id = hash_id+dimensionalize(offsets[j]);

        let start_index = indices[new_hash_id];
        for(var i = i32(start_index); i < uniform_data.unit_count; i++) {
            
            if(new_hash_id != units[i].hash_id){
                break;
            }
            if (i != index){
                let e_position = units[i].current_state;
                let e_id = units[i].id;
                let e_side = get_side(units[i].id);

                let offset = current_state - e_position;
                let dist = length(offset);
                if (dist < protected_range){
                    let norm = normalize(offset);
                    let avoid = norm * avoid_factor * (protected_range/dist);
                    velocity += avoid;
                }
                if (attack_id == -1 && e_side != side) {
                    if (dist < closest && dist < attack_range) {
                        new_attack_id = e_id;
                        enemy_index = i;
                        closest = dist;
                    }
                }
                else if (attack_id != -1) {
                    if (e_id == attack_id) {
                        new_attack_id = attack_id;
                        enemy_index = i;
                    }
                }
            }
        }
    }

    if (new_attack_id != -1) {
        units[enemy_index].attack_id = id;
        velocity += normalize(units[enemy_index].current_state-current_state)*targeting_factor;
        if(length(units[enemy_index].current_state - current_state) < kill_range) {
            units[enemy_index].health -= 1;
        }
    }
    else if (abs(current_state.x) < war_zone || abs(current_state.x) > f32(uniform_data.grid_width * uniform_data.grid_size)* 0.45) {
        velocity += normalize(vec2<f32>(0.0,0.0)-current_state)*targeting_factor;
    }
    else if (side == 1) {
        velocity.x -= targeting_factor;
    }
    else if (side == 0) {
        velocity.x += targeting_factor;
    }

    velocity = normalize(velocity) * clamp(length(velocity),-max_speed,max_speed);
    
    current_state += velocity;

    units[index].attack_id = new_attack_id;
    units[index].current_state = current_state;
    units[index].velocity = velocity;
    
}