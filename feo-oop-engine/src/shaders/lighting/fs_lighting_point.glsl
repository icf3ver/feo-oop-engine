#version 450

// This feature is not complete. I discovered through this feature that the depth buffer is not working properly
// 

layout(input_attachment_index = 0, set = 0, binding = 0) uniform subpassInput u_normals;
layout(input_attachment_index = 1, set = 0, binding = 1) uniform subpassInput u_diffuse;
layout(input_attachment_index = 3, set = 0, binding = 2) uniform subpassInput u_specular;
layout(input_attachment_index = 4, set = 0, binding = 3) uniform subpassInput u_depth;

layout(push_constant) uniform PushConstants {
    // Screen space to camera space projection matrix
    mat4 screen_to_camera;
    // The point lights color
    vec4 color;
    // The position of the light in camera space
    vec4 position_camera_space;
} push_constants;

// layout(location = 0) in vec2 v_screen_coords;

layout(location = 0) out vec4 f_color;

void main() {
    float in_depth = subpassLoad(u_depth).z;

    // Only render what exists/is visible
    if (in_depth >= 1.0) {
        discard;
    }

    // depth = -<z> (n + f) / (f - n) - 2fn / (f - n)
    // depth (f - n) = -<z> (n + f) - 2fn
    // depth (f - n) + 2fn = -<z> (n + f)
    // (depth (f - n) + 2fn) / (n + f) = -<z> 
    // (depth (f - n) / (n + f)) + (2fn / (n + f)) = -<z>

    // post thought
    
    // LET M = Matrix4::new(
    //     [ self.near_plane / half_w,                       0.0,                                                                      0.0,                                                                            0.0],
    //     [                      0.0, self.near_plane / -half_h,                                                                      0.0,                                                                            0.0],
    //     [                      0.0,                       0.0, -(self.near_plane + self.far_plane) / (self.far_plane - self.near_plane), (-2.0 * self.far_plane * self.near_plane) / (self.far_plane - self.near_plane)],
    //     [                      0.0,                       0.0,                                                                     -1.0,                                                                            0.0]
    // ).inverse() =
    // Matrix4::new(
    //     [ half_w / self.near_plane,                       0.0,                                                                            0.0,                                                                           0.0],
    //     [                      0.0, -half_h / self.near_plane,                                                                            0.0,                                                                           0.0],
    //     [                      0.0,                       0.0,                                                                            0.0,                                                                          -1.0],
    //     [                      0.0,                       0.0, -(self.far_plane - self.near_plane) / (2.0 * self.far_plane * self.near_plane), (self.far_plane + self.near_plane) / (2.0 * self.far_plane * self.near_plane)]
    // ) 
    
    // LET R_0 = 1 / M[3][3] = 2fn / (n + f)
    // LET R_1 = -(-(f - n) / 2fn) * (2fn / (n + f)) = -M[3][2] * R_0 = (f - n) / (n + f)

    //  depth * R_1 + R_0 = -<z> = <w> 

    // Reminder
    // (x', y', z', w') -> (x',y', z', _1_) / w' = gl_FragCoord in fs_draw
    // z' -> z_buffer by default
    
    // Trace back screen coord to camera space
    float R_0 = 1.0 / push_constants.screen_to_camera[3][3];
    float R_1 = -push_constants.screen_to_camera[2][3] * R_0;

    float w = in_depth * R_1 + R_0;
    vec4 texel_camera_space = push_constants.screen_to_camera * vec4(gl_FragCoord.x * w, gl_FragCoord.y * w, in_depth * w, w);
    
    vec3 in_normal = normalize(subpassLoad(u_normals).xyz);
    vec3 texel_to_light = push_constants.position_camera_space.xyz - texel_camera_space.xyz;
    float light_distance_to_texel = length(texel_to_light);
    vec3 unit_texel_to_light = normalize(texel_to_light);

    // Calculate Specular reflectance
    vec4 in_specular_all = subpassLoad(u_specular).rgba;
    vec3 specular_color = in_specular_all.rgb;
    float shine = in_specular_all.a; // NOTE     \/\/                 \/
    float specular_intensity = pow(dot(normalize(unit_texel_to_light - texel_camera_space.xyz), in_normal), shine);
    vec3 specular = specular_color * specular_intensity;

    // Calculate the the inverse of the spread of photons over the texel 
    float light_concentration = max(dot(unit_texel_to_light, in_normal), 0.0);

    // Take into account the distance of the light source
    light_concentration /= light_distance_to_texel * light_distance_to_texel;

    // Take into account the distance of the light source
    // wrong because surface becomes smaller at same rate to the eye meaning that the
    // same amount of photons hit the retna per U^2 on the inside of the eye.
    float texel_distance = length(texel_camera_space.xyz);
    light_concentration /= texel_distance * texel_distance;

    // TO REMOVE
    // Take into account the distance of the light source 
    float light_distance = length(push_constants.position_camera_space.xyz);
    light_concentration /= light_distance * light_distance;

    vec3 in_diffuse = subpassLoad(u_diffuse).rgb;

    vec3 light_color = push_constants.color.rgb * light_concentration;

    f_color.rgb = light_color * in_diffuse; // specular
    f_color.a = 1.0;
    
    // if (texel_camera_space.x > 0.0) { // ERR: depth buffer is filled with zeros
    //     f_color.rgb = vec3(100.0*in_depth, 0.0, 0.0);
    // }
}