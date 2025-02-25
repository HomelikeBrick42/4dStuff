@group(0)
@binding(0)
var output_texture: texture_storage_2d<rgba8unorm, write>;

struct Camera {
    position: vec4<f32>,
    forward: vec4<f32>,
    right: vec4<f32>,
    up: vec4<f32>,
    sun_direction: vec4<f32>,
    sun_color: vec3<f32>,
    ambient_color: vec3<f32>,
    up_sky_color: vec3<f32>,
    down_sky_color: vec3<f32>,
    aspect: f32,
}

@group(1)
@binding(0)
var<uniform> camera: Camera;

struct HyperSphere {
    position: vec4<f32>,
    color: vec3<f32>,
    radius: f32,
}

struct HyperSpheres {
    length: u32,
    data: array<HyperSphere>,
}

@group(2)
@binding(0)
var<storage, read> hyper_spheres: HyperSpheres;

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

fn hyper_sphere_hit(ray: Ray, hyper_sphere: HyperSphere) -> Hit {
    var hit: Hit;
    hit.hit = false;

    let oc = hyper_sphere.position - ray.origin;
    let a = dot(ray.direction, ray.direction); // TODO: can this be replaced with 1?
    let h = dot(ray.direction, oc);
    let c = dot(oc, oc) - hyper_sphere.radius * hyper_sphere.radius;
    let discriminant = h * h - a * c;

    if discriminant >= 0.0 {
        hit.distance = (h - sqrt(discriminant)) / a;
        if hit.distance > 0.0 {
            hit.hit = true;
            hit.position = ray.origin + ray.direction * hit.distance;
            hit.normal = (hit.position - hyper_sphere.position) / hyper_sphere.radius;
            hit.color = hyper_sphere.color;
        }
    }

    return hit;
}

fn ray_hit(ray: Ray) -> Hit {
    var hit: Hit;
    hit.hit = false;

    for (var i = 0u; i < hyper_spheres.length; i += 1u) {
        let hyper_sphere_hit = hyper_sphere_hit(ray, hyper_spheres.data[i]);
        if hyper_sphere_hit.hit && (!hit.hit || hyper_sphere_hit.distance < hit.distance) {
            hit = hyper_sphere_hit;
        }
    }

    return hit;
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
    ray.origin = camera.position;
    ray.direction = normalize(camera.right * (uv.x * camera.aspect) + camera.up * uv.y + camera.forward);

    var color = mix(camera.down_sky_color, camera.up_sky_color, ray.direction.y * 0.5 + 0.5);

    let hit = ray_hit(ray);
    if hit.hit {
        color = hit.color * camera.ambient_color;
        var sun_ray: Ray;
        sun_ray.origin = hit.position;
        sun_ray.direction = camera.sun_direction;
        let sun_hit = ray_hit(sun_ray);
        if !sun_hit.hit {
            color += hit.color * max(dot(sun_ray.direction, hit.normal), 0.0);
        }
    } else if dot(camera.sun_direction, ray.direction) > 0.99 {
        color = camera.sun_color;
    }

    textureStore(output_texture, coords, vec4<f32>(clamp(color, vec3<f32>(0.0), vec3<f32>(1.0)), 1.0));
}
