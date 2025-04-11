#version 450

layout (location = 0) in vec3 aPos;

uniform mat4 u_view_proj;
uniform mat4 u_model;

void main() {
    gl_Position = u_view_proj * u_model * vec4(aPos, 1.0);
}
