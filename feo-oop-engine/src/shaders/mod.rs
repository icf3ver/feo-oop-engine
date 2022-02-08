//! GLSL Shaders
//! 
//! TODO convert to SPIR-V (in this patch)


/// vertex shader that parses a model
pub(crate) mod vs_draw {
    use std::{sync::Arc, borrow::Cow, ffi::CStr};
    use vulkano::{pipeline::shader::{ShaderModule, GraphicsShaderType, ShaderInterfaceDef, ShaderInterfaceDefEntry, GraphicsEntryPoint}, OomError, device::Device, format::Format, descriptor::{descriptor::{ShaderStages, DescriptorDesc, DescriptorDescTy, DescriptorBufferDesc}, pipeline_layout::{PipelineLayoutDesc, PipelineLayoutDescPcRange}}};

    const SPIRV: &'static [u8] = &*include_bytes!("./spv_precompiles/vs_draw.spv");

    pub mod ty {
        #[repr(C)]
        pub struct Camera{
            pub to_view: [[f32; 4]; 4], // ident space to view space
            pub view_to_screen: [[f32; 4]; 4], // view space to screen space
        }
        #[repr(C)]
        pub struct World{
            pub object_to: [[f32; 4]; 4], // ident space to view space
        }
    }
    #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
    pub struct VertInput;

    unsafe impl ShaderInterfaceDef for VertInput {
        type Iter = VertInputIter;

        fn elements(&self) -> VertInputIter {
            VertInputIter(0)
        }
    }

    #[derive(Debug, Copy, Clone)]
    pub struct VertInputIter(u16);

    impl Iterator for VertInputIter {
        type Item = ShaderInterfaceDefEntry;

        #[inline]
        fn next(&mut self) -> Option<Self::Item> {
            if self.0 == 2 {
                self.0 += 1;
                return Some(ShaderInterfaceDefEntry {
                    location: 0..1,
                    format: Format::R32G32B32Sfloat,
                    name: Some(Cow::Borrowed("position")),
                });
            } else if self.0 == 1 {
                self.0 += 1;
                return Some(ShaderInterfaceDefEntry {
                    location: 1..2,
                    format: Format::R32G32B32Sfloat,
                    name: Some(Cow::Borrowed("normal")),
                });
            } else if self.0 == 0 {
                self.0 += 1;
                return Some(ShaderInterfaceDefEntry {
                    location: 2..3,
                    format: Format::R32G32Sfloat,
                    name: Some(Cow::Borrowed("texture_index")),
                });
            }
            None
        }

        #[inline]
        fn size_hint(&self) -> (usize, Option<usize>) {
            let len = (3 - self.0) as usize;
            (len, Some(len))
        }
    }

    impl ExactSizeIterator for VertInputIter {}

    #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
    pub struct VertOutput;

    unsafe impl ShaderInterfaceDef for VertOutput {
        type Iter = VertOutputIter;

        fn elements(&self) -> VertOutputIter {
            VertOutputIter(0)
        }
    }

    #[derive(Debug, Copy, Clone)]
    pub struct VertOutputIter(u16);

    impl Iterator for VertOutputIter {
        type Item = ShaderInterfaceDefEntry;

        #[inline]
        fn next(&mut self) -> Option<Self::Item> {
            if self.0 == 2 {
                self.0 += 1;
                return Some(ShaderInterfaceDefEntry {
                    location: 0..1,
                    format: Format::R32G32B32Sfloat,
                    name: Some(Cow::Borrowed("v_normal")),
                });
            } else if self.0 == 1 {
                self.0 += 1;
                return Some(ShaderInterfaceDefEntry {
                    location: 1..2,
                    format: Format::R32G32Sfloat,
                    name: Some(Cow::Borrowed("v_texture_index")),
                });
            } else if self.0 == 0 {
                self.0 += 1;
                return Some(ShaderInterfaceDefEntry {
                    location: 2..3,
                    format: Format::R32G32Sfloat,
                    name: Some(Cow::Borrowed("v_camspace_xy")),
                });
            }
            None
        }

        #[inline]
        fn size_hint(&self) -> (usize, Option<usize>) {
            let len = (3 - self.0) as usize;
            (len, Some(len))
        }
    }

    impl ExactSizeIterator for VertOutputIter {}

    // This structure describes layout of this stage.
    #[derive(Debug, Copy, Clone)]
    pub struct VertLayout(ShaderStages);
    unsafe impl PipelineLayoutDesc for VertLayout {
        // Number of descriptor sets it takes.
        fn num_sets(&self) -> usize {
            2
        }
        // Number of entries (bindings) in each set.
        fn num_bindings_in_set(&self, _set: usize) -> Option<usize> {
            Some(1)
        }
        // Descriptor descriptions.
        fn descriptor(&self, set: usize, binding: usize) -> Option<DescriptorDesc> {
            if set == 0 && binding == 0 {
                Some(DescriptorDesc {
                    ty: DescriptorDescTy::Buffer(
                        DescriptorBufferDesc{
                            dynamic: None,
                            storage: false
                        }
                    ),
                    array_count: 1,
                    stages: ShaderStages{
                        vertex: true,
                        ..ShaderStages::none()
                    },
                    readonly: true
                })
            } else if set == 1 && binding == 0 {
                Some(DescriptorDesc {
                    ty: DescriptorDescTy::Buffer(
                        DescriptorBufferDesc{
                            dynamic: None,
                            storage: false
                        }
                    ),
                    array_count: 1,
                    stages: ShaderStages{
                        vertex: true,
                        ..ShaderStages::none()
                    },
                    readonly: false
                })
            } else {
                None
            }
        }
        // Number of push constants ranges (think: number of push constants).
        fn num_push_constants_ranges(&self) -> usize {
            0
        }
        // Each push constant range in memory.
        fn push_constants_range(&self, _num: usize) -> Option<PipelineLayoutDescPcRange> {
            None
        }
    }
    pub struct Shader {
        module: Arc<ShaderModule>
    }
    impl Shader {
        pub fn load(device: Arc<Device>) -> Result<Self, OomError> {
            Ok(Shader{ module: unsafe { ShaderModule::new(device.clone(), &SPIRV) }? })
        }
        pub fn main_entry_point(&self) -> GraphicsEntryPoint<(), VertInput, VertOutput, VertLayout> {
            unsafe{ 
                self.module.graphics_entry_point(
                    CStr::from_bytes_with_nul_unchecked(b"main\0"),
                    VertInput,
                    VertOutput,
                    VertLayout(ShaderStages {
                        vertex: true,
                        ..ShaderStages::none()
                    }),
                    GraphicsShaderType::Vertex,
                )
            }
        }
    }

    // vulkano_shaders::shader! {
    //     ty: "vertex",
    //     path: "./src/shaders/draw/vs_draw.vert"
    // }
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
