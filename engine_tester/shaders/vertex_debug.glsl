#version 330 core

layout (location = 0) in vec3 aPos;   // Vertex Position
layout (location = 1) in vec3 aNormal; // Normal Vector but lowkey maybe for this one it should be color... meeehhhh

out vec3 fragColor;  // color this time lol

uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;

void main() {
    FragPos = vec3(model * vec4(aPos, 1.0));
    fragColor = aNormal;

    gl_Position = projection * view * vec4(FragPos, 1.0);
}