struct Camera {
    transform: Transform,
    aspect: f32,
}

@group(0) @binding(0)
var<uniform> camera: Camera;

struct Tetrahedron {
    positions: array<vec4<f32>, 4>,
}

struct Tetrahedrons {
    count: u32,
    data: array<Tetrahedron>,
}

@group(1) @binding(0)
var<storage, read> tetrahedrons: Tetrahedrons;

struct Vertex {
    position: vec3<f32>,
}

@group(1) @binding(1)
var<storage, read_write> vertices: array<Vertex>;

@group(1) @binding(2)
var<storage, read_write> indices: array<u32>;

struct Indirect {
    index_count: atomic<u32>,
    instance_count: u32,
    first_index: u32,
    base_vertex: i32,
    first_instance: u32,
    // this is not used by the indirect draw, but for inserting vertices in the compute shader
    vertex_count: atomic<u32>,
}

@group(1) @binding(3)
var<storage, read_write> indirect: Indirect;

// try messing with workgroup size in the future
@compute @workgroup_size(1, 1, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let tetrahedron_index = global_id.x;
    if tetrahedron_index >= tetrahedrons.count {
        return;
    }

    var tetrahedron = tetrahedrons.data[tetrahedron_index];
    for (var i = 0u; i < 4; i += 1u) {
        tetrahedron.positions[i] = transform_point(camera.transform, tetrahedron.positions[i]);
    }

    var positions: array<vec3<f32>, 4>;
    var position_count = 0u;
    for (var i = 0u; i < 4; i += 1u) {
        for (var j = i + 1u; i < 4; i += 1u) {
            let a = tetrahedron.positions[i];
            let b = tetrahedron.positions[j];
            if sign(a.w) != sign(b.w) {
                let distance = abs(a.w) + abs(b.w);
                if a.w <= 0.0 {
                    positions[position_count] = mix(a.xyz, b.xyz, abs(a.w) / distance);
                }
                else {
                    positions[position_count] = mix(b.xyz, a.xyz, abs(b.w) / distance);
                }
                position_count += 1u;
            }
        }
    }

    let vertex_index = atomicAdd(&indirect.vertex_count, position_count);
    for (var i = 0u; i < position_count; i += 1u) {
        vertices[vertex_index + i].position = positions[i];
    }

    if position_count == 3 {
        let index_index = atomicAdd(&indirect.index_count, 3u);
        indices[index_index + 0u] = vertex_index + 0u;
        indices[index_index + 1u] = vertex_index + 1u;
        indices[index_index + 2u] = vertex_index + 2u;
    }
    else if position_count == 4 {
        let index_index = atomicAdd(&indirect.index_count, 6u);
        indices[index_index + 0u] = vertex_index + 0u;
        indices[index_index + 1u] = vertex_index + 1u;
        indices[index_index + 2u] = vertex_index + 2u;
        indices[index_index + 3u] = vertex_index + 0u;
        indices[index_index + 4u] = vertex_index + 2u;
        indices[index_index + 5u] = vertex_index + 3u;
    }
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

fn transform_point(transform: Transform, point: vec4<f32>) -> vec4<f32> {
    let a = transform.s;
    let b = transform.e01;
    let c = transform.e02;
    let d = transform.e03;
    let f = transform.e04;
    let g = transform.e12;
    let h = transform.e13;
    let i = transform.e14;
    let j = transform.e23;
    let k = transform.e24;
    let l = transform.e34;
    let m = transform.e0123;
    let n = transform.e0124;
    let o = transform.e0134;
    let p = transform.e0234;
    let q = transform.e1234;
    let p3 = point.x;
    let p2 = point.y;
    let p1 = point.z;
    let p0 = point.w;
    let ap2 = a * p2;
    let gp3 = g * p3;
    let jp1 = j * p1;
    let kp0 = k * p0;
    let ap3 = a * p3;
    let gp2 = g * p2;
    let hp1 = h * p1;
    let ip0 = i * p0;
    let ap1 = a * p1;
    let lp0 = l * p0;
    let hp3 = h * p3;
    let jp2 = j * p2;
    let ap0 = a * p0;
    let lp1 = l * p1;
    let ip3 = i * p3;
    let kp2 = k * p2;
    let s0 = c + jp1 - ap2 - gp3 - kp0;
    let s1 = ap3 + b + hp1 - gp2 - ip0;
    let s2 = ap1 + d + jp2 - lp0 - hp3;
    let s3 = f + kp2 - ap0 - lp1 - ip3;
    let result = vec4<f32>(p0 + 2.0 * (q * (m + g * p1 + h * p2 + j * p3 - q * p0) + k * s0 + i * s1 + l * s2 - a * f - n * g - o * h - p * j), p1 + 2.0 * (a * d + m * g + q * (n + i * p2 + k * p3 - q * p1 - g * p0) + l * s3 - o * i - p * k - j * s0 - h * s1), p2 + 2.0 * (m * h + n * i + q * (l * p3 + o - q * p2 - h * p0 - i * p1) + g * s1 - a * c - l * p - k * s3 - j * s2), p3 + 2.0 * (a * b + l * o + m * j + n * k + q * (p - l * p2 - q * p3 - j * p0 - k * p1) + i * s3 + h * s2 + g * s0),);
    return result.wzyx;
}
