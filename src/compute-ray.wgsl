@group(0) @binding(0)
var output: texture_storage_2d<rgba16float, write>;

@compute @workgroup_size(8, 8)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let size = textureDimensions(output);

    if id.x >= size.x || id.y >= size.y {
        return;
    }

    let uv = vec2<f32>(id.xy) / vec2<f32>(size);

    let color = vec4<f32>(srgb_to_linear(uv.x),srgb_to_linear(uv.y), 2.0, 1.0);

    textureStore(output, id.xy, color);
}

fn srgb_to_linear(c: f32) -> f32 {
    if c <= 0.04045 {
        return c / 12.92;
    }

    return pow((c + 0.055) / 1.055, 2.4);
}