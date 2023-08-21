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

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv_start = in.atlas_coords.xy;
    let uv_end = in.atlas_coords.zw;
    var scale = uv_end - uv_start;
    let scaled_uv = scale * in.uv + uv_start;


    let sampled = textureSample(atlas_texture, samp, scaled_uv);
    return vec4<f32>(in.color.rgb, sampled.a);
    // return vec4<f32>(1.0, 0., 0., 1.);
}