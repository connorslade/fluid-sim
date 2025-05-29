struct Uniform {
    window: vec2u,
    domain: vec2u,
    tick: u32,

    scale_factor: f32,
    pan: vec2f,
    zoom: f32
}

struct VertexInput {
    @location(0) pos: vec4<f32>,
    @location(1) uv: vec2<f32>
}

struct VertexOutput {
    @builtin(position) pos: vec4<f32>,
    @location(1) uv: vec2<f32>,
}

fn index(tick: u32, pos: vec2u) -> u32 {
    return (tick % 3 * ctx.domain.x * ctx.domain.y) + pos.y * ctx.domain.x + pos.x;
}
