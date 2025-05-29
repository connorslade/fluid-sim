@group(0) @binding(0) var<uniform> ctx: Uniform;
@group(0) @binding(1) var<storage, read_write> state: array<f32>;

@compute
@workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    if state[index(ctx.tick, id.xy)] == 1.0 || state[index(ctx.tick, id.xy - vec2(1, 0))] == 1.0 || state[index(ctx.tick, id.xy - vec2(0, 1))] == 1.0 ||  state[index(ctx.tick, id.xy + vec2(1, 0))] == 1.0 || state[index(ctx.tick, id.xy + vec2(0, 1))] == 1.0 {
        state[index(ctx.tick + 1, id.xy)] = 1.0;
    }
}
