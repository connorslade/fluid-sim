struct Uniform {
    window: vec2u,
    domain: vec2u,
    tick: u32,
    flags: u32,

    scale_factor: f32,
    pan: vec2f,
    zoom: f32,
    gain: f32
}

struct VertexInput {
    @location(0) pos: vec4<f32>,
    @location(1) uv: vec2<f32>
}

struct VertexOutput {
    @builtin(position) pos: vec4<f32>,
    @location(1) uv: vec2<f32>,
}

struct Cell {
    p: atomic<f32>,
    vx: atomic<f32>,
    vy: atomic<f32>
}

fn index(tick: u32, pos: vec2u) -> u32 {
    return (tick % 3 * ctx.domain.x * ctx.domain.y) + pos.y % ctx.domain.y * ctx.domain.x + pos.x % ctx.domain.x;
}

fn add_velocity_x(tick: u32, pos: vec2u, vel: f32) { atomicAdd(&state[index(tick, pos)].vx, vel); }
fn add_velocity_y(tick: u32, pos: vec2u, vel: f32) { atomicAdd(&state[index(tick, pos)].vy, vel); }

fn get_velocity(tick: u32, pos: vec2u) -> vec2f {
    return vec2f(
        atomicLoad(&state[index(tick, pos)].vx),
        atomicLoad(&state[index(tick, pos)].vy)
    );
}

fn get_velocity_bilinear(tick: u32, pos: vec2f) -> vec2f {
    let bottom_left = vec2u(pos);
    let delta = fract(pos);

    return get_velocity(tick, bottom_left) * (1 - delta.x) * (1 - delta.y)
        + get_velocity(tick, bottom_left + vec2(1, 0)) * delta.x * (1 - delta.y)
        + get_velocity(tick, bottom_left + vec2(0, 1)) * (1 - delta.x) * delta.y
        + get_velocity(tick, bottom_left + vec2(1, 1)) * delta.x * delta.y;
}

fn set_velocity(tick: u32, pos: vec2u, val: vec2f) {
    atomicStore(&state[index(tick, pos)].vx, val.x);
    atomicStore(&state[index(tick, pos)].vy, val.y);
}

fn add_velocity(tick: u32, pos: vec2u, val: vec2f) {
    atomicAdd(&state[index(tick, pos)].vx, val.x);
    atomicAdd(&state[index(tick, pos)].vy, val.y);
}
