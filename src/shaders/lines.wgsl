struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) instance_index: u32,
}

@vertex
fn vertex(@builtin(vertex_index) vertex_index: u32, @builtin(instance_index) instance_index: u32) -> VertexOutput {
    var out: VertexOutput;
    out.instance_index = instance_index;

    let x = f32((vertex_index >> 0) & 1u);
    let y = f32((vertex_index >> 1) & 1u);
    let uv = vec2<f32>(x, y);
    out.clip_position = vec4<f32>(uv * 2.0 - 1.0, 0.0, 1.0);
    return out;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 0.0, 0.0, 0.25);
}
