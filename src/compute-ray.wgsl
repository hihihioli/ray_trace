@group(0) @binding(0)
var output: texture_storage_2d<rgba16float, write>;

@compute @workgroup_size(8, 8)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let size = textureDimensions(output);

    if id.x >= size.x || id.y >= size.y {
        return;
    }

    let aspect = f32(size.x) / f32(size.y);
    let uv = vec2<f32>(id.xy) / vec2<f32>(size);

    // right now hardcode some camera values, later put them in a uniform buffer
    let focal_length = 1.0;
    let viewport_height = 2.0;
    let viewport_width = viewport_height * aspect;
    let camera_center = vec3f(0);

    let delta_u = viewport_width / f32(size.x);
    let delta_v = viewport_height / f32(size.y);

                                       //|   the start   |   | half pixel |  | shift for the id |
    let ray_direction = vec3f(-viewport_width/2 + 0.5 * delta_u + delta_u * f32(id.x),
                                          viewport_height/2 + 0.5 * delta_v - delta_v * f32(id.y),
                                         -focal_length); // Focal length
    // Right handed coord system, so +y:up +x:right +z: towards home

    let a = 0.5 * (normalize(ray_direction).y + 1.0);
    let color = (1-a) * vec3f(1.0, 1.0, 1.0) + a * vec3f(0.0, 0.0, 0.0);

    textureStore(output, id.xy, vec4f(color,1.0));
}
