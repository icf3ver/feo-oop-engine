#version 450

layout(input_attachment_index = 0, set = 0, binding = 0) uniform subpassInput u_depth_light_hashed;

layout(push_constant) uniform PushConstants {
    mat4 screen_to_light; // (world_screen^-1 -> screen_world) * world_light
} push_constants;

layout(location=0) out float depth_cam_stencil;

void main() { // NOT DONE still a bit to do
    vec3 in_depth = subpassLoad(u_depth_light_hashed);
    depth_cam_filtered[in_depth.y][in_depth.z] = in_depth[in_depth.x]; // y, z, and x are x_u_buffer, y_u_buffer, and depth for camera respectively
}