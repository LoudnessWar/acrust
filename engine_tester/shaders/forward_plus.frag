#version 450

#define MAX_LIGHTS_PER_TILE 128

in vec3 FragPos;
in vec3 Normal;

layout(location = 0) out vec4 FragColor;

uniform vec3 cameraPos;

struct Light {
    vec3 position;
    float radius;
    vec3 color;
    float _pad;
};

layout(std430, binding = 1) buffer Lights {
    Light lights[];
};

layout(std430, binding = 2) buffer LightIndices {
    uint counts[];
    uint indices[];
};

uniform vec2 screenSize;
uniform vec2 tileSize;
uniform int numTilesX;

//dude... i like dont trust this at all but like thats life im sure it will work
void main() {
    //ok this is the junk to make sure it works with the tiles or whatever
    ivec2 tileCoord = ivec2(gl_FragCoord.xy / tileSize);
    int tileIndex = tileCoord.y * numTilesX + tileCoord.x;
    uint lightCount = counts[tileIndex];
    uint baseIndex = tileIndex * MAX_LIGHTS_PER_TILE;

    vec3 color = vec3(0.0);
    vec3 normal = normalize(Normal);
    vec3 viewDir = normalize(cameraPos - FragPos);

    //when the code is sus
    //im prolly going to make this erm... more bare bones lowkey
    for (uint i = 0; i < lightCount; ++i) {
        uint lightID = indices[baseIndex + i];
        Light light = lights[lightID];

        vec3 lightDir = light.position - FragPos;
        float distance = length(lightDir);
        lightDir /= distance;

        float attenuation = clamp(1.0 - distance / light.radius, 0.0, 1.0);
        float diffuse = max(dot(normal, lightDir), 0.0);

        color += light.color * diffuse * attenuation;
    }

    FragColor = vec4(color, 1.0);
}
