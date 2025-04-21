// #version 430 core

// layout(location = 0) in vec3 position;
// layout(location = 1) in vec3 normal;
// layout(location = 2) in vec2 texCoords;
// layout(location = 3) in vec3 tangent;    // New
// layout(location = 4) in vec3 bitangent;  // New

// out VERTEX_OUT {
//     vec3 fragmentPosition;       // World space
//     vec2 textureCoordinates;
//     mat3 TBN;                   // Tangent-bitangent-normal matrix
//     vec3 tangentViewPosition;    // New
//     vec3 tangentFragmentPosition; // New
// } vertex_out;

// uniform mat4 projection;
// uniform mat4 view;
// uniform mat4 model;
// uniform vec3 viewPos;           // Camera position in world space

// void main() {
//     // Transform position to world and clip space
//     vec4 worldPosition = model * vec4(position, 1.0);
//     gl_Position = projection * view * worldPosition;
    
//     // Calculate TBN matrix
//     vec3 T = normalize(mat3(model) * tangent);
//     vec3 B = normalize(mat3(model) * bitangent);
//     vec3 N = normalize(mat3(model) * normal);
//     mat3 TBN = transpose(mat3(T, B, N));  // Transpose for tangent->world
    
//     // World space outputs
//     vertex_out.fragmentPosition = worldPosition.xyz;
//     vertex_out.textureCoordinates = texCoords;
    
//     // Tangent space outputs
//     vertex_out.TBN = TBN;
//     vertex_out.tangentViewPosition = TBN * viewPos;
//     vertex_out.tangentFragmentPosition = TBN * worldPosition.xyz;
// }

// #version 430 core

// layout(location = 0) in vec3 position;
// layout(location = 1) in vec3 normal;//i swapped these two and it did... something? TODO check this out???
// layout(location = 2) in vec2 texCoords;

// out VERTEX_OUT {//why do it like this you vapid silly baka... its like better ok like we did it in class once
//     vec3 fragmentPosition;
//     vec3 normalVector;
//     vec2 textureCoordinates;
// } vertex_out;

// // Uniforms
// uniform mat4 projection;
// uniform mat4 view;
// uniform mat4 model;

// void main() {
//     gl_Position = projection * view * model * vec4(position, 1.0);
    
//     // World space fragment position
//     // vertex_out.fragmentPosition = vec3(view * model * vec4(position, 1.0));
    
//     // // Transform normal to world space
//     // mat3 normalMatrix = transpose(inverse(mat3(view * model)));
//     // vertex_out.normalVector = normalize(normalMatrix * normal);

//     vertex_out.fragmentPosition = vec3(view * model * vec4(position, 1.0));
//     mat3 normalMatrix = transpose(inverse(mat3(view * model)));
//     vertex_out.normalVector = normalize(normalMatrix * normal);

//     // vertex_out.fragmentPosition = vec3(model * vec4(position, 1.0));
//     // mat3 normalMatrix = transpose(inverse(mat3(model)));
//     // vertex_out.normalVector = normalize(normalMatrix * normal);//change normal to position for magic 
    
//     // Pass texture coordinates to fragment shader
//     vertex_out.textureCoordinates = texCoords;
// }

#version 430 core

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec2 texCoords;

out VERTEX_OUT {
    vec3 fragmentPosition;
    vec3 normalVector;
    vec2 textureCoordinates;
    vec3 viewDirection;  // World space view direction
} vertex_out;

uniform mat4 projection;
uniform mat4 view;
uniform mat4 model;
uniform vec3 viewPos;

void main() {
    vec4 worldPosition = model * vec4(position, 1.0);
    gl_Position = projection * view * worldPosition;
    
    vertex_out.fragmentPosition = worldPosition.xyz;
    vertex_out.normalVector = normalize(mat3(transpose(inverse(model))) * normal);
    vertex_out.textureCoordinates = texCoords;
    vertex_out.viewDirection = normalize(viewPos - worldPosition.xyz);
}