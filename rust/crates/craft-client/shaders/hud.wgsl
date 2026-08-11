// Screen-space crosshair (clip-space quads).

struct VsIn {
    @location(0) pos: vec2<f32>,
}

struct VsOut {
    @builtin(position) clip: vec4<f32>,
}

@vertex
fn vs_main(v: VsIn) -> VsOut {
    var o: VsOut;
    o.clip = vec4<f32>(v.pos, 0.0, 1.0);
    return o;
}

@fragment
fn fs_main(_v: VsOut) -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 1.0, 1.0, 0.85);
}
