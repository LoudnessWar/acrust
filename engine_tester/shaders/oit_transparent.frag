#version 430 core

in VERTEX_OUT {
    vec3 fragmentPosition;  // View space
    vec3 normalVector;      // View space
    vec2 textureCoordinates;
} vertex_out;

// MRT outputs
layout(location = 0) out vec4 accumColor;
layout(location = 1) out float revealage;

// Light structure
struct Light {
    vec3 position;
    float radius;
    vec3 color;
    float intensity;
};

struct VisibleIndex {
    int index;
};

// Buffer definitions
layout(std430, binding = 0) readonly buffer LightBuffer {
    Light lights[];
} lightBuffer;

layout(std430, binding = 2) readonly buffer VisibleLightIndicesBuffer {
    VisibleIndex data[];
} visibleLightIndicesBuffer;

// Uniforms
uniform vec4 u_diffuseColor;
uniform float u_alpha;
//uniform float u_specularPower;
uniform int u_tileCountX;
uniform int u_lightCount;
uniform mat4 view;

// Glass properties
uniform float u_ior;  // Index of refraction (default 1.52 for glass)
uniform float u_roughness;  // 0.0 = perfect glass, higher = frosted
//uniform vec3 u_tintColor;  // Color tint for the glass

// Attenuation function
float attenuate(vec3 lightDirection, float radius) {
    float distance = length(lightDirection);
    float normalizedDist = distance / radius;
    float attenuation = 1.0 / (1.0 + 25.0 * normalizedDist * normalizedDist);
    return smoothstep(0.0, 1.0, attenuation);
}

// Schlick's approximation for Fresnel
float fresnel_schlick(float cosTheta, float ior) {
    float r0 = pow((1.0 - ior) / (1.0 + ior), 2.0);
    return r0 + (1.0 - r0) * pow(1.0 - cosTheta, 5.0);
}

// More accurate Fresnel for dielectrics
float fresnel_dielectric(float cosTheta, float ior) {
    float sinTheta = sqrt(1.0 - cosTheta * cosTheta);
    float sinThetaT = sinTheta / ior;
    
    // Total internal reflection
    if (sinThetaT >= 1.0) return 1.0;
    
    float cosThetaT = sqrt(1.0 - sinThetaT * sinThetaT);
    
    float rs = (cosTheta - ior * cosThetaT) / (cosTheta + ior * cosThetaT);
    float rp = (ior * cosTheta - cosThetaT) / (ior * cosTheta + cosThetaT);
    
    return (rs * rs + rp * rp) * 0.5;
}

// GGX normal distribution for roughness
float ggx_distribution(float NdotH, float roughness) {
    float a = roughness * roughness;
    float a2 = a * a;
    float NdotH2 = NdotH * NdotH;
    float denom = (NdotH2 * (a2 - 1.0) + 1.0);
    return a2 / (3.14159265359 * denom * denom);
}

void main() {
    ivec2 location = ivec2(gl_FragCoord.xy);
    ivec2 tileID = location / ivec2(16, 16);
    int tileIndex = tileID.y * u_tileCountX + tileID.x;
    int tileOffset = tileIndex * 256;

    vec3 normal = normalize(vertex_out.normalVector);
    vec3 fragPos = vertex_out.fragmentPosition;
    vec3 viewDir = normalize(-fragPos);
    
    // Ensure normal faces camera
    normal = faceforward(normal, -viewDir, normal);
    
    float NdotV = abs(dot(normal, viewDir));
    
    // Glass properties
    float ior = u_ior > 0.0 ? u_ior : 1.52;
    float roughness = clamp(u_roughness, 0.0, 1.0);
    vec3 u_tintColor = vec3(0.0, 0.0, 0.0);
    vec3 tint = length(u_tintColor) > 0.0 ? u_tintColor : vec3(1.0);
    
    // Calculate Fresnel effect
    float fresnel = fresnel_dielectric(NdotV, ior);
    
    // Refraction direction (for color absorption simulation)
    vec3 refractDir = refract(-viewDir, normal, 1.0 / ior);
    float refractAmount = length(refractDir);
    
    // Calculate thickness approximation for absorption
    float thickness = (1.0 - NdotV) * 2.0;  // Thicker at grazing angles
    
    // Absorption (Beer's law approximation)
    vec3 absorption = exp(-tint * thickness * 0.5);
    
    // Lighting calculations
    vec3 diffuse = vec3(0.0);
    vec3 specular = vec3(0.0);
    
    for (int i = 0; i < u_lightCount; ++i) {
        int lightIndex = visibleLightIndicesBuffer.data[tileOffset + i].index;
        if (lightIndex < 0 || lightIndex >= lightBuffer.lights.length()) break;

        Light light = lightBuffer.lights[lightIndex];
        vec3 lightPos = vec3(view * vec4(light.position, 1.0));
        vec3 lightVec = lightPos - fragPos;
        float distance = length(lightVec);

        if (distance >= light.radius) continue;

        float attenuation = attenuate(lightVec, light.radius);
        vec3 lightDir = normalize(lightVec);
        
        // Minimal diffuse (glass is mostly specular)
        float diff = abs(dot(normal, lightDir));
        diffuse += light.color * diff * attenuation * 0.05;
        
        // Specular reflection with roughness
        vec3 halfVec = normalize(lightDir + viewDir);
        float NdotH = max(dot(normal, halfVec), 0.0);
        
        float D = ggx_distribution(NdotH, roughness);
        float specPower = mix(128.0, 16.0, roughness);
        float spec = pow(NdotH, specPower) * D;
        
        specular += light.color * spec * attenuation * fresnel;
    }
    
    // Combine lighting with glass properties
    vec3 baseColor = u_diffuseColor.rgb;
    
    // Transmitted light (refracted) with absorption
    vec3 transmitted = baseColor * absorption * (1.0 - fresnel) * 0.8;
    
    // Reflected light
    vec3 reflected = specular * 2.0;
    
    // Minimal ambient
    vec3 ambient = baseColor * 0.01;
    
    vec3 result = transmitted + reflected + ambient + diffuse * baseColor * 0.1;
    
    // Vary alpha based on viewing angle (Fresnel for transparency)
    float alpha = mix(u_alpha, min(1.0, u_alpha * 2.5), fresnel);
    
    // Edge highlighting
    float edge = pow(1.0 - NdotV, 2.0);
    result += vec3(1.0) * edge * 0.05;
    
    vec4 color = vec4(result, alpha);
    
    // OIT weight calculation (tuned for glass)
    float z = gl_FragCoord.z;
    float weight = color.a * max(0.01, min(3000.0, 
        10.0 / (0.00001 + pow(z / 200.0, 4.0) + pow(1.0 - fresnel, 2.0) * 0.5)));
    
    accumColor = vec4(color.rgb * color.a, color.a) * weight;
    revealage = color.a;
}