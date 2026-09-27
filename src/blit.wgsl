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

    let color = aces_tone_map(hdr,1);

    return vec4<f32>(color,1.0);

    //if depth > 0 {return vec4f(vec3f(1/(depth+1)),1);} else {return vec4f(vec3f(0),1);}

}

fn aces_tone_map(hdr: vec3<f32>, exposure: f32) -> vec3<f32> {
    let m1 = mat3x3(
        0.59719, 0.07600, 0.02840,
        0.35458, 0.90834, 0.13383,
        0.04823, 0.01566, 0.83777,
    );
    let m2 = mat3x3(
        1.60475, -0.10208, -0.00327,
        -0.53108,  1.10813, -0.07276,
        -0.07367, -0.00605,  1.07602,
    );
    let v = m1 * (hdr * exp2(exposure));
    let a = v * (v + 0.0245786) - 0.000090537;
    let b = v * (0.983729 * v + 0.4329510) + 0.238081;
    return clamp(m2 * (a / b), vec3(0.0), vec3(1.0));
}