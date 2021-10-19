#version 450

// TODO split into texture and one color 

layout(location = 0) in vec3 v_normal;
layout(location = 1) in vec2 v_texture_index; 
layout(location = 2) in vec2 v_camspace_xy;

layout(set = 2, binding = 1) uniform sampler2D u_map_diffuse;
layout(set = 2, binding = 2) uniform sampler2D u_map_ambient;
layout(set = 2, binding = 3) uniform sampler2D u_map_specular;
layout(set = 2, binding = 4) uniform sampler2D u_map_specular_highlights;
// layout(set = 2, binding = 5) uniform sampler2D u_map_emissive;
// layout(set = 2, binding = 6) uniform sampler2D u_map_d;
// layout(set = 2, binding = 7) uniform sampler2D u_map_refl;

// https://www.fileformat.info/format/material/
layout(set = 2, binding = 0) uniform Material { // TODO use struct type 
    // diffuse reflectance
    // [0..3]: color 
    // [3]:
    //   0 -> [0..3] only
    //   1 -> map * [0..3]
    // //   2 -> map_alpha // TODO: implement
    vec4 diffuse;

    // ambient reflectance
    // [0..3]: color 
    // [3]:
    //   0 -> none,
    //   1 -> [0..3] only,
    //   2 -> map * [0..3],
    // //   3 -> map_alpha // TODO: implement
    vec4 ambient;

    // specular reflectance
    // [0..3]: specular color
    // [3]: size of specular highlights / shine
    //    0 -> dull unfocused -> no specular anything
    //    ..
    //    1000 -> shiny focused
    //    1001 -> specular highlight size map only
    //    n -> specular highlight size map + (n - 1001) where map + (n - 1001) <= 1000   
    //    2001 -> ...
    //    sign:
    //      + -> no specular map just [0..3]
    //      - -> specular map * [0..3]
    vec4 specular;

    // // emissive color
    // // |4th value|: 
    // //   0 -> no emissive anything
    // //   ..
    // //   1000 -> high
    // // 4th value sign:
    // //   + -> no mask map
    // //   - -> mask map
    // vec4 emissive_color,

    // [0]: alpha transparency:
    //     0 -> transparent 
    //     ..
    //     1 -> opaque
    // [1..2]: index of refraction: // TODO: implement
    //   [1]: 
    //     0..1000
    //   [2]: // redefine
    //     0 -> no effect
    //     1 -> no effect
    // [2]: illumination model:
    //     0 -> Color on and Ambient off
    //     1 -> Color on and Ambient on
    //     2 -> Highlight on
    //     3 -> Reflection on and Ray trace on
    //     4 -> Transparency: Glass on
    //     Reflection: Ray trace on
    //     5 -> Reflection: Fresnel on and Ray trace on
    //     6 -> Transparency: Refraction on
    //     Reflection: Fresnel off and Ray trace on
    //     7 -> Transparency: Refraction on
    //     Reflection: Fresnel on and Ray trace on
    //     8 -> Reflection on and Ray trace off
    //     9 -> Transparency: Glass on
    //     Reflection: Ray trace off
    //     10 -> Casts shadows onto invisible surfaces
    vec4 other;
} u_material;

layout(location = 0) out vec4 f_normals; // [x, y, z, 1.0]
layout(location = 1) out vec4 f_diffuse; // [r, g, b, 1.0]
layout(location = 2) out vec4 f_ambient; // [r, g, b, exists(1(yes), 0(no))]
layout(location = 3) out vec4 f_specular; // [r, g, b, size]
// layout(location = 4) out vec4 f_emissive_color; // 4th value: 0(none)..1000

