#version 330 core

uniform float near;
uniform float far;

out vec4 fragColor;

// Need to linearize the depth because we are using the projection
//this is like basically just using view and like project matrix to like
//like we could just use them for this because near and far should well are lowkey attainable from them so its like over or whatever
float LinearizeDepth(float depth) {
	float z = depth * 2.0 - 1.0;
	return (2.0 * near * far) / (far + near - z * (far - near));
}

void main() {
	float depth = LinearizeDepth(gl_FragCoord.z) / far;
    //depth = 1.0 - depth;
	fragColor = vec4(vec3(depth), 1.0f);
}

// #version 450
// layout(location = 0) out vec4 FragColor;

// void main() {
//     float z = gl_FragCoord.z;
//     float flipped_z = 1.0 - z;         // now 0 = near, 1 = far
//     FragColor = vec4(flipped_z, flipped_z, flipped_z, 1.0);

// }