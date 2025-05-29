@group(0) @binding(0) var<uniform> ctx: Uniform;
@group(0) @binding(1) var<storage, read_write> state: array<f32>;

@vertex
fn vert(in: VertexInput) -> VertexOutput {
    let size = vec2f(ctx.window);
    let domain = vec2f(ctx.domain);

    let pos = in.pos.xy * domain / size * ctx.zoom + ctx.pan / size * ctx.scale_factor;
    return VertexOutput(vec4(pos, 0.0, 1.0), in.uv);
}

@fragment
fn frag(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4(1.0, 0.0, 0.0, 0.0);
}
