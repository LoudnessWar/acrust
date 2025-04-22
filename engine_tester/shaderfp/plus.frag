#version 430

in VERTEX_OUT{
	vec3 fragmentPosition;
	vec2 textureCoordinates;
	mat3 TBN;
	vec3 tangentViewPosition;
	vec3 tangentFragmentPosition;
} fragment_in;

struct Light {
    vec3 position;
    float radius;
    vec3 color;
    float intensity;
};

struct VisibleIndex {
	int index;
};

layout(std430, binding = 0) buffer LightBuffer {
    Light lights[];
} lightBuffer;

layout(std430, binding = 2) writeonly buffer VisibleLightIndicesBuffer {
    VisibleIndex data[];
} visibleLightIndicesBuffer;

uniform vec4 u_diffuseColor;
uniform float u_specularPower;
uniform int u_tileCountX;
uniform int u_lightCount;

out vec4 fragColor;

float attenuate(vec3 lightDirection, float radius) {
	float cutoff = 0.5;
	float attenuation = dot(lightDirection, lightDirection) / (100.0 * radius);
	attenuation = 1.0 / (attenuation * 15.0 + 1.0);
	attenuation = (attenuation - cutoff) / (1.0 - cutoff);
	return clamp(attenuation, 0.0, 1.0);
}

void main() {
	ivec2 location = ivec2(gl_FragCoord.xy);
	ivec2 tileID = location / ivec2(16, 16);
	uint index = tileID.y * u_tileCountX + tileID.x;

	vec4 base_diffuse = u_diffuseColor;//vec4(1.0);
	vec4 base_specular = vec4(1.0);

	vec3 normal = vec3(0.0, 0.0, 1.0);
	vec3 viewDirection = normalize(fragment_in.tangentViewPosition - fragment_in.tangentFragmentPosition);
	vec4 color = vec4(0.0);

	uint offset = index * 256;
	for (uint i = 0; i < 256 && visibleLightIndicesBuffer.lights[offset + i].index != -1; i++) {
		uint lightIndex = visibleLightIndicesBuffer.lights[offset + i].index;
		PointLight light = lightBuffer.data[lightIndex];

		vec4 lightColor = light.color;
		vec3 tangentLightPosition = fragment_in.TBN * light.position.xyz;
		float lightRadius = light.paddingAndRadius.w;

		vec3 lightDirection = tangentLightPosition - fragment_in.tangentFragmentPosition;
		float attenuation = attenuate(lightDirection, lightRadius);

		lightDirection = normalize(lightDirection);
		vec3 halfway = normalize(lightDirection + viewDirection);

		float diffuse = max(dot(lightDirection, normal), 0.0);
		float specular = pow(max(dot(normal, halfway), 0.0), 32.0);
		if (diffuse == 0.0) {
			specular = u_specularPower;
		}

		vec3 irradiance = lightColor.rgb * ((base_diffuse.rgb * diffuse) + (base_specular.rgb * vec3(specular))) * attenuation;
		color.rgb += irradiance;
	}

	color.rgb += base_diffuse.rgb * (0.08 * u_lightCount);
	fragColor = color;
}


// #version 330 core

// layout (location = 0) in vec3 position;
// layout (location = 1) in vec3 normal;
// layout (location = 2) in vec2 texCoords;
// layout (location = 3) in vec3 tangent;
// layout (location = 4) in vec3 bitangent;

// out VERTEX_OUT {
// 	vec3 fragmentPosition;
// 	vec2 textureCoordinates;
// 	mat3 TBN;
// 	vec3 tangentViewPosition;
// 	vec3 tangentFragmentPosition;
// } vertex_out;

// uniform mat4 projection;
// uniform mat4 view;
// uniform mat4 model;
// uniform vec3 viewPosition;

// void main() {
// 	gl_Position = projection * view * model * vec4(position, 1.0);
// 	vertex_out.fragmentPosition = vec3(model * vec4(position, 1.0));
// 	vertex_out.textureCoordinates = texCoords;

