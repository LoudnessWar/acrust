#version 430 core

in VERTEX_OUT {
    vec3 fragmentPosition;  // Now in view space
    vec3 normalVector;      // Now in view space
    vec2 textureCoordinates;
} fragment_in;

// Light structure (keep your existing)
struct Light {
    vec3 position;
    float radius;
    vec3 color;
    float intensity;
};

struct VisibleIndex {
    int index;
};

// Buffer definitions (keep your existing)
layout(std430, binding = 0) readonly buffer LightBuffer {
    Light lights[];
} lightBuffer;

layout(std430, binding = 2) readonly buffer VisibleLightIndicesBuffer {
    VisibleIndex data[];
} visibleLightIndicesBuffer;

// Uniforms
uniform vec4 u_diffuseColor;
uniform float u_specularPower;
uniform int u_tileCountX;
uniform int u_lightCount;
uniform mat4 view;

out vec4 fragColor;

// Improved attenuation function
// float attenuate(vec3 lightDirection, float radius) {
//     float distance = length(lightDirection);
//     float normalizedDist = distance / radius;
//     float attenuation = 1.0 / (1.0 + 25.0 * normalizedDist * normalizedDist);
//     return smoothstep(0.0, 1.0, attenuation);
// }

// float attenuate(vec3 lightDir, float radius) {
//     float d = length(lightDir);
//     float norm = d / radius;
//     float att = pow(1.0 - clamp(norm, 0.0, 1.0), 2.0); // quadratic smooth fade
//     return att;
// }

// float attenuate(vec3 lightDirection, float radius) {
//     float distance = length(lightDirection);
//     float normDist = distance / radius;
//     float falloff = clamp(1.0 - normDist, 0.0, 1.0);
//     return falloff * falloff;
// }

float attenuate(vec3 lightDirection, float radius) {
    float distance = length(lightDirection);
    float normalizedDist = distance / radius;
    float attenuation = 1.0 / (1.0 + 25.0 * normalizedDist * normalizedDist);
    return smoothstep(0.0, 1.0, attenuation);
}

vec3 heatMapColor(float value) {
    const vec3 c1 = vec3(0.0, 0.0, 1.0);
    const vec3 c2 = vec3(0.0, 1.0, 1.0);
    const vec3 c3 = vec3(0.0, 1.0, 0.0);
    const vec3 c4 = vec3(1.0, 1.0, 0.0);
    const vec3 c5 = vec3(1.0, 0.0, 0.0);

    if (value < 0.25) return mix(c1, c2, value / 0.25);
    if (value < 0.5)  return mix(c2, c3, (value - 0.25) / 0.25);
    if (value < 0.75) return mix(c3, c4, (value - 0.5) / 0.25);
    return mix(c4, c5, (value - 0.75) / 0.25);
}

vec3 calculateFlatNormal() {
    vec3 dFdxPos = dFdx(fragment_in.fragmentPosition);
    vec3 dFdyPos = dFdy(fragment_in.fragmentPosition);
    return normalize(cross(dFdxPos, dFdyPos));
}


