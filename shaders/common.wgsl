struct Uniform {
    window: vec2u,
    domain: vec2u,
    tick: u32,
    view: u32,

    scale_factor: f32,
    pan: vec2f,
    zoom: f32,
    gain: f32,
    dt: f32
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
    let wrapped_x = ((pos.x + ctx.domain.x) % ctx.domain.x);
    let wrapped_y = ((pos.y + ctx.domain.y) % ctx.domain.y);
    return (tick % 3 * ctx.domain.x * ctx.domain.y) + wrapped_y * ctx.domain.x + wrapped_x;
}

/// Approximates the divergence of the velocity vector field at pos.
///
/// div > 0, internal volume decreasing
/// div < 0, internal volume increasing
fn divergence(pos: vec2u) -> f32 {
    let dx = atomicLoad(&state[index(ctx.tick, pos + vec2(1, 0))].vx) - atomicLoad(&state[index(ctx.tick, pos - vec2(1, 0))].vx);
    let dy = atomicLoad(&state[index(ctx.tick, pos + vec2(0, 1))].vy) - atomicLoad(&state[index(ctx.tick, pos - vec2(0, 1))].vy);
    return (dx + dy) / 2.0;
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

fn get_pressure_bilinear(tick: u32, pos: vec2f) -> f32 {
    let bottom_left = vec2u(pos);
    let delta = fract(pos);

    return get_pressure(tick, bottom_left) * (1 - delta.x) * (1 - delta.y)
        + get_pressure(tick, bottom_left + vec2(1, 0)) * delta.x * (1 - delta.y)
        + get_pressure(tick, bottom_left + vec2(0, 1)) * (1 - delta.x) * delta.y
        + get_pressure(tick, bottom_left + vec2(1, 1)) * delta.x * delta.y;
}

fn get_pressure(tick: u32, pos: vec2u) -> f32 { return atomicLoad(&state[index(tick, pos)].p); }
fn set_pressure(tick: u32, pos: vec2u, val: f32) { atomicStore(&state[index(tick, pos)].p, val); }