// 	mat3 normalMatrix = transpose(inverse(mat3(model)));
// 	vec3 T = normalize(normalMatrix * tangent);
// 	vec3 B = normalize(normalMatrix * bitangent);
// 	vec3 N = normalize(normalMatrix * normal);

// 	mat3 TBN = mat3(T, B, N);
// 	vertex_out.TBN = TBN;
// 	vertex_out.tangentViewPosition = TBN * viewPosition;
// 	vertex_out.tangentFragmentPosition = TBN * vertex_out.fragmentPosition;
// }


// #version 430 core

// in VERTEX_OUT {
//     vec3 fragmentPosition;
//     vec3 normalVector;
//     vec2 textureCoordinates;
//     mat3 TBN;  // Added for tangent-space lighting
// } fragment_in;

// struct Light {
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
// uniform mat4 view;
// uniform vec3 u_cameraPosition;  // Now used for proper view direction

// layout(location = 0) out vec4 fragColor;

// float attenuate(vec3 lightDirection, float radius) {
//     float distance = length(lightDirection);
//     float normalizedDist = distance / radius;
//     float attenuation = 1.0 / (1.0 + 25.0 * normalizedDist * normalizedDist);
//     return smoothstep(0.0, 1.0, attenuation);
// }

// void main() {
//     ivec2 location = ivec2(gl_FragCoord.xy);
//     ivec2 tileID = location / ivec2(16, 16);
//     int tileIndex = tileID.y * u_tileCountX + tileID.x;
//     int tileOffset = tileIndex * 256;

//     // Lighting accumulators
//     vec3 totalDiffuse = vec3(0.0);
//     vec3 totalSpecular = vec3(0.0);

//     // Surface properties
//     vec3 normal = normalize(fragment_in.normalVector);
//     vec3 fragPos = fragment_in.fragmentPosition;
//     vec3 viewDir = normalize(u_cameraPosition - fragPos);  // Fixed view direction

//     // Light processing
//     for (int i = 0; i < u_lightCount; ++i) {
//         int lightIndex = visibleLightIndicesBuffer.data[tileOffset + i].index;
//         if (lightIndex < 0 || lightIndex >= lightBuffer.lights.length()) break;

//         Light light = lightBuffer.lights[lightIndex];
//         vec3 lightPos = vec3(view * vec4(light.position, 1.0));
//         vec3 lightDir = lightPos - fragPos;
//         float distance = length(lightDir);

//         if (distance > light.radius) continue;

//         float attenuation = attenuate(lightDir, light.radius);
//         lightDir = normalize(lightDir);

//         // Diffuse component
//         float diff = max(dot(normal, lightDir), 0.0);
//         vec3 diffuse = light.color * u_diffuseColor.rgb * diff * attenuation;

//         // Specular component (Blinn-Phong)
//         vec3 halfwayDir = normalize(lightDir + viewDir);
//         float spec = pow(max(dot(normal, halfwayDir), 0.0), u_specularPower);
//         vec3 specular = light.color * spec * attenuation;

//         // Energy conservation - only apply specular if light hits surface
//         specular *= step(0.001, diff);

//         totalDiffuse += diffuse;
//         totalSpecular += specular;
//     }

//     // Combine with ambient
//     vec3 ambient = u_diffuseColor.rgb * 0.08;
//     vec3 color = ambient + totalDiffuse + totalSpecular;

//     // Debug modes (unchanged from your original)
//     int mode = 4;
//     switch (mode) {
//         case 0: /* ... */ break;
//         case 1: /* ... */ break;
//         case 2: /* ... */ break;
//         case 3: /* ... */ break;
//         case 4:
//             fragColor = vec4(color, u_diffuseColor.a);
//             if (length(fragment_in.normalVector) < 0.001) {
//                 fragColor = vec4(1.0, 0.0, 1.0, 1.0);
//             }
//             break;
//         case 5: /* ... */ break;
//         case 6: /* ... */ break;
//         case 7: /* ... */ break;
//         case 8: /* ... */ break;
//     }
// }



