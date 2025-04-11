#version 450
layout(location = 0) out vec4 FragColor;

void main() {
    float z = gl_FragCoord.z;
    float flipped_z = 1.0 - z;         // now 0 = near, 1 = far
    FragColor = vec4(flipped_z, flipped_z, flipped_z, 1.0);

}