@group(0) @binding(0) var<uniform> ctx: Uniform;
@group(0) @binding(1) var<storage, read_write> state: array<Cell>;

@compute
@workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let pos = global_id.xy;

    let current = ctx.tick;
    let next = ctx.tick + 1;

    // Set divergence to zero. Guarantees that no matter is created or destroyed.
    // ∇∙V = 0
    let correction = 1.9 * divergence(pos) / 4.0;
    add_velocity_x(next, pos, atomicLoad(&state[index(current, pos)].vx));
    add_velocity_y(next, pos, atomicLoad(&state[index(current, pos)].vy));
    add_velocity_x(next, pos + vec2(1, 0), -correction);
    add_velocity_x(next, pos - vec2(1, 0), correction);
    add_velocity_y(next, pos + vec2(0, 1), -correction);
    add_velocity_y(next, pos - vec2(0, 1), correction);
    set_velocity(ctx.tick + 2, pos, vec2f(0.0));
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
