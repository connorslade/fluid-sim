@group(0) @binding(0) var<uniform> ctx: Uniform;
@group(0) @binding(1) var<storage, read_write> state: array<f32>;

@vertex
fn vert(in: VertexInput) -> VertexOutput {
    let size = vec2f(ctx.window);
    let zoom = ctx.zoom * ctx.zoom;

    let pos = in.pos.xy * vec2f(ctx.domain) / size * zoom + ctx.pan / size * ctx.scale_factor;
    return VertexOutput(vec4(pos, 0.0, 1.0), in.uv);
}

@fragment
fn frag(in: VertexOutput) -> @location(0) vec4<f32> {
    let coord = vec2u(in.uv * vec2f(ctx.domain));
    let idx = index(ctx.tick, coord);

    let val = saturate((ctx.gain * state[idx] + 1.0) / 2.0);
    return vec4(colormap(val), 0.0);
}