// // #version 430 core

// // in VERTEX_OUT {
// //     vec3 fragmentPosition;
// //     vec3 normalVector;
// //     vec2 textureCoordinates;
// // } fragment_in;

// // struct Light {//TODO should prolly change these to like vec4 so like no just like spam casting to vec4 from vec3 
// //     vec3 position;
// //     float radius;
// //     vec3 color;
// //     float intensity;
// // };

// // struct VisibleIndex {
// //     int index;
// // };

// // layout(std430, binding = 0) readonly buffer LightBuffer {
// //     Light lights[];
// // } lightBuffer;

// // layout(std430, binding = 2) readonly buffer VisibleLightIndicesBuffer {
// //     VisibleIndex data[];
// // } visibleLightIndicesBuffer;

// // uniform vec4 u_diffuseColor;
// // uniform float u_specularPower;
// // uniform int u_tileCountX;
// // uniform int u_lightCount;
// // uniform mat4 view;//todo big bigs
// // // uniform vec3 u_cameraPosition; // Not used for now

// // layout(location = 0) out vec4 fragColor;

// // // float attenuate(vec3 lightDirection, float radius) {
// // //     float cutoff = 0.5;
// // //     float attenuation = dot(lightDirection, lightDirection) / (100.0 * radius);
// // //     attenuation = 1.0 / (attenuation * 15.0 + 1.0);
// // //     attenuation = (attenuation - cutoff) / (1.0 - cutoff);
// // //     return clamp(attenuation, 0.0, 1.0);
// // // }

// // // float attenuate(vec3 lightDirection, float radius) {
// // //     float distance = length(lightDirection);
// // //     float normalizedDistance = distance / radius;
// // //     float attenuation = 1.0 / (1.0 + 25.0 * normalizedDistance * normalizedDistance);
// // //     return smoothstep(0.0, 1.0, attenuation);
// // // }

// // // float attenuate(vec3 lightDir, float radius) {
// // //     float dist = length(lightDir);
// // //     float normDist = dist / radius;
// // //     return pow(1.0 - smoothstep(0.0, 1.0, normDist), 2.0);
// // // }

// // float attenuate(vec3 lightDirection, float radius) {
// //     float distance = length(lightDirection);
// //     float normalizedDist = distance / radius;
// //     float attenuation = 1.0 / (1.0 + 25.0 * normalizedDist * normalizedDist);
// //     return smoothstep(0.0, 1.0, attenuation);
// // }

// // vec3 heatMapColor(float value) {
// //     const vec3 c1 = vec3(0.0, 0.0, 1.0);
// //     const vec3 c2 = vec3(0.0, 1.0, 1.0);
// //     const vec3 c3 = vec3(0.0, 1.0, 0.0);
// //     const vec3 c4 = vec3(1.0, 1.0, 0.0);
// //     const vec3 c5 = vec3(1.0, 0.0, 0.0);

// //     if (value < 0.25) return mix(c1, c2, value / 0.25);
// //     if (value < 0.5)  return mix(c2, c3, (value - 0.25) / 0.25);
// //     if (value < 0.75) return mix(c3, c4, (value - 0.5) / 0.25);
// //     return mix(c4, c5, (value - 0.75) / 0.25);
// // }

// // void main() {
// //     ivec2 location = ivec2(gl_FragCoord.xy);
// //     ivec2 tileID = location / ivec2(16, 16);
// //     int tileIndex = tileID.y * u_tileCountX + tileID.x;
// //     int tileOffset = tileIndex * 256;//256 hardcoded bitch dont fuck with my offset

// //     vec3 totalDiffuse = vec3(0.0);
// //     vec3 totalSpecular = vec3(0.0);


// //     //funny ones can remove later
// //     uint visibleLightCount = 0;
// //     float totalInfluence = 0.0;
// //     int lightCounter = 0;

