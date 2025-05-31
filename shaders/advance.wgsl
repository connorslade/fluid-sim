@group(0) @binding(0) var<uniform> ctx: Uniform;
@group(0) @binding(1) var<storage, read_write> state: array<Cell>;

@compute
@workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let pos = global_id.xy;

    let dt = 0.1;
    let vel = get_velocity(ctx.tick, pos);
    set_velocity(ctx.tick + 1, pos, get_velocity_bilinear(ctx.tick, vec2f(pos) - vel * dt));
    set_velocity(ctx.tick + 2, pos, vec2f(0.0));
}
