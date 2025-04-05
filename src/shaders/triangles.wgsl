struct VertexInput {
    @location(0) position: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
}

struct Camera {
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