// //     //these are not funny doe
// //     vec3 normal = normalize(fragment_in.normalVector);//got rid of pure 0 avoidance... we will see how that goes
// //     vec3 fragPos = fragment_in.fragmentPosition;
// //     vec3 viewDir = normalize(fragPos);//have mercy on me and work... just work dont be wrapping my normals the wrong way or nothing

// //     // if (dot(normal, viewDir) < 0.0)
// //     //     normal = -normal;

// //     vec3 color = vec3(0.0);//final color but not actually bc i be tweaking with it

// //     for (int i = 0; i < u_lightCount; ++i) {
// //         int lightIndex = visibleLightIndicesBuffer.data[tileOffset + i].index;
// //         if (lightIndex < 0 || lightIndex >= lightBuffer.lights.length()) break;

// //         visibleLightCount++;

// //         Light light = lightBuffer.lights[lightIndex];
// //         vec3 lightPos = vec3(view * vec4(light.position, 1.0));
// //         //vec3 lightPos = light.position.xyz;
// //         vec3 lightDir = lightPos - fragPos;
// //         float distance = length(lightDir);

// //         if (distance > light.radius) continue;//hmm what happends if i like make it >=

// //         float attenuation = attenuate(lightDir, light.radius);
// //         totalInfluence += attenuation;

// //         lightDir = normalize(lightDir);

// //         float diff = max(dot(normal, lightDir), 0.0);
// //         vec3 halfway = normalize(lightDir + viewDir);
// //         float spec = pow(max(dot(normal, halfway), 0.0), u_specularPower);

// //         vec3 lightDiffuse = light.color.rgb * u_diffuseColor.rgb * diff * attenuation;
// //         vec3 lightSpecular = light.color.rgb * spec * attenuation;

// //         if (diff == 0.0) spec = 0.0;

// //         vec3 irradiance = light.color.rgb * ((u_diffuseColor.rgb * diff) + vec3(spec)) * attenuation;
// //         color += irradiance;
// //         lightCounter++;
// //     }

// //     color += u_diffuseColor.rgb * 0.08;//I should not HARD CODE THIS!!!! IT SHOULD BE A... LIKE... Interpolation of all the lighst in the scenenenenene TODO

// //     int mode = 4;
    
// //     // if (location.x % 16 == 0 || location.y % 16 == 0) {
// //     //     fragColor = vec4(0.3, 0.3, 0.3, 1.0);
// //     //     return;
// //     // }


// //     switch (mode) {
// //         case 0:
// //             // Raw count of visible lights (grayscale)
// //             float ratio = float(visibleLightCount) / max(float(u_lightCount), 1.0);
// //             fragColor = vec4(vec3(ratio), 1.0);
// //             break;
            
// //         case 1:
// //             // Heatmap visualization of light count
// //             float normalizedCount = float(visibleLightCount) / 3.0; // Adjust based on expected max lights per tile
// //             fragColor = vec4(heatMapColor(normalizedCount), 1.0);
// //             break;
            
// //         case 2:
// //             // Light influence visualization
// //             fragColor = vec4(heatMapColor(min(totalInfluence / 3.0, 1.0)), 1.0);
// //             break;
            
// //         case 3:
// //             // Normal visualization (helps validate geometry)
// //             fragColor = vec4(normalize(fragment_in.normalVector) * 0.5 + 0.5, 1.0);
// //             //fragColor = vec4(normal * 0.5 + 0.5, 1.0);
// //             //fragColor = vec4(normalize(normal) * 0.5 + 0.5, 1.0);
// //             break;
// //         case 4:
// //             fragColor = vec4(color, u_diffuseColor.a);
// //             if (length(fragment_in.normalVector) < 0.001) {
// //                 fragColor = vec4(1.0, 0.0, 1.0, 1.0); // magenta = BAD normal
// //                 return;
// //             }
// //             break;
// //         case 5:
// //             fragColor = vec4(color * u_specularPower, u_diffuseColor.a);
// //             break;
// //         case 6:
// //             fragColor = vec4(heatMapColor(float(lightCounter) / float(u_lightCount)), 1.0);
// //             break;
// //         case 7:
// //             float val = clamp(float(lightCounter) / 50.0, 0.0, 1.0);
// //             fragColor = vec4(val, val, val, 1.0);
// //             break;
// //         case 8:
// //             fragColor = vec4(sign(normal) * 0.5 + 0.5, 1.0);
// //             break;
// //     }