void main() {
    ivec2 location = ivec2(gl_FragCoord.xy);
    ivec2 tileID = location / ivec2(16, 16);
    int tileIndex = tileID.y * u_tileCountX + tileID.x;
    int tileOffset = tileIndex * 256;

    uint visibleLightCount = 0;
    float totalInfluence = 0.0;
    int lightCounter = 0;

    vec3 normal = normalize(fragment_in.normalVector);
    vec3 fragPos = fragment_in.fragmentPosition;
    vec3 viewDir = normalize(-fragPos);

    if (dot(normal, viewDir) < 0.0)
        normal = -normal;

    vec3 diffuse = vec3(0.0);
    vec3 specular = vec3(0.0);

    for (int i = 0; i < u_lightCount; ++i) {
        int lightIndex = visibleLightIndicesBuffer.data[tileOffset + i].index;
        if (lightIndex < 0 || lightIndex >= lightBuffer.lights.length()) break;

        visibleLightCount++;

        Light light = lightBuffer.lights[lightIndex];
        vec3 lightPos = vec3(view * vec4(light.position, 1.0));
        vec3 lightVec = lightPos - fragPos;
        float distance = length(lightVec);

        if (distance >= light.radius) continue;

        float attenuation = attenuate(lightVec, light.radius);
        totalInfluence += attenuation;

        vec3 lightDir = normalize(lightVec);

        float diff = max(dot(normal, lightDir), 0.0);
        diffuse += light.color * diff * attenuation;

        vec3 reflectDir = reflect(-lightDir, normal);
        //todo shininess should be controlled by something
        float shininess = clamp(u_specularPower, 1.0, 64.0);
        float spec = pow(max(dot(viewDir, reflectDir), 0.0), shininess);

        // Apply energy conservation scaling
        spec *= (1.0 - diff);

        //Fresnel term which makes the lighting slightly more realistic
        float fresnel = pow(1.0 - max(dot(viewDir, normal), 0.0), 5.0);
        fresnel = mix(0.1, 1.0, fresnel);
        spec *= fresnel;

        specular += light.color * spec * attenuation;

        lightCounter++;
    }

    vec3 result = (diffuse * u_diffuseColor.rgb) + specular;
    result += u_diffuseColor.rgb * 0.03;

    //4 and 5 are the normal ones
    int mode = 9;

    switch (mode) {
        case 0:
            float ratio = float(visibleLightCount) / max(float(u_lightCount), 1.0);
            fragColor = vec4(vec3(ratio), 1.0);
            break;
        case 1:
            float normalizedCount = float(visibleLightCount) / 3.0;
            fragColor = vec4(heatMapColor(normalizedCount), 1.0);
            break;
        case 2:
            fragColor = vec4(heatMapColor(min(totalInfluence / 3.0, 1.0)), 1.0);
            break;
        case 3:
            fragColor = vec4(normalize(fragment_in.normalVector) * 0.5 + 0.5, 1.0);
            break;
        case 4:
            fragColor = vec4(result, u_diffuseColor.a);
            if (length(fragment_in.normalVector) < 0.001) {
                fragColor = vec4(1.0, 0.0, 1.0, 1.0);
                return;
            }
            break;
        case 5:
            fragColor = vec4(result * u_specularPower, u_diffuseColor.a);
            break;
        case 6:
            fragColor = vec4(heatMapColor(float(lightCounter) / float(u_lightCount)), 1.0);
            break;
        case 7:
            float val = clamp(float(lightCounter) / 50.0, 0.0, 1.0);
            fragColor = vec4(val, val, val, 1.0);
            break;
        case 8:
            fragColor = vec4(sign(normal) * 0.5 + 0.5, 1.0);
            break;
        case 9:
            vec3 flatNormal = calculateFlatNormal();
            vec3 normalVis = mix(flatNormal * 0.5 + 0.5, normal * 0.5 + 0.5, 0.5);
            fragColor = vec4(normalVis, 1.0);
            break;
    }
}

    
    // result = pow(result, vec3(1.0/2.2));

    // float ndotv = dot(normal, viewDir);
    // vec3 debugColor = vec3(ndotv > 0 ? ndotv : 0, 0, ndotv < 0 ? -ndotv : 0);
    
    //fragColor = vec4(result, u_diffuseColor.a);


// #version 430 core

// in VERTEX_OUT {
//     vec3 fragmentPosition;
//     vec3 normalVector;
//     vec2 textureCoordinates;
// } fragment_in;

// struct Light {//TODO should prolly change these to like vec4 so like no just like spam casting to vec4 from vec3 
//     vec3 position;
//     float radius;
//     vec3 color;
//     float intensity;
// };

// struct VisibleIndex {
//     int index;
// };

// layout(std430, binding = 0) readonly buffer LightBuffer {
//     Light lights[];
// } lightBuffer;

// layout(std430, binding = 2) readonly buffer VisibleLightIndicesBuffer {
//     VisibleIndex data[];
// } visibleLightIndicesBuffer;

// uniform vec4 u_diffuseColor;
// uniform float u_specularPower;
// uniform int u_tileCountX;
// uniform int u_lightCount;
// uniform mat4 view;//todo big bigs
// // uniform vec3 u_cameraPosition; // Not used for now

// layout(location = 0) out vec4 fragColor;

// // float attenuate(vec3 lightDirection, float radius) {
// //     float cutoff = 0.5;
// //     float attenuation = dot(lightDirection, lightDirection) / (100.0 * radius);
// //     attenuation = 1.0 / (attenuation * 15.0 + 1.0);
// //     attenuation = (attenuation - cutoff) / (1.0 - cutoff);
// //     return clamp(attenuation, 0.0, 1.0);
// // }

// // float attenuate(vec3 lightDirection, float radius) {
// //     float distance = length(lightDirection);
// //     float normalizedDistance = distance / radius;
// //     float attenuation = 1.0 / (1.0 + 25.0 * normalizedDistance * normalizedDistance);
// //     return smoothstep(0.0, 1.0, attenuation);
// // }

// // float attenuate(vec3 lightDir, float radius) {
// //     float dist = length(lightDir);
// //     float normDist = dist / radius;
// //     return pow(1.0 - smoothstep(0.0, 1.0, normDist), 2.0);
// // }

// float attenuate(vec3 lightDirection, float radius) {
//     float distance = length(lightDirection);
//     float normalizedDist = distance / radius;
//     float attenuation = 1.0 / (1.0 + 25.0 * normalizedDist * normalizedDist);
//     return smoothstep(0.0, 1.0, attenuation);
// }

// vec3 heatMapColor(float value) {
//     const vec3 c1 = vec3(0.0, 0.0, 1.0);
//     const vec3 c2 = vec3(0.0, 1.0, 1.0);
//     const vec3 c3 = vec3(0.0, 1.0, 0.0);
//     const vec3 c4 = vec3(1.0, 1.0, 0.0);
//     const vec3 c5 = vec3(1.0, 0.0, 0.0);

