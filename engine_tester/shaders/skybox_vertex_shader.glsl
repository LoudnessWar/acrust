#version 330 core
layout(location = 0) in vec3 position;

out vec3 TexCoords;

uniform mat4 view;
uniform mat4 projection;

void main() {
    vec3 pos = position * 3; // Scale up the skybox size
    TexCoords = position; // Use original position for texture sampling
    gl_Position = (projection * view * vec4(pos, 1.0)).xyww;
}