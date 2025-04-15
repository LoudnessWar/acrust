#version 430 core

in VERTEX_OUT {
    vec3 fragmentPosition;
    vec3 normalVector;
    vec2 textureCoordinates;
} fragment_in;

struct Light {
    vec3 position;
    float radius;
    vec3 color;
    float intensity;
};

struct VisibleIndex {
    int index;
};

layout(std430, binding = 0) readonly buffer LightBuffer {
    Light lights[];
} lightBuffer;

layout(std430, binding = 2) readonly buffer VisibleLightIndicesBuffer {//TODO Change this to binding 1
    VisibleIndex data[];
} visibleLightIndicesBuffer;

// Material properties
uniform vec4 u_diffuseColor;
uniform float u_specularPower;

// Global uniforms
uniform int u_tileCountX;
uniform int u_lightCount;

uniform mat4 view;

out vec4 fragColor;

void main() {
    ivec2 location = ivec2(gl_FragCoord.xy);
    ivec2 tileID = location / ivec2(16, 16);
    uint index = tileID.y * u_tileCountX + tileID.x;
    uint offset = index * 256;

    float totalInfluence = 0.0;
    int lightCount = 0;

    vec3 normal = normalize(fragment_in.normalVector);
    vec3 viewDir = normalize(-fragment_in.fragmentPosition);

    for (int i = 0; i < 256; i++) {
        int lightIndex = visibleLightIndicesBuffer.data[offset + i].index;
        if (lightIndex == -1) break;

        Light light = lightBuffer.lights[lightIndex];
        vec3 lightDir = light.position - fragment_in.fragmentPosition;
        float dist = length(lightDir);

        if (dist < light.radius) {
            vec3 L = normalize(lightDir);
            float attenuation = 1.0 - dist / light.radius;

            float diffuse = max(dot(normal, L), 0.0);

            vec3 halfVec = normalize(L + viewDir);
            float specular = pow(max(dot(normal, halfVec), 0.0), u_specularPower);

            float lightContribution = (diffuse + specular) * attenuation * light.intensity;
            totalInfluence += lightContribution;
            lightCount++;
        }
    }

    if (location.x % 16 == 0 || location.y % 16 == 0) {
        fragColor = vec4(0.2, 0.2, 0.2, 1.0); // grid
        return;
    }

    float normInfluence = clamp(totalInfluence / 10.0, 0.0, 1.0);
    float normCount = float(lightCount) / float(max(u_lightCount, 1));

    vec3 finalColor = mix(vec3(0.0), u_diffuseColor.rgb, normInfluence);
    fragColor = vec4(finalColor + vec3(normCount * 0.2), u_diffuseColor.a);
}