// //     //fragColor = vec4(finalColor * (u_specularPower * 0.1), u_diffuseColor.a);
// // }

// // #version 430 core

// // in VERTEX_OUT {
// //     vec3 fragmentPosition;  // Now in view space
// //     vec3 normalVector;      // Now in view space
// //     vec2 textureCoordinates;
// // } fragment_in;

// // // Light structure (keep your existing)
// // struct Light {
// //     vec3 position;
// //     float radius;
// //     vec3 color;
// //     float intensity;
// // };

// // struct VisibleIndex {
// //     int index;
// // };

// // // Buffer definitions (keep your existing)
// // layout(std430, binding = 0) readonly buffer LightBuffer {
// //     Light lights[];
// // } lightBuffer;

// // layout(std430, binding = 2) readonly buffer VisibleLightIndicesBuffer {
// //     VisibleIndex data[];
// // } visibleLightIndicesBuffer;

// // // Uniforms
// // uniform vec4 u_diffuseColor;
// // uniform float u_specularPower;
// // uniform int u_tileCountX;
// // uniform int u_lightCount;
// // uniform mat4 view;

// // out vec4 fragColor;

// // // Improved attenuation function
// // float attenuate(vec3 lightDirection, float radius) {
// //     float distance = length(lightDirection);
// //     float normalizedDist = distance / radius;
// //     float attenuation = 1.0 / (1.0 + 25.0 * normalizedDist * normalizedDist);
// //     return smoothstep(0.0, 1.0, attenuation);
// // }

// // void main() {
// //     ivec2 location = ivec2(gl_FragCoord.xy);
// //     ivec2 tileID = location / ivec2(16, 16);
// //     int tileIndex = tileID.y * u_tileCountX + tileID.x;
// //     int tileOffset = tileIndex * 256;

// //     // Normalize the view-space normal
// //     vec3 normal = normalize(fragment_in.normalVector);
    
// //     // View vector points toward the camera (0,0,0 in view space)
// //     vec3 viewDir = normalize(-fragment_in.fragmentPosition);

// //     // if (dot(normal, viewDir) < 0.0) normal = -normal;
    
// //     // Lighting accumulators
// //     vec3 diffuse = vec3(0.0);
// //     vec3 specular = vec3(0.0);

// //     for (int i = 0; i < u_lightCount; ++i) {
// //         int lightIndex = visibleLightIndicesBuffer.data[tileOffset + i].index;
// //         if (lightIndex < 0 || lightIndex >= lightBuffer.lights.length()) break;

// //         Light light = lightBuffer.lights[lightIndex];
        
// //         // Transform light to view space
// //         vec3 lightPos = vec3(view * vec4(light.position, 1.0));
// //         vec3 lightDir = lightPos - fragment_in.fragmentPosition;
// //         float distance = length(lightDir);
        
// //         if (distance > light.radius) continue;
        
// //         lightDir = normalize(lightDir);
// //         float attenuation = attenuate(lightDir, light.radius);
        
// //         // Diffuse
// //         float diff = max(dot(normal, lightDir), 0.0);
// //         diffuse += light.color * diff * attenuation;
        
// //         // Specular (Blinn-Phong)
// //         vec3 halfwayDir = normalize(lightDir + viewDir);
// //         float spec = pow(max(dot(normal, halfwayDir), 0.0), u_specularPower);
// //         specular += light.color * spec * attenuation;
// //     }

