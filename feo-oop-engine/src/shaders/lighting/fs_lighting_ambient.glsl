#version 450
layout(input_attachment_index = 1, set = 0, binding = 0) uniform subpassInput u_diffuse;
layout(input_attachment_index = 2, set = 0, binding = 1) uniform subpassInput u_ambient;
layout(input_attachment_index = 4, set = 0, binding = 2) uniform subpassInput u_depth;

layout(push_constant) uniform PushConstants {
    // The color of the ambient light
    vec4 color;
} push_constants;

layout(location = 0) out vec4 f_color;

void main() { 
    float in_depth = subpassLoad(u_depth).x;
    
    if (in_depth >= 1.0) {
        discard;
    }

    // Load in the pixel color
    vec3 in_diffuse = subpassLoad(u_diffuse).rgb;
    vec3 in_ambient = subpassLoad(u_ambient).rgb;

    // output
    f_color.rgb = push_constants.color.rgb * in_ambient * in_diffuse;
    

    // if ( in_depth == 0.0 ) {
    //     f_color.rgb = vec3(1.0, 0.0, 0.0);
    // } else {
    //     f_color.rgb = vec3(0.0, 1.0, 0.0);//vec3(in_depth > 1.0 ? 1.0 : in_depth);
    // }

    f_color.a = 1.0;
}
