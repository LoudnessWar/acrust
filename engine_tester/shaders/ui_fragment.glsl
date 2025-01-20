#version 330 core

// Input texture coordinates from the vertex shader
in vec2 TexCoord;

// Output color
out vec4 FragColor;

// Texture sampler
uniform sampler2D texture1;

void main()
{
    // Fetch the texture color using the texture coordinates
    FragColor = texture(texture1, TexCoord);
}
