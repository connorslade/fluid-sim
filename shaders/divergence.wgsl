@group(0) @binding(0) var<uniform> ctx: ComputeUniform;
@group(0) @binding(1) var<storage, read_write> state: array<Cell>;

@compute
@workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let pos = global_id.xy;
    if !in_bounds(pos) { return; }

    let current = ctx.tick;
    let next = ctx.tick + 1;

    // Set divergence to zero. Guarantees that energy is conserved.
    // ∇∙V = 0
    //
    // This is achieved by calculating the current divergence at each cell and
    // applying it as a correction to each neighboring cell. Because ticking all
    // the other cells will modify the divergence of previously processed cells,
    // this process must be iterated a few times to converge on a
    // divergence-free solution.
    let correction = 1.9 * divergence(pos) / f32(neighbors(pos));
    add_velocity_x(next, pos, atomicLoad(&state[index(current, pos)].vx));
    add_velocity_y(next, pos, atomicLoad(&state[index(current, pos)].vy));
    add_velocity_x(next, pos + vec2(1, 0), -correction);
    add_velocity_x(next, pos - vec2(1, 0), correction);
    add_velocity_y(next, pos + vec2(0, 1), -correction);
    add_velocity_y(next, pos - vec2(0, 1), correction);
    set_pressure(next, pos, atomicLoad(&state[index(current, pos)].p));
    set_velocity(ctx.tick + 2, pos, vec2f(0.0));
}

fn neighbors(pos: vec2u) -> u32 {
    return u32(pos.x > 0) + u32(pos.y > 0) + u32(pos.x + 1 < ctx.domain.x) + u32(pos.y + 1 < ctx.domain.y);
}
