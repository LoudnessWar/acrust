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
uniform float u_specularPower;
uniform int u_tileCountX;
uniform int u_lightCount;
uniform mat4 view;

// Attenuation function (same as opaque shader)
float attenuate(vec3 lightDirection, float radius) {
    float distance = length(lightDirection);
    float normalizedDist = distance / radius;
    float attenuation = 1.0 / (1.0 + 25.0 * normalizedDist * normalizedDist);
    return smoothstep(0.0, 1.0, attenuation);
}

void main() {
    ivec2 location = ivec2(gl_FragCoord.xy);
    ivec2 tileID = location / ivec2(16, 16);
    int tileIndex = tileID.y * u_tileCountX + tileID.x;
    int tileOffset = tileIndex * 256;

    vec3 normal = normalize(vertex_out.normalVector);
    vec3 fragPos = vertex_out.fragmentPosition;
    vec3 viewDir = normalize(-fragPos);
    
    // Ensure normal faces camera for transparent surfaces
    normal = faceforward(normal, -viewDir, normal);

    vec3 diffuse = vec3(0.0);
    vec3 specular = vec3(0.0);

    // Process lights
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

        // Diffuse (reduced for glass - mostly specular)
        float diff = abs(dot(normal, lightDir));  // abs() for two-sided
        diffuse += light.color * diff * attenuation * 0.3;  // Reduced diffuse

        // Specular (main reflection for glass)
        vec3 reflectDir = reflect(-lightDir, normal);
        float shininess = clamp(u_specularPower, 1.0, 128.0);  // Higher for glass
        float spec = pow(max(dot(viewDir, reflectDir), 0.0), shininess);
        
        specular += light.color * spec * attenuation;
    }

    // Glass properties
    vec3 baseColor = u_diffuseColor.rgb;
    vec3 result = (diffuse * baseColor * 0.5) + (specular * 1.5);  // More specular
    result += baseColor * 0.02;  // Minimal ambient

    // Fresnel for transparency
    float fresnel = pow(1.0 - abs(dot(viewDir, normal)), 3.0);
    float edgeAlpha = mix(u_alpha, min(1.0, u_alpha * 2.0), fresnel);
    
    // Absorption (optional - for colored glass)
    float thickness = 1.0 - abs(dot(viewDir, normal));
    vec3 absorption = pow(vec3(0.95, 0.97, 1.0), vec3(thickness));
    result *= absorption;

    vec4 color = vec4(result, edgeAlpha);

    // OIT weight calculation
    float z = gl_FragCoord.z;
    float weight = color.a * max(0.01, min(3000.0, 10.0 / (0.00001 + pow(z / 200.0, 4.0))));

    accumColor = vec4(color.rgb * color.a, color.a) * weight;
    revealage = color.a;
}