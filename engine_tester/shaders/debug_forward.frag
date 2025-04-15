#version 430

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

// Shader storage buffer objects
layout(std430, binding = 0) readonly buffer LightBuffer {
    Light data[];
} lightBuffer;

layout(std430, binding = 2) readonly buffer VisibleLightIndicesBuffer {
    VisibleIndex data[];
} visibleLightIndicesBuffer;

uniform int numberOfTilesX;
uniform int totalLightCount;
uniform mat4 view;
uniform mat4 projection;
uniform vec3 cameraPosition;

out vec4 fragColor;

// This will help visualize why lights are getting culled
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
    // Determine which tile this pixel belongs to
    ivec2 location = ivec2(gl_FragCoord.xy);
    ivec2 tileID = location / ivec2(16, 16);
    uint index = tileID.y * numberOfTilesX + tileID.x;

    // Count visible lights for this tile
    uint offset = index * 256;
    uint visibleLightCount = 0;
    
    // Count and check if this fragment is actually affected by any lights
    float totalInfluence = 0.0;
    
    for (int i = 0; i < 256; i++) {
        int lightIndex = visibleLightIndicesBuffer.data[offset + i].index;
        if (lightIndex == -1) break;
        
        visibleLightCount++;
        
        // Calculate actual influence of this light on this fragment
        Light light = lightBuffer.data[lightIndex];
        vec3 lightDir = light.position - fragment_in.fragmentPosition;
        float distance = length(lightDir);
        
        if (distance < light.radius) {
            // Light influences this fragment
            float attenuation = 1.0 - (distance / light.radius);
            totalInfluence += attenuation;
        }
    }
    
    // Visualization options
    int mode = 2; // Change this to select different visualization modes
    
    if (location.x % 16 == 0 || location.y % 16 == 0) {
        // Grid lines for tile visualization
        fragColor = vec4(0.3, 0.3, 0.3, 1.0);
        return;
    }
    
    // Choose visualization mode
    switch (mode) {
        case 0:
            // Raw count of visible lights (grayscale)
            float ratio = float(visibleLightCount) / max(float(totalLightCount), 1.0);
            fragColor = vec4(vec3(ratio), 1.0);
            break;
            
        case 1:
            // Heatmap visualization of light count
            float normalizedCount = float(visibleLightCount) / 20.0; // Adjust based on expected max lights per tile
            fragColor = vec4(heatMapColor(normalizedCount), 1.0);
            break;
            
        case 2:
            // Light influence visualization
            fragColor = vec4(heatMapColor(min(totalInfluence / 3.0, 1.0)), 1.0);
            break;
            
        case 3:
            // Normal visualization (helps validate geometry)
            fragColor = vec4(normalize(fragment_in.normalVector) * 0.5 + 0.5, 1.0);
            break;
    }
}