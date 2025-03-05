#version 330 core

in vec3 FragPos;
in vec3 Normal;
in vec2 TexCoord;

out vec4 FragColor;

//uniform sampler2D texture_diffuse; // Diffuse Texture
uniform vec3 lightDir;  // Directional light direction
uniform vec3 lightColor; // Light color
uniform vec3 objectColor; // Fallback color if no texture

void main() {
    // Normalize vectors
    vec3 norm = normalize(Normal);
    vec3 light = normalize(-lightDir); // Reverse because light shines TO the object

    // Diffuse lighting (basic Lambertian reflection)
    float diff = max(dot(norm, light), 0.0);
    vec3 diffuse = diff * lightColor;

    // Texture or solid color fallback
    //vec3 textureColor = texture(texture_diffuse, TexCoord).rgb;
    //vec3 finalColor = mix(objectColor, textureColor, 1.0);

    FragColor = vec4(objectColor, 1.0);
}
