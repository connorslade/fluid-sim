struct Uniform {
    window: vec2u,
    domain: vec2u,
    tick: u32,

    scale_factor: f32,
    pan: vec2f,
    zoom: f32,
    gain: f32
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

// todo: optimize this with texture lookup?
fn colormap(val: f32) -> vec3f {
    const colors: array<vec3f, 8> = array(
        vec3f(0.056399322167734, 0.056399092153948, 0.470000090878925),
        vec3f(0.222644860182128, 0.392446581996066, 0.751295557598444),
        vec3f(0.347983959048214, 0.715547021974385, 0.939863082545764),
        vec3f(0.681041753359947, 0.924929944364665, 0.919010549472813),
        vec3f(0.949673685165908, 0.854808289883082, 0.558650656382998),
        vec3f(0.920694559125121, 0.606274029060815, 0.306062848509552),
        vec3f(0.786802228396302, 0.337085773006795, 0.157695851171065),
        vec3f(0.590000114532225, 0.076696367700191, 0.119475059357670)
    );

    let a = mix(colors[0], colors[1], val);
    let b = mix(colors[1], colors[2], val);
    let c = mix(colors[2], colors[3], val);
    let d = mix(colors[3], colors[4], val);
    let e = mix(colors[4], colors[5], val);
    let f = mix(colors[5], colors[6], val);
    let g = mix(colors[6], colors[7], val);

    let h = mix(a, b, val);
    let i = mix(b, c, val);
    let j = mix(c, d, val);
    let k = mix(d, e, val);
    let l = mix(e, f, val);
    let m = mix(f, g, val);

    let n = mix(h, i, val);
    let o = mix(i, j, val);
    let p = mix(j, k, val);
    let q = mix(k, l, val);
    let r = mix(l, m, val);

    let s = mix(n, o, val);
    let t = mix(o, p, val);
    let u = mix(p, q, val);
    let v = mix(q, r, val);

    let w = mix(s, t, val);
    let x = mix(t, u, val);
    let y = mix(u, v, val);

    let z = mix(w, x, val);
    let α = mix(x, y, val);

    let β = mix(z, α, val);

    return β;
}
