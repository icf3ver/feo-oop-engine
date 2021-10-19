#version 450

layout(input_attachment_index = 0, set = 0, binding = 0) uniform subpassInput u_depth;

layout(push_constant) uniform PushConstants {
    mat4 screen_to_light; // (world_screen^-1 -> screen_world) * world_light
} push_constants;

layout(location=0) out float depth_light_hashed;

void main() {
    vec4 in_depth = vec4(subpassLoad(u_depth).xyz, 1.0);
    vec4 pos = push_constants.screen_to_light * in_depth;

    vec3 old = depth_light_hashed[int(pos.x)][int(pos.y)]; // fmt (-z/w) #x #y // x and y in u_depth
    if (old || old[0] > pos.w){ // greater -z = smaller z = closer
        depth_light_hashed[pos.x][pos.y] = vec3(pos.w, in_depth.x, in_depth.y);
    }
}