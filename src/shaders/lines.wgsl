struct Info {
    aspect: f32,
}

@group(0) @binding(0)
var<uniform> info: Info;

struct Line {
    a: vec2<f32>,
    b: vec2<f32>,
    width: f32,
    color: vec4<f32>,
}

struct Lines {
    length: u32,
    data: array<Line>,
}

@group(0) @binding(1)
var<storage, read> lines: Lines;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) instance_index: u32,
}

@vertex
fn vertex(@builtin(vertex_index) vertex_index: u32, @builtin(instance_index) instance_index: u32) -> VertexOutput {
    var out: VertexOutput;
    out.instance_index = instance_index;

    let x = f32((vertex_index >> 0) & 1u);
    let y = f32((vertex_index >> 1) & 1u) - 0.5;

    let line = lines.data[instance_index];
    let a_to_b = line.b - line.a;
    let normal = normalize(vec2<f32>(a_to_b.y, - a_to_b.x));

    var point = mix(line.a, line.b, x) + normal * y * line.width;
    point.x /= info.aspect;

    out.clip_position = vec4<f32>(point, 0.0, 1.0);
    return out;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let line = lines.data[in.instance_index];
    return line.color;
}
