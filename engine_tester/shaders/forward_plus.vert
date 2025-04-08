#version 430 core

//forward plus vert
layout(location = 0) in vec3 aPos;
layout(location = 1) in vec3 aNormal;
layout(location = 2) in vec2 aTexCoord;
//lol no texture coords
out vec3 FragPos;
out vec3 Normal;
out vec2 TexCoord;//lol I needed this actaully bc it was getting to frag and not finding it lol

uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;

void main() {
    vec4 worldPos = model * vec4(aPos, 1.0);
    FragPos = worldPos.xyz;
    Normal = mat3(transpose(inverse(model))) * aNormal;
    TexCoord = aTexCoord;

    gl_Position = projection * view * worldPos;
}
