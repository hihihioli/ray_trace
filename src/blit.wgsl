@group(0) @binding(0)
var source: texture_2d<f32>;

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> @builtin(position) vec4<f32>{
     let positions = array<vec2<f32>, 3>(
            vec2<f32>(-1.0, -1.0),
            vec2<f32>( 3.0, -1.0),
            vec2<f32>(-1.0,  3.0),
     );

    return vec4f(positions[vertex_index], 0.0, 1.0);
}

@fragment
fn fs_main(@builtin(position) position: vec4f) -> @location(0) vec4<f32> {
    let pixel = vec2<i32>(position.xy);

    let texture_sample = textureLoad(source,pixel, 0);
    let hdr = texture_sample.rgb;
    let depth = texture_sample.a;

    let color = clamp(hdr,vec3f(0),vec3f(1));
    let color_corrected = vec3f(srgb_to_linear(color.r),srgb_to_linear(color.g),srgb_to_linear(color.b));

    return vec4<f32>(color,1.0);

}


fn srgb_to_linear(c: f32) -> f32 {
    if c <= 0.04045 {
        return c / 12.92;
    }

    return pow((c + 0.055) / 1.055, 2.4);
}