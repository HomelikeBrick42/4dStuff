@group(0)
@binding(0)
var output_texture: texture_storage_2d<rgba8unorm, write>;

struct Camera {
    position: vec4<f32>,
    forward: vec4<f32>,
    right: vec4<f32>,
    up: vec4<f32>,
    aspect: f32,
}

@group(1)
@binding(0)
var<uniform> camera: Camera;

struct Ray {
    origin: vec4<f32>,
    direction: vec4<f32>,
}

@compute
@workgroup_size(16, 16, 1)
fn main(
    @builtin(global_invocation_id) global_id: vec3<u32>,
) {
    let size = textureDimensions(output_texture);
    let coords = global_id.xy;

    if coords.x >= size.x || coords.y >= size.y {
        return;
    }

    var uv = ((vec2<f32>(coords) + 0.5) / vec2<f32>(size)) * 2.0 - 1.0;

    var ray: Ray;
    ray.origin = vec4<f32>(0.0, 0.0, 0.0, 0.0);
    ray.direction = normalize(camera.right * (uv.x * camera.aspect) + camera.up * uv.y + camera.forward);

    let color = ray.direction.xyz * 0.5 + 0.5;
    textureStore(output_texture, coords, vec4<f32>(clamp(color, vec3<f32>(0.0), vec3<f32>(1.0)), 1.0));
}
