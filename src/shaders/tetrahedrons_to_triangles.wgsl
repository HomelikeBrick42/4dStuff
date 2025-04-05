struct Tetrahedron {
    a: vec4<f32>,
    b: vec4<f32>,
    c: vec4<f32>,
    d: vec4<f32>,
}

struct Tetrahedrons {
    count: u32,
    data: array<Tetrahedron>,
}

@group(0) @binding(0)
var<storage, read> tetrahedrons: Tetrahedrons;

struct Vertex {
    position: vec3<f32>,
}

@group(0) @binding(1)
var<storage, read_write> vertices: array<Vertex>;

@group(0) @binding(2)
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

@group(0) @binding(3)
var<storage, read_write> indirect: Indirect;

// try messing with workgroup size in the future
@compute @workgroup_size(1, 1, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let tetrahedron_index = global_id.x;
    if tetrahedron_index >= tetrahedrons.count {
        return;
    }
    let tetrahedron = tetrahedrons.data[tetrahedron_index];

    let vertex_index = atomicAdd(&indirect.vertex_count, 3u);
    vertices[vertex_index + 0u].position = tetrahedron.a.xyz;
    vertices[vertex_index + 1u].position = tetrahedron.b.xyz;
    vertices[vertex_index + 2u].position = tetrahedron.c.xyz;

    let index_index = atomicAdd(&indirect.index_count, 3u);
    indices[index_index + 0u] = vertex_index + 0u;
    indices[index_index + 1u] = vertex_index + 1u;
    indices[index_index + 2u] = vertex_index + 2u;
}
