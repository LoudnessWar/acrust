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

layout(std430, binding = 2) readonly buffer VisibleLightIndicesBuffer {
    VisibleIndex data[];
} visibleLightIndicesBuffer;

uniform vec4 u_diffuseColor;
uniform float u_specularPower;
uniform int u_tileCountX;
uniform int u_lightCount;
uniform mat4 view;

// Added normals control uniforms
uniform float u_useNormalSmoothing;     // Toggle between flat and smooth normals
uniform float u_smoothingFactor;       // 0.0 = flat, 1.0 = fully smooth

layout(location = 0) out vec4 fragColor;

float attenuate(vec3 lightDirection, float radius) {
    float cutoff = 0.5;
    float attenuation = dot(lightDirection, lightDirection) / (100.0 * radius);
    attenuation = 1.0 / (attenuation * 15.0 + 1.0);
    attenuation = (attenuation - cutoff) / (1.0 - cutoff);
    return clamp(attenuation, 0.0, 1.0);
}

vec3 heatMapColor(float value) {
    const vec3 c1 = vec3(0.0, 0.0, 1.0);
    const vec3 c2 = vec3(0.0, 1.0, 1.0);
    const vec3 c3 = vec3(0.0, 1.0, 0.0);
    const vec3 c4 = vec3(1.0, 1.0, 0.0);
    const vec3 c5 = vec3(1.0, 0.0, 0.0);

    if (value < 0.25) return mix(c1, c2, value / 0.25);
    if (value < 0.5)  return mix(c2, c3, (value - 0.25) / 0.25);
    if (value < 0.75) return mix(c3, c4, (value - 0.5) / 0.25);
    return mix(c4, c5, (value - 0.75) / 0.25);
}

// Simple function to calculate flat normal from derivatives
vec3 calculateFlatNormal() {
    vec3 dFdxPos = dFdx(fragment_in.fragmentPosition);
    vec3 dFdyPos = dFdy(fragment_in.fragmentPosition);
    return normalize(cross(dFdxPos, dFdyPos));
}

void main() {
    ivec2 location = ivec2(gl_FragCoord.xy);
    ivec2 tileID = location / ivec2(16, 16);
    int tileIndex = tileID.y * u_tileCountX + tileID.x;
    int tileOffset = tileIndex * 256;

    //funny ones can remove later
    uint visibleLightCount = 0;
    float totalInfluence = 0.0;
    int lightCounter = 0;

    // Choose which normal to use based on smoothing settings
    vec3 inputNormal = normalize(fragment_in.normalVector);
    vec3 flatNormal = calculateFlatNormal();
    
    // Apply smoothing factor to blend between flat and smooth normals
    vec3 normal;
    if (u_useNormalSmoothing > 0.5) {
        normal = normalize(mix(flatNormal, inputNormal, u_smoothingFactor));
    } else {
        normal = inputNormal; // Use the input normal directly
    }
    
    // Check for degenerate normals
    if (length(normal) < 0.001) {
        normal = vec3(0.0, 1.0, 0.0); // Fallback normal
    }

    vec3 fragPos = fragment_in.fragmentPosition;
    vec3 viewDir = normalize(fragPos);

    vec3 color = vec3(0.0);

    for (int i = 0; i < u_lightCount; ++i) {
        int lightIndex = visibleLightIndicesBuffer.data[tileOffset + i].index;
        if (lightIndex < 0 || lightIndex >= lightBuffer.lights.length()) break;

        visibleLightCount++;

        Light light = lightBuffer.lights[lightIndex];
        vec3 lightPos = vec3(view * vec4(light.position, 1.0));
        vec3 lightDir = lightPos - fragPos;
        float distance = length(lightDir);

        if (distance > light.radius) continue;

        float attenuation = attenuate(lightDir, light.radius);
        totalInfluence += attenuation;

        lightDir = normalize(lightDir);

        float diff = max(dot(normal, lightDir), 0.0);
        vec3 halfway = normalize(lightDir + viewDir);
        float spec = pow(max(dot(normal, halfway), 0.0), u_specularPower);

        if (diff == 0.0) spec = 0.0;

        vec3 irradiance = light.color.rgb * ((u_diffuseColor.rgb * diff) + vec3(spec)) * attenuation;
        color += irradiance;
        lightCounter++;
    }

    color += u_diffuseColor.rgb * 0.08;
    if (u_useNormalSmoothing > 0.5) {
        color *= vec3(1.2, 1.0, 1.0);
    }

    int mode = 4; // Default visual mode
    
    switch (mode) {
        case 0:
            // Raw count of visible lights (grayscale)
            float ratio = float(visibleLightCount) / max(float(u_lightCount), 1.0);
            fragColor = vec4(vec3(ratio), 1.0);
            break;
            
        case 1:
            // Heatmap visualization of light count
            float normalizedCount = float(visibleLightCount) / 3.0;
            fragColor = vec4(heatMapColor(normalizedCount), 1.0);
            break;
            
        case 2:
            // Light influence visualization
            fragColor = vec4(heatMapColor(min(totalInfluence / 3.0, 1.0)), 1.0);
            break;
            
        case 3:
            // Normal visualization (helps validate geometry)
            fragColor = vec4(normal * 0.5 + 0.5, 1.0);
            break;
            
        case 4:
            fragColor = vec4(color, u_diffuseColor.a);
            if (length(normal) < 0.001) {
                fragColor = vec4(1.0, 0.0, 1.0, 1.0); // magenta = BAD normal
                return;
            }
            break;
            
        case 5:
            fragColor = vec4(color * u_specularPower, u_diffuseColor.a);
            break;
            
        case 6:
            fragColor = vec4(heatMapColor(float(lightCounter) / float(u_lightCount)), 1.0);
            break;
            
        case 7:
            float val = clamp(float(lightCounter) / 50.0, 0.0, 1.0);
            fragColor = vec4(val, val, val, 1.0);
            break;
            
        case 8:
            fragColor = vec4(sign(normal) * 0.5 + 0.5, 1.0);
            break;
            
        case 9: // Added new visualization mode to compare normals
            // Red = Flat normal, Green = Smooth normal, Yellow = Mix
            vec3 normalVis = mix(flatNormal * 0.5 + 0.5, inputNormal * 0.5 + 0.5, u_smoothingFactor);
            fragColor = vec4(normalVis, 1.0);
            break;
    }
}