//     if (value < 0.25) return mix(c1, c2, value / 0.25);
//     if (value < 0.5)  return mix(c2, c3, (value - 0.25) / 0.25);
//     if (value < 0.75) return mix(c3, c4, (value - 0.5) / 0.25);
//     return mix(c4, c5, (value - 0.75) / 0.25);
// }

// void main() {
//     ivec2 location = ivec2(gl_FragCoord.xy);
//     ivec2 tileID = location / ivec2(16, 16);
//     int tileIndex = tileID.y * u_tileCountX + tileID.x;
//     int tileOffset = tileIndex * 256;//256 hardcoded bitch dont fuck with my offset

//     vec3 totalDiffuse = vec3(0.0);
//     vec3 totalSpecular = vec3(0.0);


//     //funny ones can remove later
//     uint visibleLightCount = 0;
//     float totalInfluence = 0.0;
//     int lightCounter = 0;

//     //these are not funny doe
//     vec3 normal = normalize(fragment_in.normalVector);//got rid of pure 0 avoidance... we will see how that goes
//     vec3 fragPos = fragment_in.fragmentPosition;
//     vec3 viewDir = normalize(fragPos);//have mercy on me and work... just work dont be wrapping my normals the wrong way or nothing

//     // if (dot(normal, viewDir) < 0.0)
//     //     normal = -normal;

//     vec3 color = vec3(0.0);//final color but not actually bc i be tweaking with it

//     for (int i = 0; i < u_lightCount; ++i) {
//         int lightIndex = visibleLightIndicesBuffer.data[tileOffset + i].index;
//         if (lightIndex < 0 || lightIndex >= lightBuffer.lights.length()) break;

//         visibleLightCount++;

//         Light light = lightBuffer.lights[lightIndex];
//         vec3 lightPos = vec3(view * vec4(light.position, 1.0));
//         //vec3 lightPos = light.position.xyz;
//         vec3 lightDir = lightPos - fragPos;
//         float distance = length(lightDir);

//         if (distance > light.radius) continue;//hmm what happends if i like make it >=

//         float attenuation = attenuate(lightDir, light.radius);
//         totalInfluence += attenuation;

//         lightDir = normalize(lightDir);

//         float diff = max(dot(normal, lightDir), 0.0);
//         vec3 halfway = normalize(lightDir + viewDir);
//         float spec = pow(max(dot(normal, halfway), 0.0), u_specularPower);

//         vec3 lightDiffuse = light.color.rgb * u_diffuseColor.rgb * diff * attenuation;
//         vec3 lightSpecular = light.color.rgb * spec * attenuation;

//         if (diff == 0.0) spec = 0.0;

//         vec3 irradiance = light.color.rgb * ((u_diffuseColor.rgb * diff) + vec3(spec)) * attenuation;
//         color += irradiance;
//         lightCounter++;
//     }

//     color += u_diffuseColor.rgb * 0.08;//I should not HARD CODE THIS!!!! IT SHOULD BE A... LIKE... Interpolation of all the lighst in the scenenenenene TODO

//     int mode = 4;
    
//     // if (location.x % 16 == 0 || location.y % 16 == 0) {
//     //     fragColor = vec4(0.3, 0.3, 0.3, 1.0);
//     //     return;
//     // }


//     switch (mode) {
//         case 0:
//             // Raw count of visible lights (grayscale)
//             float ratio = float(visibleLightCount) / max(float(u_lightCount), 1.0);
//             fragColor = vec4(vec3(ratio), 1.0);
//             break;
            
//         case 1:
//             // Heatmap visualization of light count
//             float normalizedCount = float(visibleLightCount) / 3.0; // Adjust based on expected max lights per tile
//             fragColor = vec4(heatMapColor(normalizedCount), 1.0);
//             break;
            
//         case 2:
//             // Light influence visualization
//             fragColor = vec4(heatMapColor(min(totalInfluence / 3.0, 1.0)), 1.0);
//             break;
            
//         case 3:
//             // Normal visualization (helps validate geometry)
//             fragColor = vec4(normalize(fragment_in.normalVector) * 0.5 + 0.5, 1.0);
//             //fragColor = vec4(normal * 0.5 + 0.5, 1.0);
//             //fragColor = vec4(normalize(normal) * 0.5 + 0.5, 1.0);
//             break;
//         case 4:
//             fragColor = vec4(color, u_diffuseColor.a);
//             if (length(fragment_in.normalVector) < 0.001) {
//                 fragColor = vec4(1.0, 0.0, 1.0, 1.0); // magenta = BAD normal
//                 return;
//             }
//             break;
//         case 5:
//             fragColor = vec4(color * u_specularPower, u_diffuseColor.a);
//             break;
//         case 6:
//             fragColor = vec4(heatMapColor(float(lightCounter) / float(u_lightCount)), 1.0);
//             break;
//         case 7:
//             float val = clamp(float(lightCounter) / 50.0, 0.0, 1.0);
//             fragColor = vec4(val, val, val, 1.0);
//             break;
//         case 8:
//             fragColor = vec4(sign(normal) * 0.5 + 0.5, 1.0);
//             break;
//     }



//     //fragColor = vec4(finalColor * (u_specularPower * 0.1), u_diffuseColor.a);
// }