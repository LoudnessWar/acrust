#version 330 core

// Vertex position attribute (layout location 0)
layout(location = 0) in vec3 aPos;

// Texture coordinate attribute (layout location 1)
layout(location = 1) in vec2 aTexCoord;

// Output to the fragment shader
out vec2 TexCoord;

// Uniform for projection matrix
uniform mat4 projection;

void main()
{
    // Transform the vertex position using the projection matrix
    gl_Position = projection * vec4(aPos, 1.0);

    // Pass texture coordinates to the fragment shader
    TexCoord = aTexCoord;
}
