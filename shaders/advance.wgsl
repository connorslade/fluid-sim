@group(0) @binding(0) var<uniform> ctx: ComputeUniform;
@group(0) @binding(1) var<storage, read_write> state: array<Cell>;

@compute
@workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let pos = global_id.xy;
    if !in_bounds(pos) { return; }

    let vel = get_velocity(ctx.tick, pos);
    let last = vec2f(pos) - vel * ctx.dt;

    // Advance the velocity and pressure fields.
    set_velocity(ctx.tick + 1, pos, get_velocity_bilinear(ctx.tick, last));
    set_pressure(ctx.tick + 1, pos, get_pressure_bilinear(ctx.tick, last));
    set_velocity(ctx.tick + 2, pos, vec2f(0.0));
}
