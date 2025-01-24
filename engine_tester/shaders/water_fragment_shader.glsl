#version 330 core

in vec2 TexCoord;
in vec3 WorldPos;
in vec3 Normal;

out vec4 FragColor;

uniform vec4 waterColor;
uniform vec3 lightPosition;
uniform vec4 lightColor;
//uniform float lightIntensity;

void main() {
    // Light calculations
    vec3 lightDir = normalize(lightPosition - WorldPos);
    float distance = length(lightPosition - WorldPos);
    float attenuation = 1.0 / (distance * distance);

    float ambientStrength = 0.7;
    vec3 ambient = ambientStrength * lightColor.rgb;

    float diff = max(dot(Normal, lightDir), 0.0);
    vec3 diffuse = diff * lightColor.rgb * attenuation;

    vec3 viewDir = normalize(-WorldPos); // Assuming the camera is at (0,0,0)
    vec3 reflectDir = reflect(-lightDir, Normal);
    float spec = pow(max(dot(viewDir, reflectDir), 0.0), 30.0);
    vec3 specular = spec * lightColor.rgb * attenuation;

    vec3 finalColor = ambient + diffuse + specular;
    finalColor = mix(waterColor.rgb, finalColor, 0.5); // Combine water color with lighting

    FragColor = vec4(finalColor, waterColor.a);
}
