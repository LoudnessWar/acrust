// #version 450

// layout(location = 0) out vec4 outColor;

// in vec2 v_uv; // from vertex shader, ranging [0, 1]
// uniform sampler2D u_debugTex;

// void main() {
//     vec4 debug = texture(u_debugTex, v_uv);
//     outColor = debug;
// }
#version 330 core
in vec2 TexCoord;
out vec4 FragColor;

uniform sampler2D debugTexture;

void main() {
    // Sample the debug texture
    vec4 debugColor = texture(debugTexture, TexCoord);
    
    // Optional: Adjust brightness to make easier to see
    //debugColor.rgb *= 5.0; // Multiply to make dim lights more visible
    
    FragColor = debugColor;//vec4(debugColor, 1.0);
}