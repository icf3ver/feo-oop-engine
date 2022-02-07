//! GLSL Shaders
//! 
//! TODO convert to SPIR-V (in this patch)

/// vertex shader that parses a model
pub(crate) mod vs_draw {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "./src/shaders/draw/vs_draw.vert"
    }
}

/// fragment shader that writes out diffuse color and albedo
pub(crate) mod fs_draw {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "./src/shaders/draw/fs_draw.frag"
    }
}

/// vertex shader for light systems
pub(crate) mod vs_lighting {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "./src/shaders/lighting/vs_lighting.vert"
    }
}

/// fragment shader for a point light
pub(crate) mod fs_lighting_point {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "./src/shaders/lighting/fs_lighting_point.frag"
    }
}

/// fragment shader for directional lighting
pub(crate) mod fs_lighting_directional {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "./src/shaders/lighting/fs_lighting_directional.frag"
    }
}

/// fragment shader for ambient lighting 
pub(crate) mod fs_lighting_ambient {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "./src/shaders/lighting/fs_lighting_ambient.frag"
    }
}
