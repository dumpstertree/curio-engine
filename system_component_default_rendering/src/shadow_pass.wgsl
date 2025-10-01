// shadow_pass.wgsl

struct ShadowCamera {
    view_proj: mat4x4<f32>,
};
@group(0) @binding(0)
var<uniform> shadow_camera: ShadowCamera;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(5) model_matrix_0: vec4<f32>,
    @location(6) model_matrix_1: vec4<f32>,
    @location(7) model_matrix_2: vec4<f32>,
    @location(8) model_matrix_3: vec4<f32>,
};

struct VSOut {
    @builtin(position) clip_position: vec4<f32>,
};

@vertex
fn vs_main(input: VertexInput) -> VSOut {
    let model = mat4x4<f32>(
        input.model_matrix_0, // column 0
        input.model_matrix_1, // column 1
        input.model_matrix_2, // column 2
        input.model_matrix_3, // column 3
    );

    var out: VSOut;
    // Column-major multiplication: view_proj * model * position
    out.clip_position = shadow_camera.view_proj * model * vec4<f32>(input.position, 1.0);

    return out;
}
