#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec2 texture_index;

layout(set = 0, binding = 0) uniform Camera { // TODO PushConstants
    mat4 to_view;  // ident space to view space
    mat4 view_to_screen;  // view space to screen space
} u_camera; // TODO: push_constants

layout(set = 1, binding = 0) buffer World {
    mat4 object_to; // object space to ident space
} u_world;

layout(location = 0) out vec3 v_normal;
layout(location = 1) out vec2 v_texture_index;
layout(location = 2) out vec2 v_camspace_xy;

void main() {
    // Texture index 
    v_texture_index = vec2(0, 1) + vec2(1, -1) * texture_index;

    // object space to view space projection matrix // pre-multiplication
    mat4 object_to_view = u_camera.to_view /* ident space to view space */ * u_world.object_to /* object space to ident space */; 
    // rotation part of object space to view space projection
    // cut out the translation
    // inverse(scale) and cancel out effect of transpose on rotation
    mat3 normal_to_view_rot_scl = inverse(transpose(mat3(object_to_view)));
    // object space to screen space projection matrix
    vec4 object_in_camspace = object_to_view * vec4(position, 1.0);

    v_normal = normalize(normal_to_view_rot_scl * normal);
    v_camspace_xy = object_in_camspace.xy;
    gl_Position = u_camera.view_to_screen * object_in_camspace;
}