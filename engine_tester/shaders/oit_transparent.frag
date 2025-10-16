#version 430 core

in VERTEX_OUT {
    vec3 fragmentPosition;
    vec3 normalVector;
    vec2 textureCoordinates;
} vertex_out;

layout(location = 0) out vec4 accumColor;
layout(location = 1) out float revealage;

uniform vec4 u_diffuseColor;
uniform float u_alpha;

void main() {
    vec4 color = u_diffuseColor;
    color.a = u_alpha;

    float z = gl_FragCoord.z;
    float weight = clamp(pow(min(1.0, color.a * 10.0) + 0.01, 3.0) *
                         1e8 * pow(1.0 - z * 0.9, 3.0), 1e-2, 3e3);

    accumColor = vec4(color.rgb * color.a, color.a) * weight;
    revealage = 1.0 - color.a;  // ✅ correct
}
