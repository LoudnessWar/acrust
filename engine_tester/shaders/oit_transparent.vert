#version 430 core

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec2 texCoords;

out VERTEX_OUT {
    vec3 fragmentPosition;
    vec3 normalVector;
    vec2 textureCoordinates;
} vertex_out;

uniform mat4 projection;
uniform mat4 view;
uniform mat4 model;

void main() {
    gl_Position = projection * view * model * vec4(position, 1.0);
    
    vertex_out.fragmentPosition = vec3(view * model * vec4(position, 1.0));
    mat3 normalMatrix = transpose(inverse(mat3(view * model)));
    vertex_out.normalVector = normalize(normalMatrix * normal);

    vertex_out.textureCoordinates = texCoords;
}