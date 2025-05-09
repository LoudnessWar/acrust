#version 430

layout (location = 0) in vec3 aPos;

// Uniforms
uniform mat4 projection;
uniform mat4 view;
uniform mat4 model;

void main() {
	gl_Position = projection * view * model * vec4(aPos, 1.0);
}