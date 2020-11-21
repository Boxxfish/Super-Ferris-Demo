# version 450

layout(location=0) in vec3 a_position;
layout(location=1) in vec2 a_tex_coords;

layout(location=0) out vec2 v_tex_coords;

layout(set=1, binding=0)
uniform Globals {
    mat4 proj_mat;
};

layout(set=2, binding=0)
uniform PerQuad {
    mat4 model_mat;
};

void main() {
    v_tex_coords = a_tex_coords;
    gl_Position = proj_mat * model_mat * vec4(a_position, 1.0);
}