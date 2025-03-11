@group(0) @binding(0)
var output_texture: texture_storage_2d<rgba32float, write>;

struct Camera {
    position: vec4<f32>,
    forward: vec4<f32>,
    up: vec4<f32>,
    right: vec4<f32>,
    sun_direction: vec4<f32>,
    sun_color: vec3<f32>,
    sun_light_color: vec3<f32>,
    ambient_light_color: vec3<f32>,
    up_sky_color: vec3<f32>,
    down_sky_color: vec3<f32>,
}

@group(1) @binding(0)
var<uniform> camera: Camera;

struct Ray {
    origin: vec4<f32>,
    direction: vec4<f32>,
}

struct Hit {
    hit: bool,
    position: vec4<f32>,
    normal: vec4<f32>,
    color: vec3<f32>,
    distance: f32,
}

fn ray_hit(ray: Ray) -> Hit {
    var hit: Hit;
    hit.hit = false;
    return hit;
}

@compute @workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let size = textureDimensions(output_texture);
    let coords = global_id.xy;

    if coords.x >= size.x || coords.y >= size.y {
        return;
    }

    let aspect = f32(size.x) / f32(size.y);
    let uv = ((vec2<f32>(coords) + 0.5) / vec2<f32>(size)) * 2.0 - 1.0;

    var ray: Ray;
    ray.origin = camera.position;
    ray.direction = normalize(camera.right * (uv.x * aspect) + camera.up * uv.y + camera.forward);

    var color = mix(camera.down_sky_color, camera.up_sky_color, ray.direction.y * 0.5 + 0.5);

    let hit = ray_hit(ray);
    if hit.hit {
        color = hit.color * camera.ambient_light_color;
        var sun_ray: Ray;
        sun_ray.origin = hit.position;
        sun_ray.direction = camera.sun_direction;
        let sun_hit = ray_hit(sun_ray);
        if !sun_hit.hit {
            color += camera.sun_light_color * hit.color * max(dot(sun_ray.direction, hit.normal), 0.0);
        }
    }
    else if dot(camera.sun_direction, ray.direction) > 0.99 {
        color = camera.sun_color;
    }

    textureStore(output_texture, coords, vec4<f32>(clamp(color, vec3<f32>(0.0), vec3<f32>(1.0)), 1.0));
}
