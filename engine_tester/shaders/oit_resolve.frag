#version 330 core

in vec2 v_texCoord;

out vec4 fragColor;

uniform sampler2D u_accumTex;  // RGBA16F accumulation
uniform sampler2D u_revealTex; // R16F revealage

void main() {
    // Sample the OIT textures
    vec4 accum = texture(u_accumTex, v_texCoord);
    float reveal = texture(u_revealTex, v_texCoord).r;
    
    // Suppress overflow (optional, helps with bright colors)
    if (isinf(accum.a)) {
        accum.rgb = vec3(accum.a);
    }
    
    // Prevent division by zero
    if (accum.a == 0.0) {
        discard;
    }
    
    // Calculate final color
    vec3 avgColor = accum.rgb / max(accum.a, 0.00001);
    
    // Blend with background using revealage
    fragColor = vec4(avgColor, 1.0 - reveal);
}