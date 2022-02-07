#version 450

layout(input_attachment_index = 0, set = 0, binding = 0) uniform subpassInput u_normals;
layout(input_attachment_index = 1, set = 0, binding = 1) uniform subpassInput u_diffuse;

layout(push_constant) uniform PushConstants {
    // The color of the directional light
    vec4 color;
    // The direction in which the directional light is shining
    vec4 direction;
} push_constants;

layout(location = 0) out vec4 f_color;

void main() {
    vec3 in_normal = normalize(subpassLoad(u_normals).rgb);
    float light_percent = -dot(normalize(push_constants.direction.xyz), in_normal);
    light_percent = max(light_percent, 0.0);

    vec3 in_diffuse = subpassLoad(u_diffuse).rgb;
    f_color = vec4(light_percent * push_constants.color.rgb * in_diffuse, 1.0);
}