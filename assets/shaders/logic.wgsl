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
}

@group(0) @binding(0)
var<storage, read_write> units: array<Unit>;

@group(0) @binding(1)
var<storage, read_write> indices : array<u32>;

@group(0) @binding(2)
var<uniform> uniform_data : UniformData;

const targeting_factor : f32 = 0.001;

const matching_factor : f32 = 0.05;
const avoid_factor : f32 = 0.03;
const centering_factor : f32 = 0.05;

const visible_range : f32 = 10.0;
const protected_range : f32 = 5.0;

@compute @workgroup_size(16, 1, 1)
fn hash(@builtin(global_invocation_id) invocation_id: vec3<u32>){
    let width = i32(uniform_data.dimensions.x/f32(uniform_data.grid_size));
    let height = i32(uniform_data.dimensions.y/f32(uniform_data.grid_size));

    let index = i32(invocation_id.x); 
    let position = units[index].position;
    let x = i32((position.x / f32(uniform_data.grid_size)) + (f32(width)/2.0));
    let y = i32((position.y / f32(uniform_data.grid_size)) + (f32(height)/2.0));
    units[index].hash_id =  x + (y*width);
}

@compute @workgroup_size(16, 1, 1)
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


@compute @workgroup_size(16, 1, 1)
fn sort(@builtin(global_invocation_id) invocation_id: vec3<u32>){

    let idx_start = i32(invocation_id.x)*16;

    let half_step = uniform_data.step/2;
    for(var i = idx_start;i<idx_start+16;i++){
        let low = (i/half_step) * uniform_data.step + (i % half_step);
                
        let direction = ((low/uniform_data.level) + 1)%2;

        compare(
            u32(low),
            u32(low + half_step),
            direction,
        );
    }
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
@compute @workgroup_size(16, 1, 1)
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
    //alignment
    var x_vel_avg : f32 = 0.0;
    var y_vel_avg : f32 = 0.0;

    var neighboring_boids : f32 = 0.0;

    //cohesion
    var x_pos_avg : f32 = 0.0;
    var y_pos_avg : f32 = 0.0;


    let start_index = indices[hash_id];

    for(var i = i32(start_index); i < uniform_data.unit_count; i++) {

        if(units[i].hash_id != units[index].hash_id){
            break;
        }
        if (i != index){
            let o_position = units[i].position;
            let o_velocity = units[i].velocity;
            let distance = position - o_position;
            
            if (length(distance) < protected_range){
                close_dx += distance.x;
                close_dy += distance.y;
            }
            if (length(distance) < visible_range) {
                x_vel_avg += o_velocity.x;
                y_vel_avg += o_velocity.y;

                x_pos_avg += position.x;
                y_pos_avg += position.y;
                neighboring_boids += 1.0;
            }
        }
    }
    //separation
    velocity.x += avoid_factor * close_dx;
    velocity.y += avoid_factor * close_dy;

    if (neighboring_boids != 0){
        x_pos_avg = x_pos_avg/neighboring_boids;
        y_pos_avg = y_pos_avg/neighboring_boids;
        x_vel_avg = x_vel_avg/neighboring_boids;
        y_vel_avg = y_vel_avg/neighboring_boids;
    }
    

    velocity.x += (x_vel_avg-velocity.x)*matching_factor;
    velocity.y += (y_vel_avg-velocity.y)*matching_factor;
    
    velocity.x += (x_pos_avg - position.x)*centering_factor;
    velocity.y += (y_pos_avg - position.y)*centering_factor;
    
    let max_speed = 1.0;

    velocity = normalize(velocity) * clamp(length(velocity),-max_speed,max_speed);
    
    position += velocity;

    //clamp in bounds
    if (position.x > uniform_data.dimensions.x/2.){
        position.x -= uniform_data.dimensions.x;
    }
    if (position.x < -uniform_data.dimensions.x/2.){
        position.x += uniform_data.dimensions.x;
    }
    if (position.y > uniform_data.dimensions.y/2.){
        position.y -= uniform_data.dimensions.y;
    }
    if (position.y < -uniform_data.dimensions.y/2.){
        position.y += uniform_data.dimensions.y;
    }
    units[index].position = position;
    units[index].velocity = velocity;
    
}