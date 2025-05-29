@group(0) @binding(0) var<uniform> ctx: Uniform;
@group(0) @binding(1) var<storage, read_write> state: array<f32>;

@compute
@workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let pos = global_id.xy;

    let c = 0.1;
    let dx = state[index(ctx.tick, pos + vec2(1, 0))] - 2.0 * state[index(ctx.tick, pos)] + state[index(ctx.tick, pos - vec2(1, 0))];
    let dy = state[index(ctx.tick, pos + vec2(0, 1))] - 2.0 * state[index(ctx.tick, pos)] + state[index(ctx.tick, pos - vec2(0, 1))];

    let Δ =  c * (dx + dy);
    state[index(ctx.tick + 1, pos)] = (2.0 * state[index(ctx.tick, pos)] - state[index(ctx.tick + 2, pos)]) + Δ;
}
