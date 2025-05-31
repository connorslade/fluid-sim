@group(0) @binding(0) var<uniform> ctx: Uniform;
@group(0) @binding(1) var<storage, read_write> state: array<Cell>;

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

    if (ctx.flags & 1) != 0 {
        let vel = vec2(atomicLoad(&state[idx].vx), atomicLoad(&state[idx].vy));
        return vec4((vel / 2.0 + 0.5), 0.0, 1.0);
    } else {
        let val = saturate(ctx.gain * atomicLoad(&state[idx].p));
        return vec4(colormap(val), 0.0);
    }
}

// todo: no need for the bèzier curve through colorspace
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
