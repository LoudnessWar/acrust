#version 330 core

layout(location = 0) in vec3 aPos; // Vertex position
layout(location = 1) in vec2 aTexCoord; // Texture coordinates

out vec2 TexCoord;
out vec3 WorldPos;
out vec3 Normal;

uniform mat4 model;
uniform mat4 view;       // From the camera
uniform mat4 projection; // From the camera
uniform float waveSpeed;
uniform float waveScale;
uniform float timeFactor;
uniform float waveHeight;

float noise(vec2 p) {
    return fract(sin(dot(p, vec2(12.9898, 78.233))) * 43758.5453);
}

void main() {
    vec3 worldPos = vec3(model * vec4(aPos, 1.0));

    // Calculate wave displacement
    float wave = sin(worldPos.x * waveScale + timeFactor * waveSpeed) +
                 sin(worldPos.z * waveScale * 0.5 + timeFactor * waveSpeed) * 0.5 +
                 sin(worldPos.z * waveScale * 0.25 + timeFactor * waveSpeed) * 0.25 +
                 noise(worldPos.xz * 0.1) * 0.5;

    worldPos.y += wave * waveHeight;

    TexCoord = aTexCoord;
    WorldPos = worldPos;
    Normal = normalize(vec3(-waveHeight * waveScale * cos((worldPos.x + timeFactor) * waveScale), 
                             1.0, 
                             -waveHeight * waveScale * cos((worldPos.z + timeFactor) * waveScale)));
    gl_Position = projection * view * vec4(worldPos, 1.0);
}