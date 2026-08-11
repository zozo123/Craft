// Block pipeline — WGSL port of shaders/block_{vertex,fragment}.glsl (simplified fog).

struct Uniforms {
    matrix: mat4x4<f32>,
    camera: vec3<f32>,
    fog_distance: f32,
    daylight: f32,
    _pad: vec3<f32>,
}

@group(0) @binding(0) var<uniform> u: Uniforms;
@group(0) @binding(1) var atlas: texture_2d<f32>;
@group(0) @binding(2) var atlas_samp: sampler;

struct VsIn {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) ao: f32,
    @location(4) light: f32,
}

struct VsOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) ao: f32,
    @location(2) light: f32,
    @location(3) diffuse: f32,
    @location(4) fog: f32,
}

const LIGHT_DIR: vec3<f32> = vec3<f32>(-0.57735026, 0.57735026, -0.57735026);

@vertex
fn vs_main(v: VsIn) -> VsOut {
    var o: VsOut;
    o.clip = u.matrix * vec4<f32>(v.position, 1.0);
    o.uv = v.uv;
    o.ao = 0.3 + (1.0 - v.ao) * 0.7;
    o.light = v.light;
    o.diffuse = max(0.0, dot(v.normal, LIGHT_DIR));
    let dist = distance(u.camera, v.position);
    o.fog = pow(clamp(dist / u.fog_distance, 0.0, 1.0), 4.0);
    return o;
}

@fragment
fn fs_main(v: VsOut) -> @location(0) vec4<f32> {
    var color = textureSample(atlas, atlas_samp, v.uv).rgb;
    // Magenta key in the original atlas = discard.
    if (all(color == vec3<f32>(1.0, 0.0, 1.0))) {
        discard;
    }
    let cloud = all(color == vec3<f32>(1.0, 1.0, 1.0));
    var df = v.diffuse;
    var ao = v.ao;
    if (cloud) {
        df = 1.0 - v.diffuse * 0.2;
        ao = 1.0 - (1.0 - v.ao) * 0.2;
    }
    ao = min(1.0, ao + v.light);
    df = min(1.0, df + v.light);
    let value = min(1.0, u.daylight + v.light);
    let ambient = vec3<f32>(value * 0.3 + 0.2);
    let light_c = ambient + ambient * df;
    color = clamp(color * light_c * ao, vec3<f32>(0.0), vec3<f32>(1.0));
    let sky = vec3<f32>(0.55, 0.72, 0.95) * u.daylight;
    color = mix(color, sky, v.fog);
    return vec4<f32>(color, 1.0);
}
