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
    // Calculate tile information
    ivec2 location = ivec2(gl_FragCoord.xy);
    ivec2 tileID = location / ivec2(16, 16);
    int tileIndex = tileID.y * u_tileCountX + tileID.x;
    int tileOffset = tileIndex * 256;

    // Prepare lighting variables
    vec3 normal = normalize(vertex_out.normalVector);
    vec3 fragPos = vertex_out.fragmentPosition;
    vec3 viewDir = normalize(-fragPos);

    // Flip normal if facing away
    if (dot(normal, viewDir) < 0.0)
        normal = -normal;

    vec3 diffuse = vec3(0.0);
    vec3 specular = vec3(0.0);

    // Process each visible light in this tile
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

        // Diffuse lighting
        float diff = max(dot(normal, lightDir), 0.0);
        diffuse += light.color * diff * attenuation;

        // Specular lighting
        vec3 reflectDir = reflect(-lightDir, normal);
        float shininess = clamp(u_specularPower, 1.0, 64.0);
        float spec = pow(max(dot(viewDir, reflectDir), 0.0), shininess);

        /*
        // Energy conservation
        spec *= (1.0 - diff);

        // Fresnel term
        
        float fresnel = pow(1.0 - max(dot(viewDir, normal), 0.0), 5.0);
        fresnel = mix(0.1, 1.0, fresnel);
        spec *= fresnel;
        */

        specular += light.color * spec * attenuation;
    }

    // Combine lighting
    vec3 result = (diffuse * u_diffuseColor.rgb) + specular;
    result += u_diffuseColor.rgb * 0.03;  // Ambient

    // Apply alpha
    vec4 color = vec4(result, u_alpha);

    // Calculate weight for OIT
    float z = gl_FragCoord.z;
    float weight = color.a * max(0.01, min(3000.0, 10.0 / (0.00001 + pow(z / 200.0, 4.0))));

    // Output to MRTs
    accumColor = vec4(color.rgb * color.a, color.a) * weight;
    revealage = color.a;
}