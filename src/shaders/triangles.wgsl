struct VertexInput {
    @location(0) position: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
}

struct Camera {
    transform: Transform,
    aspect: f32,
}

@group(0) @binding(0)
var<uniform> camera: Camera;

@vertex
fn vertex(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(in.position.zy, 0.0, in.position.x);
    out.clip_position.x /= camera.aspect;
    return out;
}

@fragment
fn pixel(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 0.0, 0.0, 1.0);
}

struct Transform {
    s: f32,
    e01: f32,
    e02: f32,
    e03: f32,
    e04: f32,
    e12: f32,
    e13: f32,
    e14: f32,
    e23: f32,
    e24: f32,
    e34: f32,
    e0123: f32,
    e0124: f32,
    e0134: f32,
    e0234: f32,
    e1234: f32,
}
