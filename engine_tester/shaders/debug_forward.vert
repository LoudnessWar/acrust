#version 330 core

layout(location = 0) in vec3 position;
layout(location = 2) in vec3 normal;//i swapped these two and it did... something?
layout(location = 1) in vec2 texCoords;

out VERTEX_OUT {//why do it like this you vapid silly baka... its like better ok like we did it in class once
    vec3 fragmentPosition;
    vec3 normalVector;
    vec2 textureCoordinates;
} vertex_out;

// Uniforms
uniform mat4 projection;
uniform mat4 view;
uniform mat4 model;

void main() {
    gl_Position = projection * view * model * vec4(position, 1.0);
    
    // World space fragment position
    vertex_out.fragmentPosition = vec3(model * vec4(position, 1.0));
    
    // Transform normal to world space
    mat3 normalMatrix = transpose(inverse(mat3(model)));
    vertex_out.normalVector = normalize(normalMatrix * normal);
    
    // Pass texture coordinates to fragment shader
    vertex_out.textureCoordinates = texCoords;
}