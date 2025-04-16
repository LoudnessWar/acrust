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
};

layout(std430, binding = 2) readonly buffer VisibleLightIndicesBuffer {
    VisibleIndex data[];
} visibleLightIndicesBuffer;

uniform vec4 u_diffuseColor;
uniform float u_specularPower;
uniform int u_tileCountX;
uniform int u_lightCount;
// uniform vec3 u_cameraPosition; // Not used for now

layout(location = 0) out vec4 fragColor;

vec3 heatMapColor(float value) {
    const vec3 c1 = vec3(0.0, 0.0, 1.0); // Blue for low values
    const vec3 c2 = vec3(0.0, 1.0, 1.0); // Cyan
    const vec3 c3 = vec3(0.0, 1.0, 0.0); // Green
    const vec3 c4 = vec3(1.0, 1.0, 0.0); // Yellow
    const vec3 c5 = vec3(1.0, 0.0, 0.0); // Red for high values
    
    float v = clamp(value, 0.0, 1.0);
    
    if (v < 0.25) {
        return mix(c1, c2, v/0.25);
    } else if (v < 0.5) {
        return mix(c2, c3, (v-0.25)/0.25);
    } else if (v < 0.75) {
        return mix(c3, c4, (v-0.5)/0.25);
    } else {
        return mix(c4, c5, (v-0.75)/0.25);
    }
}

void main() {
    ivec2 location = ivec2(gl_FragCoord.xy);
    ivec2 tileID = location / ivec2(16, 16);
    uint tileIndex = tileID.y * uint(u_tileCountX) + tileID.x;
    uint offset = tileIndex * 256;

    uint visibleLightCount = 0;
    float totalInfluence = 0.0;

    //vec3 normal = normalize(fragment_in.normalVector);
    vec3 normal = normalize(fragment_in.normalVector + vec3(0.001)); // avoid pure zero
    vec3 fragPos = fragment_in.fragmentPosition;

    vec3 finalColor = vec3(0.0);

    for (int i = 0; i < 256; ++i) {
        int lightIndex = visibleLightIndicesBuffer.data[offset + i].index;
        if (lightIndex == -1) break;

        visibleLightCount++;

        Light light = lights[lightIndex];
        vec3 lightDir = light.position - fragPos;
        float distance = length(lightDir);
        if (distance > light.radius) continue;// is making it > or like in a < r like >= like how much of a difference does it make???

        lightDir = normalize(lightDir);

        float attenuation = 1.0 - distance / light.radius;
        totalInfluence += attenuation;
        float diff = 1.0;// max(dot(normal, lightDir), 0.0);
        //float diff = max(dot(normal, lightDir), 0.0);

        vec3 lightIntensity = light.color * light.intensity * attenuation;
        finalColor += diff * lightIntensity * u_diffuseColor.rgb;
        //finalColor = vec3(1.0);
    }

    int mode = 4;
    
    if (location.x % 16 == 0 || location.y % 16 == 0) {
        fragColor = vec4(0.3, 0.3, 0.3, 1.0);
        return;
    }

    switch (mode) {
        case 0:
            // Raw count of visible lights (grayscale)
            float ratio = float(visibleLightCount) / max(float(u_lightCount), 1.0);
            fragColor = vec4(vec3(ratio), 1.0);
            break;
            
        case 1:
            // Heatmap visualization of light count
            float normalizedCount = float(visibleLightCount) / 3.0; // Adjust based on expected max lights per tile
            fragColor = vec4(heatMapColor(normalizedCount), 1.0);
            break;
            
        case 2:
            // Light influence visualization
            fragColor = vec4(heatMapColor(min(totalInfluence / 3.0, 1.0)), 1.0);
            break;
            
        case 3:
            // Normal visualization (helps validate geometry)
            //fragColor = vec4(normalize(fragment_in.normalVector) * 0.5 + 0.5, 1.0);
            fragColor = vec4(normal * 0.5 + 0.5, 1.0);
            break;
        case 4:
            fragColor = vec4(finalColor, u_diffuseColor.a);
            break;
        case 5:
            fragColor = vec4(finalColor * u_specularPower, u_diffuseColor.a);
            break;
    }


    if (length(fragment_in.normalVector) < 0.001) {
        fragColor = vec4(1.0, 0.0, 1.0, 1.0); // magenta = BAD normal
        return;
    }

    //fragColor = vec4(finalColor * (u_specularPower * 0.1), u_diffuseColor.a);
}