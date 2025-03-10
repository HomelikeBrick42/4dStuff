@group(0) @binding(0)
var output_texture: texture_storage_2d<rgba32float, write>;

@compute @workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let size = textureDimensions(output_texture);
    let coords = global_id.xy;

    if coords.x >= size.x || coords.y >= size.y {
        return;
    }

    let uv = ((vec2<f32>(coords) + 0.5) / vec2<f32>(size)) * 2.0 - 1.0;

    let color = vec3<f32>(uv * 0.5 + 0.5, 0.0);
    textureStore(output_texture, coords, vec4<f32>(clamp(color, vec3<f32>(0.0), vec3<f32>(1.0)), 1.0));
}
