#version 330 core
layout(location = 0) in vec3 position;
layout(location = 1) in vec2 texCoords;

uniform mat4 projection;

out vec2 TexCoords;

void main() {
    gl_Position = projection * vec4(position, 1.0);
    TexCoords = texCoords;
}
