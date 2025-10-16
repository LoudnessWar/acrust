#version 430 core

in VERTEX_OUT {
    vec3 fragmentPosition;
    vec3 normalVector;
    vec2 textureCoordinates;
} vertex_out;

// MRT outputs
layout(location = 0) out vec4 accumColor;
layout(location = 1) out float revealage;

// Material properties
uniform vec4 u_diffuseColor;
uniform float u_alpha; // Transparency: 0.0 = fully transparent, 1.0 = opaque
uniform int u_lightCount;
uniform int u_tileCountX;

// Optional: Add your light buffers if you want lit transparency
// layout(std430, binding = 0) buffer LightBuffer { ... };

void main() {
    // 1. Calculate base color
    vec4 color = u_diffuseColor;
    
    // Optional: Add basic lighting using your normalVector
    vec3 normal = normalize(vertex_out.normalVector);
    // ... your lighting calculations here if needed ...
    
    // Apply alpha
    color.a = u_alpha;
    
    // 2. Calculate weight for weighted blended OIT
    float z = gl_FragCoord.z;
    float weight = clamp(pow(min(1.0, color.a * 10.0) + 0.01, 3.0) * 1e8 * 
                         pow(1.0 - z * 0.9, 3.0), 1e-2, 3e3);
    
    // 3. Output to MRTs
    // Accumulation: premultiplied color * weight
    accumColor = vec4(color.rgb * color.a, color.a) * weight;
    
    // Revealage: (1 - alpha)
    revealage = color.a;
}