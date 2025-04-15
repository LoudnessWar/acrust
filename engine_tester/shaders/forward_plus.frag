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

layout(location = 0) out vec4 fragColor;


void main() {
    // Calculate tile ID
    // const uint TILE_SIZE = 16;
    // uvec2 tileID = uvec2(gl_FragCoord.xy / TILE_SIZE);
    // uint tileIndex = tileID.y * uint(u_tileCountX) + tileID.x;
    ivec2 location = ivec2(gl_FragCoord.xy);
    ivec2 tileID = location / ivec2(16, 16);
    uint index = tileID.y * u_tileCountX + tileID.x;
    
    // Calculate offset into visible light indices buffer
    uint lightOffset = index * 256; // Assuming MAX_LIGHTS_PER_TILE = 256
    
    // Prepare lighting calculation
    vec3 normal = normalize(fragment_in.normalVector);
    vec3 viewDir = normalize(-fragment_in.fragmentPosition); // Assuming FragPos is in view space
    
    // Simple ambient light
    vec3 lighting = vec3(0.1); // Constant ambient term
    
    // Process all lights affecting this tile
    for (int i = 0; i < 256; i++) { // Process up to the maximum number of lights per tile
        int lightIndex = visibleLightIndicesBuffer.data[lightOffset + i].index;
        
        //the double check here TODO remove
        if (lightIndex == -1) break;
        if (lightIndex < 0 || lightIndex >= u_lightCount) break;
        
        Light light = lights[lightIndex];
        vec3 lightColor = light.color * light.intensity;
        
        // Calculate light direction and distance
        vec3 fragWorldPos = vec3(inverse(view) * vec4(fragment_in.fragmentPosition, 1.0));
        vec3 lightDir = light.position - fragWorldPos;
        //vec3 lightDir = light.position - fragment_in.fragmentPosition;
        float distance = length(lightDir);
        lightDir = normalize(lightDir);
        
        // Skip if beyond light radius
        if (distance > light.radius) continue;
        
        // Calculate attenuation
        float attenuation = max(0.0, 1.0 - distance / light.radius);
        
        // Diffuse lighting
        float diff = max(dot(normal, lightDir), 0.0);
        vec3 diffuse = lightColor * diff * u_diffuseColor.rgb;
        
        // Specular lighting
        vec3 halfwayDir = normalize(lightDir + viewDir);
        float spec = pow(max(dot(normal, halfwayDir), 0.0), u_specularPower);
        vec3 specular = lightColor * spec * vec3(0.3);
        
        // Combine with attenuation
        lighting += (diffuse * 2.0 + specular) * attenuation;
    }
    
    // Final color
    fragColor = vec4(lighting * 2.0, 1.0);
    //fragColor = vec4(fragment_in.fragmentPosition * 0.01, 1.0);
}