#version 150

in vec3 normal_f;
out vec4 color;

const vec3 light_source = normalize(vec3(-0.2, 1.0, 1.0));

void main() {
    float brightness = 0.1 + 0.6 * clamp(dot(light_source, normal_f), 0.0, 1.0);
    color = vec4(brightness, brightness, brightness, 1.0);
}
