#version 150

uniform sampler2D vane_texture;

in vec3 normal_f;
in vec2 texture_f;
out vec4 color;

const vec3 light_source = normalize(vec3(-0.2, 1.0, 1.0));

void main() {
    float brightness = 0.1 + 0.6 * clamp(dot(light_source, normal_f), 0.0, 1.0);
    vec3 gray = vec3(brightness);
    color = vec4(texture(vane_texture, texture_f).rgb * brightness, 1.0);
}
