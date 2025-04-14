#version 430

in VERTEX_OUT {
    vec3 fragmentPosition;
    vec3 normalVector;
    vec2 textureCoordinates;
} fragment_in;

struct PointLight {
    vec4 color;
    vec4 position;
    vec4 paddingAndRadius;
};

struct VisibleIndex {
    int index;
};

// Shader storage buffer objects
layout(std430, binding = 0) readonly buffer LightBuffer{
    PointLight data[];
} lightBuffer;

layout(std430, binding = 1) readonly buffer VisibleLightIndicesBuffer{
    VisibleIndex data[];
} visibleLightIndicesBuffer;

uniform int numberOfTilesX;
uniform int totalLightCount;

out vec4 fragColor;

void main() {
    // Determine which tile this pixel belongs to
    ivec2 location = ivec2(gl_FragCoord.xy);
    ivec2 tileID = location / ivec2(16, 16);
    uint index = tileID.y * numberOfTilesX + tileID.x;

    // Count visible lights for this tile
    uint offset = index * 1024;
    uint i;
    for (i = 0; i < 1024 && visibleLightIndicesBuffer.data[offset + i].index != -1; i++);

    // Visualize the ratio of visible lights to total lights
    float ratio = float(i) / float(totalLightCount);
    fragColor = vec4(vec3(ratio, ratio, ratio), 1.0);
}
