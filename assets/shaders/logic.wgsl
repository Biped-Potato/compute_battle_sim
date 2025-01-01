struct Unit {
    position: vec2<f32>,
    velocity : vec2<f32>
}

struct UniformData{
    dimensions : vec2<f32>,
    unit_count : i32,
    level : i32,
    step : i32,
}

@group(0) @binding(0)
var<storage, read_write> units: array<Unit>;

@group(0) @binding(1)
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

const targeting_factor : f32 = 0.01;

const matching_factor : f32 = 0.05;
const avoid_factor : f32 = 0.02;
const centering_factor : f32 = 0.05;

const visible_range : f32 = 10.0;
const protected_range : f32 = 5.0;

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
    if (units[a].position.x > units[b].position.x){
        e = 1;
    }
    if direction == e {
        let temp = units[a];
        units[a] = units[b];
        units[b] = temp;
    }
}

fn bool_to_int(b: bool) -> i32 {
    if b {
        return 1;
    }
    return 0;
}
@compute @workgroup_size(16, 1, 1)
fn update(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let index = i32(invocation_id.x); 
    var position : vec2<f32> = units[index].position;
    var velocity : vec2<f32> = units[index].velocity;
    

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




    for(var i = 0; i < uniform_data.unit_count;i++) {
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
    //separation
    velocity.x += avoid_factor * close_dx;
    velocity.y += avoid_factor * close_dy;

    //alignment
    x_vel_avg = x_vel_avg/neighboring_boids;
    y_vel_avg = y_vel_avg/neighboring_boids;

    velocity.x += (x_vel_avg-velocity.x)*matching_factor;
    velocity.y += (y_vel_avg-velocity.y)*matching_factor;

    //cohesion
    x_pos_avg = x_pos_avg/neighboring_boids;
    y_pos_avg = y_pos_avg/neighboring_boids;

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
    units[index] = Unit(
        position,
        velocity
    ); 
}