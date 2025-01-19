#version 330 core
layout(location = 0) in vec3 position;

out vec3 TexCoords;

uniform mat4 view;        // Rotation-only view matrix
uniform mat4 projection;  // Projection matrix

void main() {
    // Scale skybox size and apply transformations
    vec4 pos = projection * view * vec4(position, 1.0);
    gl_Position = pos; // Use full transformation pipeline
    TexCoords = position; // Pass original position for texture sampling
}
