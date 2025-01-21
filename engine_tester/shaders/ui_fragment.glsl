#version 330 core
in vec2 TexCoords;

uniform sampler2D texture1;
uniform vec4 color;
uniform bool useTexture;

out vec4 FragColor;

void main() {
    if (useTexture) {
        FragColor = texture(texture1, TexCoords);
    } else {
        FragColor = color; // Fallback color
    }
}
