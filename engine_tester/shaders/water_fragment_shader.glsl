#version 330 core

in vec2 TexCoord;
in vec3 WorldPos;
in vec3 Normal;

out vec4 FragColor;

uniform vec4 waterColor;
uniform vec3 lightPosition;
uniform vec4 lightColor;
uniform float waveScale;
uniform float timeFactor;
uniform float waveSpeed;

void main() {
    // Calculate wave effect
    float wave = sin(WorldPos.x * waveScale + (timeFactor * 0.2) * waveSpeed) + 
                 sin(WorldPos.z * waveScale + (timeFactor * 0.2) * waveSpeed);

    // Cell shading effect
    float cellShading = floor(wave * 3.0) / 3.0;

    // Define colors
    vec4 baseColor = waterColor;
    vec4 highlightColor = vec4(1.0, 1.0, 1.0, 0.5);
    vec4 shadowColor = vec4(0.0, 0.2, 0.4, 0.5);

    // Mix colors based on cell shading
    vec4 color = mix(
        mix(baseColor, shadowColor, 0.3),
        highlightColor, 
        cellShading * 0.5
    );

    // Apply alpha from waterColor
    color.a = waterColor.a;

    // Add a subtle animation effect
    color.rgb += sin(timeFactor * 2.0) * 0.001;

    // Light calculations
    vec3 lightDir = normalize(lightPosition - WorldPos);
    float distance = length(lightPosition - WorldPos);
    float attenuation = 1.0 / (distance * distance);

    float ambientStrength = 0.7;
    vec3 ambient = ambientStrength * lightColor.rgb * color.rgb;

    float diff = max(dot(Normal, lightDir), 0.0);
    vec3 diffuse = diff * lightColor.rgb * attenuation * color.rgb;

    vec3 viewDir = normalize(-WorldPos); // Assuming the camera is at (0,0,0)
    vec3 reflectDir = reflect(-lightDir, Normal);
    float spec = pow(max(dot(viewDir, reflectDir), 0.0), 30.0);
    vec3 specular = spec * lightColor.rgb * attenuation  * color.rgb;

    // Combine lighting with water color
    vec3 finalColor = ambient + diffuse + specular;
    finalColor = mix(waterColor.rgb, finalColor, 0.5);

    // Output the final color
    FragColor = vec4(finalColor, waterColor.a);
}