// //     // Combine lighting
// //     vec3 result = (diffuse * u_diffuseColor.rgb) + specular;
// //     result += u_diffuseColor.rgb * 0.03;  // Ambient

// //     result = pow(result, vec3(1.0/2.2));
    
// //     fragColor = vec4(result, u_diffuseColor.a);
// // }

// // #version 430 core

// // in VERTEX_OUT {
// //     vec3 fragmentPosition;  // Now in view space
// //     vec3 normalVector;      // Now in view space
// //     vec2 textureCoordinates;
// // } fragment_in;

// // // Light structure (keep your existing)
// // struct Light {
// //     vec3 position;
// //     float radius;
// //     vec3 color;
// //     float intensity;
// // };

// // struct VisibleIndex {
// //     int index;
// // };

// // // Buffer definitions (keep your existing)
// // layout(std430, binding = 0) readonly buffer LightBuffer {
// //     Light lights[];
// // } lightBuffer;

// // layout(std430, binding = 2) readonly buffer VisibleLightIndicesBuffer {
// //     VisibleIndex data[];
// // } visibleLightIndicesBuffer;

// // // Uniforms
// // uniform vec4 u_diffuseColor;
// // uniform float u_specularPower;
// // uniform int u_tileCountX;
// // uniform int u_lightCount;
// // uniform mat4 view;

// // out vec4 fragColor;

// // // Improved attenuation function
// // float attenuate(vec3 lightDirection, float radius) {
// //     float distance = length(lightDirection);
// //     float normalizedDist = distance / radius;
// //     float attenuation = 1.0 / (1.0 + 25.0 * normalizedDist * normalizedDist);
// //     return smoothstep(0.0, 1.0, attenuation);
// // }

// // void main() {
// //     ivec2 location = ivec2(gl_FragCoord.xy);
// //     ivec2 tileID = location / ivec2(16, 16);
// //     int tileIndex = tileID.y * u_tileCountX + tileID.x;
// //     int tileOffset = tileIndex * 256;

// //     // Normalize the view-space normal
// //     vec3 normal = normalize(fragment_in.normalVector);
    
// //     // View vector points toward the camera (0,0,0 in view space)
// //     vec3 viewDir = normalize(-fragment_in.fragmentPosition);

// //     // if (dot(normal, viewDir) < 0.0) normal = -normal;
    
// //     // Lighting accumulators
// //     vec3 diffuse = vec3(0.0);
// //     vec3 specular = vec3(0.0);

// //     for (int i = 0; i < u_lightCount; ++i) {
// //         int lightIndex = visibleLightIndicesBuffer.data[tileOffset + i].index;
// //         if (lightIndex < 0 || lightIndex >= lightBuffer.lights.length()) break;

// //         Light light = lightBuffer.lights[lightIndex];
        
// //         // Transform light to view space
// //         vec3 lightPos = vec3(view * vec4(light.position, 1.0));
// //         vec3 lightDir = lightPos - fragment_in.fragmentPosition;
// //         float distance = length(lightDir);
        
// //         if (distance > light.radius) continue;
        
// //         lightDir = normalize(lightDir);
// //         float attenuation = attenuate(lightDir, light.radius);
        
// //         // Diffuse
// //         float diff = max(dot(normal, lightDir), 0.0);
// //         diffuse += light.color * diff * attenuation;
        
// //         // Specular (Blinn-Phong)
// //         vec3 halfwayDir = normalize(lightDir + viewDir);
// //         float spec = pow(max(dot(normal, halfwayDir), 0.0), u_specularPower);
// //         specular += light.color * spec * attenuation;
// //     }

// //     // Combine lighting
// //     vec3 result = (diffuse * u_diffuseColor.rgb) + specular;
// //     result += u_diffuseColor.rgb * 0.03;  // Ambient

// //     result = pow(result, vec3(1.0/2.2));
    
// //     fragColor = vec4(result, u_diffuseColor.a);
// // }