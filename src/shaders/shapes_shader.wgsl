@group(0) @binding(0)
var<uniform> camera: Camera;

struct Camera {
    view_projection: mat4x4<f32>,
    position: vec3<f32>,
}

struct VertexInput {
    @location(0) position: vec3<f32>,
}

struct InstanceInput {
    @location(1) model_matrix_0: vec4<f32>,
    @location(2) model_matrix_1: vec4<f32>,
    @location(3) model_matrix_2: vec4<f32>,
    @location(4) model_matrix_3: vec4<f32>,
    @location(5) albedo: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
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
    // out.clip_position = vec4<f32>(vertex.position, 1.0);
    // out.clip_position = camera.view_projection * vec4<f32>(vertex.position, 1.0);
    out.clip_position = camera.view_projection * model_matrix * vec4<f32>(vertex.position, 1.0);
    out.color = instance.albedo;
    return out;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}