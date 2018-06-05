#version 150

in vec3 position;
in vec3 normal;
in vec2 texture;

out vec3 normal_f;
out vec2 texture_f;

const float camera_z = 4.0;
const float plane_z = 1.0;
const float plane_d = camera_z - plane_z;

void main() {
    float shrinkage = plane_d / (camera_z - position.z);
    gl_Position = vec4(position.xy * shrinkage, (position.z + 1.0) / 4.0, 1.0);
    normal_f = normal;
    texture_f = texture;
}
