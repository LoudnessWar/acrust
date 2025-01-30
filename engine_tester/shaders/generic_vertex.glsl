#version 330 core

layout (location = 0) in vec3 aPos;   // Vertex Position
layout (location = 1) in vec3 aNormal; // Normal Vector
layout (location = 2) in vec2 aTexCoord; // Texture Coordinates

out vec3 FragPos;  // Position in World Space
out vec3 Normal;   // Normal in World Space
out vec2 TexCoord; // Texture Coordinates

uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;

void main() {
    FragPos = vec3(model * vec4(aPos, 1.0)); // Transform to world space
    Normal = mat3(transpose(inverse(model))) * aNormal; // Normal transformation
    TexCoord = aTexCoord;

    gl_Position = projection * view * vec4(FragPos, 1.0); // Final Position
}