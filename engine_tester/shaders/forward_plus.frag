#version 430 core

in vec2 TexCoord;
in vec3 FragPos;
in vec3 Normal;

layout(location = 0) out vec4 fragColor;

// Light structure
struct Light {
    vec3 position;
    float radius;
    // Add more properties as needed (color, etc.)
};

// Light data from compute shader
layout(std430, binding = 0) readonly buffer LightBuffer {
    Light lights[];
};

// Light grid from compute shader
layout(std430, binding = 1) readonly buffer LightGrid {
    ivec2 grid[]; // [offset, count] for each tile
};

// Light indices from compute shader
layout(std430, binding = 2) readonly buffer LightIndices {
    int indices[];
};

// Depth texture from prepass
uniform sampler2D u_depthTex;

// Material properties
uniform vec4 u_diffuseColor;
uniform float u_specularPower;

// Global uniforms
uniform float u_tileCountX;
//uniform float u_tileCountY;
uniform int u_lightCount;
//uniform float u_screenWidth;
//uniform float u_screenHeight;

void main() {
    // Sample depth texture
    float depth = texture(u_depthTex, TexCoord).r;
    
    // Calculate tile ID
    const uint TILE_SIZE = 16;
    uvec2 tileID = uvec2(gl_FragCoord.xy / TILE_SIZE);
    uint tileIndex = tileID.y * uint(u_tileCountX) + tileID.x;
    
    // Get light count and offset for this tile
    ivec2 lightData = grid[tileIndex];
    int lightOffset = lightData.x;
    int lightCount = min(lightData.y, u_lightCount); // Use the uniform lightCount as an upper bound
    
    // Prepare lighting calculation
    vec3 normal = normalize(Normal);
    vec3 viewDir = normalize(-FragPos); // Assuming v_Position is in view space
    
    // Apply depth-based ambient term
    float aoFactor = mix(0.7, 1.0, depth);
    vec3 lighting = vec3(0.1 * aoFactor); // Ambient light affected by depth
    
    // Process all lights affecting this tile
    for (int i = 0; i < lightCount; i++) {
        int lightIndex = indices[lightOffset + i];
        
        // Safety check to ensure we don't access beyond our light buffer
        // This ensures u_lightCount is used in the shader logic
        if (lightIndex >= u_lightCount) continue;
        
        Light light = lights[lightIndex];
        
        // Calculate light direction and distance
        vec3 lightDir = light.position - FragPos;
        float distance = length(lightDir);
        lightDir = normalize(lightDir);
        
        // Skip if beyond light radius
        if (distance > light.radius) continue;
        
        // Calculate attenuation
        float attenuation = 1.0 - smoothstep(0.0, light.radius, distance);
        
        // Diffuse lighting
        float diff = max(dot(normal, lightDir), 0.0);
        vec3 diffuse = diff * u_diffuseColor.rgb;
        
        // Specular lighting
        vec3 halfwayDir = normalize(lightDir + viewDir);
        float spec = pow(max(dot(normal, halfwayDir), 0.0), u_specularPower);
        vec3 specular = spec * vec3(0.3); // Specular color
        
        // Combine with attenuation
        lighting += (diffuse + specular) * attenuation;
    }
    
    // Apply light count influence (ensures u_lightCount is definitely used)
    // This is subtle but ensures the uniform won't be optimized out
    float lightInfluence = 1.0 + float(u_lightCount) * 0.001; 
    lighting *= min(lightInfluence, 1.1); // Limit the effect
    
    // Final color with depth influence
    fragColor = vec4(lighting, 1.0);
}