@group(0) @binding(0)
var<uniform> camera: Camera;
@group(1) @binding(0)
var texture: texture_2d<f32>;
@group(2) @binding(0)
var samp: sampler;
@group(3) @binding(0)
var<storage, read> lights: array<Light>;

struct Camera {
    view_projection: mat4x4<f32>,
    position: vec3<f32>,
}

struct Light {
    position: vec4<f32>,
    color: vec4<f32>,
}

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) normal: vec3<f32>,
}

struct InstanceInput {
    @location(3) model_matrix_0: vec4<f32>,
    @location(4) model_matrix_1: vec4<f32>,
    @location(5) model_matrix_2: vec4<f32>,
    @location(6) model_matrix_3: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) world_normal: vec3<f32>,
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

    let world_position: vec4<f32> = model_matrix * vec4<f32>(vertex.position, 1.0);
    out.world_position = world_position.xyz;
    out.clip_position = camera.view_projection * world_position;
    out.uv = vertex.uv;
    out.normal = vertex.normal;
    out.world_normal = normalize((model_matrix * vec4<f32>(vertex.normal, 0.0)).xyz);

    return out;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {

    let object_color = textureSample(texture, samp, in.uv);

    let ambient_strength = 0.2;
    let ambient_color = vec3<f32>(0.1, 0.1, 0.1) * ambient_strength;

    let view_dir = normalize(camera.position - in.world_position);

    // var final_color = ambient_color * object_color.xyz;
    var final_color = vec3<f32>(0.0);
    for (var i = 0u; i < arrayLength(&lights); i++) {
        let light = lights[i];
        let light_dir = normalize(light.position.xyz - in.world_position);
        let half_dir = normalize(view_dir + light_dir);
        // let reflect_dir = reflect(-light_dir, in.world_normal);

        let diffuse_strength = max(dot(in.world_normal, light_dir), 0.0);
        let diffuse_color = light.color.xyz * diffuse_strength;

        // let specular_strength = pow(max(dot(view_dir, reflect_dir), 0.0), 32.0);
        let specular_strength = pow(max(dot(in.world_normal, half_dir), 0.0), 32.0);
        let specular_color = specular_strength * light.color.xyz;


        final_color += (diffuse_color + specular_color) * object_color.xyz;
        // final_color += (diffuse_color) * object_color.xyz;
        // // final_color += specular_color;
    }

    return vec4<f32>(final_color, object_color.a);
}