void main() {
    // GLFragDepth = gl_FragCoord.z; https://www.khronos.org/registry/OpenGL-Refpages/es3.0/html/gl_FragCoord.xhtml

    // z_camera.default() ~ -z_worl
    
    // -z = w
    // gl_FragCoord.w = 1/Wc 
    // Wc = gl_Position.w from vertex shader
    // v_normal was built from z
    if (gl_FragCoord.w < 0.0 || gl_FragCoord.z == 0 || 0.0 <= dot(v_normal, normalize(vec3(v_camspace_xy.xy, -1.0/gl_FragCoord.w)))) {
        discard;
    }

    f_normals = vec4(v_normal, 1.0);
    f_specular = vec4(0.0, 0.0, 0.0, 0.0);
    f_ambient = vec4(0.0, 0.0, 0.0, 0.0);
    f_diffuse = vec4(1.0, 0.0, 1.0, 1.0);
    
    switch (int(u_material.other[3])) {
        // case 10:
        //     // Casts shadows onto invisible surfaces

        //     // TODO
            
        //     discard;
        // // Reflection: Ray trace off

        // // TODO

        // case 9:
        //     // Transparency: Glass on

        //     // TODO
            
        //     discard;
        // case 8: 
        //     // Reflection on and Ray trace off

        //     // TODO
            
        //     discard;
        // // Reflection: Fresnel on and Ray trace on

        // // TODO

        // case 7:
        //     // Transparency: Refraction on

        //     // TODO
            
        //     discard;
        // // Reflection: Fresnel off and Ray trace on

        // // TODO

        // case 6:
        //     // Transparency: Refraction on

        //     // TODO
            
        //     discard;
        // case 5:
        //     // Reflection: Fresnel on and Ray trace on
            
        //     // TODO
            
        //     discard;
        // //Reflection: Ray trace on

        // // TODO

        case 4:
            // Transparency: Glass on
            if (u_material.other[0] > 0.0 || u_material.other[0] < 1) {
                // TODO add to glass image
                
                discard;
            } else if (u_material.other[0] != 1) {
                discard;
            }
            discard;
        case 3:
            // Reflection on and Ray trace on

            // TODO

            discard;
        case 2:
            // Specular Highlights on
            float ns = abs(u_material.specular[3]);
            if (ns != 0 && ns <= 1000) {
                f_specular = u_material.specular;
            } else if (ns <= 2001) {
                ns += -1001 + texture(u_map_specular_highlights, v_texture_index)[0];
                if (ns != 0 && ns <= 1000){
                    f_specular = vec4(u_material.specular.rgb, ns);
                } else if (ns != 0) {
                    discard;
                }
            } else if (ns != 0) {
                discard;
            }

            if (u_material.specular[3] < 0){
                f_specular *= vec4(texture(u_map_specular, v_texture_index).rgb, 1.0);
            }
        case 1:
            // Color on and Ambient on
            switch (int(u_material.ambient[3])) {
                case 2:
                    f_ambient = vec4(texture(u_map_ambient, v_texture_index).rgb * u_material.ambient.rgb, 1.0);
                    break;
                case 1:
                    f_ambient = u_material.ambient;
                case 0:
                    break;
                default:
                    discard;
            }
        case 0:
            // Color on and Ambient off
            switch (int(u_material.diffuse[3])) {
                case 1:
                    f_diffuse = vec4(texture(u_map_diffuse, v_texture_index).rgb * u_material.diffuse.rgb, 1.0);
                    break;
                case 0:
                    f_diffuse = u_material.diffuse;
                    break;
                default:
                    discard;
            }
            break;
        default:
            discard; // TOKNOW does this disregard the already outputted values? ply yes
    }
    
}

// f_normals = vec4(v_normal, 1.0);
// f_specular = vec4(0.0, 0.0, 0.0, 0.0);
// f_ambient = u_material.ambient;
// f_diffuse = u_material.diffuse;
// f_diffuse = vec4((dot(normalize(v_normal), vec3(0.0, 0.0, 1.0)) + 1.0)/2.0, 0.0, -(dot(normalize(v_normal), vec3(0.0, 0.0, 1.0)) - 1.0)/2.0, 1.0);
// f_ambient = f_diffuse;
// TODO: Rasterization
// if (dot(normalize(v_normal), normalize(gl_FragCoord.xyz)) > 0.0 || gl_FragCoord.z < 0.0) {
//     discard;
// }
// float brightness = dot(normalize(v_normal), normalize(LIGHT));
// vec3 dark_color = vec3(1.0, 0.0, 0.0);
// // vec3 regular_color = vec3(0.7, 0.7, 0.7);

// if (gl_FragCoord.w < 0.0 || gl_FragCoord.w > 1.0 ){
//     f_ambient = vec4(1.0, 0.0, 0.0, 1.0);
//     f_diffuse = vec4(1.0, 0.0, 0.0, 1.0);
// } else {    
//     f_ambient = vec4(0.0, 0.0, 1.0, 1.0);
//     f_diffuse = vec4(0.0, 0.0, 1.0, 1.0);
// }