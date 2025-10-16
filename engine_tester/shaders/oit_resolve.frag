#version 330 core

in vec2 v_texCoord;
out vec4 fragColor;

uniform sampler2D u_accumTex;
uniform sampler2D u_revealTex;

void main() {
    vec4 accum = texture(u_accumTex, v_texCoord);
    float reveal = texture(u_revealTex, v_texCoord).r;
    
    // DEBUG: Visualize different components
    // Test 1: Show accum RGB directly (normalized)
    //fragColor = vec4(accum.rgb / 10000.0, 1.0); // Scale down to see it
    
    // Test 2: Show accum alpha
    //fragColor = vec4(vec3(accum.a / 10000.0), 1.0);
    
    // Test 3: Show reveal (should be darker where transparent objects are)
    //fragColor = vec4(vec3(reveal), 1.0);
    
    // Test 4: Show inverse reveal (brighter where transparent objects are)
    fragColor = vec4(vec3(1.0 - reveal), 1.0);
    //fragColor = vec4(1.0, 1.0, 0.0, 1.0);
    
    // Test 5: Final composite (uncomment when ready)
    /*
    if (accum.a <= 0.00001) {
        discard;
    }
    vec3 avgColor = accum.rgb / accum.a;
    fragColor = vec4(avgColor, 1.0 - reveal);
    */
}