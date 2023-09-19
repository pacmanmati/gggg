@group(0) @binding(0)
var<uniform> camera: Camera;
@group(0) @binding(1)
var atlas_texture: texture_2d<f32>;
@group(0) @binding(2)
var samp: sampler;

struct Camera {
    view_projection: mat4x4<f32>,
    position: vec3<f32>,
}

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) atlas_coords: vec4<f32>,
}

struct InstanceInput {
    @location(2) model_matrix_0: vec4<f32>,
    @location(3) model_matrix_1: vec4<f32>,
    @location(4) model_matrix_2: vec4<f32>,
    @location(5) model_matrix_3: vec4<f32>,
    @location(6) albedo: vec4<f32>,
    @location(7) atlas_coords: vec4<f32>,
}

@vertex
fn vertex(vertex: VertexInput, instance: InstanceInput) -> VertexOutput {
    let model_matrix = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );

    var out: VertexOutput;
    out.clip_position = camera.view_projection * model_matrix * vec4<f32>(vertex.position, 1.0);
    out.uv = vertex.uv;
    out.color = instance.albedo;
    out.atlas_coords = instance.atlas_coords;
    return out;
}

fn median(r: f32, g: f32, b: f32) -> f32 {
    return max(min(r, g), min(max(r, g), b));
}

// this function is needed in 3d only https://github.com/Chlumsky/msdfgen
// fn screen_px_range(uv_start: vec2<f32>, uv_end: vec2<f32>, scaled_uv: vec2<f32>) -> f32 {
//     let px_range = 1.0; 
//     let texture_size = vec2<f32>(textureDimensions(atlas_texture)) * (uv_end - uv_start);
//     let unit_range = vec2<f32>(px_range) / texture_size;
//     let screen_tex_size = vec2<f32>(1.0) / fwidth(scaled_uv);
//     return max(0.5 * dot(unit_range, screen_tex_size), 1.0);
// }

    @fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv_start = in.atlas_coords.xy;
    let uv_end = in.atlas_coords.zw;
    var scale = uv_end - uv_start;
    let scaled_uv = scale * in.uv + uv_start;


    let sampled = textureSample(atlas_texture, samp, scaled_uv).a;
    if sampled > 0.5 {
        return vec4<f32>(1.0, 1.0, 1.0, sampled) * vec4<f32>(in.color.rgb, 1.0);
    } else {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }
    // let buffer = 0.1;
    // let smoothing = 0.1;
    // let alpha = smoothstep(buffer - smoothing, buffer + smoothing, 0.5 - sampled.a);
    // return vec4<f32>(in.color.rgb, alpha);

    // vec3 msd = texture(msdf, texCoord).rgb;
    // float sd = median(msd.r, msd.g, msd.b);
    // float screenPxDistance = screenPxRange()*(sd - 0.5);
    // float opacity = clamp(screenPxDistance + 0.5, 0.0, 1.0);
    // color = mix(bgColor, fgColor, opacity);

    // let msd = textureSample(atlas_texture, samp, scaled_uv).rgb;
    // let sd = median(msd.r, msd.g, msd.b);
    // // if pixel range was set to 2 (radius?)
    // // and a 32x32 distance field was generated, but then scaled up to a 72x72 quad
    // let screen_px_distance = 1.0 * (sd - 0.5); // e.g. 72/32 * 2 = 4.5
    // let opacity = clamp(screen_px_distance + 0.5, 0.0, 1.0);
    // let color = mix(in.color.rgb, vec3<f32>(0.0, 0.0, 1.0), opacity);
    // return vec4<f32>(color, 1.0);
}