@group(0) @binding(0) var<uniform> ctx: Uniform;
@group(0) @binding(1) var<storage, read_write> state: array<Cell>;

// - [x] Correct velocity field divergence
// - [ ] Tick velocity field?

@compute
@workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let pos = global_id.xy;

    // Set divergence to zero. Guarantees that no matter is created or destroyed.
    // ∇∙V = 0
    let correction = 1.9 * divergence(pos) / 4.0;
    add_velocity_x(pos, atomicLoad(&state[index(ctx.tick, pos)].vx));
    add_velocity_y(pos, atomicLoad(&state[index(ctx.tick, pos)].vy));
    add_velocity_x(pos + vec2(1, 0), -correction);
    add_velocity_x(pos - vec2(1, 0), correction);
    add_velocity_y(pos + vec2(0, 1), -correction);
    add_velocity_y(pos - vec2(0, 1), correction);
    set_velocity(ctx.tick + 2, pos, vec2f(0.0));
}

/// Approximates the gradient of the pressure field.
fn gradient(pos: vec2u) -> vec2f {
    let dx = atomicLoad(&state[index(ctx.tick, pos + vec2(1, 0))].p) - atomicLoad(&state[index(ctx.tick, pos - vec2(1, 0))].p);
    let dy = atomicLoad(&state[index(ctx.tick, pos + vec2(0, 1))].p) - atomicLoad(&state[index(ctx.tick, pos - vec2(0, 1))].p);
    return vec2(dx, dy) / 2.0;
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

/// Basic utilities ///

fn add_velocity_x(pos: vec2u, vel: f32) { atomicAdd(&state[index(ctx.tick + 1, pos)].vx, vel); }
fn add_velocity_y(pos: vec2u, vel: f32) { atomicAdd(&state[index(ctx.tick + 1, pos)].vy, vel); }
fn get_velocity(tick: u32, pos: vec2u) -> vec2f {
    return vec2f(
        atomicLoad(&state[index(tick, pos)].vx),
        atomicLoad(&state[index(tick, pos)].vy)
    );
}
fn set_velocity(tick: u32, pos: vec2u, val: vec2f) {
    atomicStore(&state[index(tick, pos)].vx, val.x);
    atomicStore(&state[index(tick, pos)].vy, val.y);
}
