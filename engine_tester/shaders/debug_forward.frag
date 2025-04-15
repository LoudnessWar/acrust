#version 430

in VERTEX_OUT {
    vec3 fragmentPosition;
    vec3 normalVector;
    vec2 textureCoordinates;
} fragment_in;

struct Light {//TODO make pos and color vec4 so no casting them later on when used
    vec3 position;
    float radius;
    vec3 color;
    float intensity;
};

struct VisibleIndex {
    int index;
};

// Shader storage buffer objects
layout(std430, binding = 0) readonly buffer LightBuffer{
    Light data[];
} lightBuffer;

layout(std430, binding = 2) readonly buffer VisibleLightIndicesBuffer{
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
    uint offset = index *  256;
    uint i;
    for (i = 0; i < 256 && visibleLightIndicesBuffer.data[offset + i].index != -1; i++);

    // Visualize the ratio of visible lights to total lights
    float ratio = float(i) / float(totalLightCount);
    // if (totalLightCount <= 0) {
    //     fragColor = vec4(1.0, 0.0, 0.0, 1.0); // Red if no lights at all
    //     return;
    // }
    if (location.x % 16 == 0 || location.y % 16 == 0) {
        fragColor = vec4(0.0, 1.0, 0.0, 1.0); // Green grid lines
        return;
    }
    fragColor = vec4(vec3(ratio, ratio, ratio), 1.0);
    //fragColor = vec4(normalize(fragment_in.normalVector) * 0.5 + 0.5, 1.0);
}

