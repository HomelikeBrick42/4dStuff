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
    distance: f32,
    color: vec3<f32>,
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
        hit.hit = true;
        hit.distance = (h - sqrt(discriminant)) / a;
        hit.color = hyper_sphere.color;
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

    var color = ray.direction.xyz * 0.5 + 0.5;
    let hit = ray_hit(ray);
    if hit.hit {
        color = hit.color;
    }
    textureStore(output_texture, coords, vec4<f32>(clamp(color, vec3<f32>(0.0), vec3<f32>(1.0)), 1.0));
}
