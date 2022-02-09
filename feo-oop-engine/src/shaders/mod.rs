//! GLSL Shaders

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
                    readonly: true
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
    use std::{borrow::Cow, sync::Arc, ffi::CStr};
    use vulkano::{pipeline::shader::{ShaderInterfaceDef, ShaderInterfaceDefEntry, ShaderModule, GraphicsEntryPoint, GraphicsShaderType}, format::Format, descriptor::{descriptor::{ShaderStages, DescriptorDesc, DescriptorDescTy, DescriptorBufferDesc, DescriptorImageDesc, DescriptorImageDescDimensions, DescriptorImageDescArray}, pipeline_layout::{PipelineLayoutDesc, PipelineLayoutDescPcRange}}, device::Device, OomError};

    const SPIRV: &'static [u8] = &*include_bytes!("./spv_precompiles/fs_draw.spv");
    
    pub mod ty {
        #[repr(C)]
        #[derive(Clone)]
        pub struct Material {// diffuse reflectance
            // [0..3]: color 
            // [3]:
            //   0 -> [0..3] only
            //   1 -> map * [0..3]
            // //   2 -> map_alpha // TODO: implement
            pub diffuse: [f32; 4],
        
            // ambient reflectance
            // [0..3]: color 
            // [3]:
            //   0 -> none,
            //   1 -> [0..3] only,
            //   2 -> map * [0..3],
            // //   3 -> map_alpha // TODO: implement
            pub ambient: [f32; 4],
        
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
            pub specular: [f32; 4],
        
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
            pub other: [f32; 4],
        }
    }

    // Same as with our vertex shader, but for fragment one instead.
    #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
    pub struct FragInput;
    unsafe impl ShaderInterfaceDef for FragInput {
        type Iter = FragInputIter;

        fn elements(&self) -> FragInputIter {
            FragInputIter(0)
        }
    }
    #[derive(Debug, Copy, Clone)]
    pub struct FragInputIter(u16);

    impl Iterator for FragInputIter {
        type Item = ShaderInterfaceDefEntry;

        #[inline]
        fn next(&mut self) -> Option<Self::Item> {
            if self.0 == 2 {
                self.0 += 1;
                return Some(ShaderInterfaceDefEntry {
                    location: 2..3,
                    format: Format::R32G32Sfloat,
                    name: Some(Cow::Borrowed("v_camspace_xy")),
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
                    location: 0..1,
                    format: Format::R32G32B32Sfloat,
                    name: Some(Cow::Borrowed("v_normal")),
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

    impl ExactSizeIterator for FragInputIter {}

    #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
    pub struct FragOutput;
    unsafe impl ShaderInterfaceDef for FragOutput {
        type Iter = FragOutputIter;

        fn elements(&self) -> FragOutputIter {
            FragOutputIter(0)
        }
    }

    #[derive(Debug, Copy, Clone)]
    pub struct FragOutputIter(u16);

    impl Iterator for FragOutputIter {
        type Item = ShaderInterfaceDefEntry;

        #[inline]
        fn next(&mut self) -> Option<Self::Item> {
            if self.0 == 3 {
                self.0 += 1;
                return Some(ShaderInterfaceDefEntry {
                    location: 0..1,
                    format: Format::R32G32B32A32Sfloat,
                    name: Some(Cow::Borrowed("f_normals")),
                });
            } else if self.0 == 2 {
                self.0 += 1;
                return Some(ShaderInterfaceDefEntry {
                    location: 1..2,
                    format: Format::R32G32B32A32Sfloat,
                    name: Some(Cow::Borrowed("f_diffuse")),
                });
            } else if self.0 == 1 {
                self.0 += 1;
                return Some(ShaderInterfaceDefEntry {
                    location: 2..3,
                    format: Format::R32G32B32A32Sfloat,
                    name: Some(Cow::Borrowed("f_ambient")),
                });
            } else if self.0 == 0 {
                self.0 += 1;
                return Some(ShaderInterfaceDefEntry {
                    location: 3..4,
                    format: Format::R32G32B32A32Sfloat,
                    name: Some(Cow::Borrowed("f_specular")),
                });
            }
            None
        }
        #[inline]
        fn size_hint(&self) -> (usize, Option<usize>) {
            let len = (4 - self.0) as usize;
            (len, Some(len))
        }
    }

    impl ExactSizeIterator for FragOutputIter {}

    // Layout same as with vertex shader.
    #[derive(Debug, Copy, Clone)]
    pub struct FragLayout(ShaderStages);
    unsafe impl PipelineLayoutDesc for FragLayout {
        fn num_sets(&self) -> usize {
            3
        }
        fn num_bindings_in_set(&self, set: usize) -> Option<usize> {
            if set == 2 {
                Some(5)
            } else {
                None
            }
        }
        fn descriptor(&self, set: usize, binding: usize) -> Option<DescriptorDesc> {
            if set == 2 && binding == 0 {
                return Some(DescriptorDesc {
                    ty: DescriptorDescTy::Buffer(
                        DescriptorBufferDesc{
                            dynamic: None,
                            storage: false
                        }
                    ),
                    array_count: 1,
                    stages: ShaderStages{
                        fragment: true,
                        ..ShaderStages::none()
                    },
                    readonly: false
                });
            } else if set == 2 && (binding >= 1 || binding <= 4) {
                return Some(DescriptorDesc {
                    ty: DescriptorDescTy::CombinedImageSampler(DescriptorImageDesc{
                        sampled: true,
                        dimensions: DescriptorImageDescDimensions::TwoDimensional,
                        format: None,
                        multisampled: false,
                        array_layers: DescriptorImageDescArray::NonArrayed,
                    }),
                    array_count: 1,
                    stages: ShaderStages{
                        fragment: true,
                        ..ShaderStages::none()
                    },
                    readonly: true
                });
            }
            None
        }
        fn num_push_constants_ranges(&self) -> usize {
            0
        }
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
        pub fn main_entry_point(&self) -> GraphicsEntryPoint<(), FragInput, FragOutput, FragLayout> {
            unsafe{ 
                self.module.graphics_entry_point(
                    CStr::from_bytes_with_nul_unchecked(b"main\0"),
                    FragInput,
                    FragOutput,
                    FragLayout(ShaderStages {
                        fragment: true,
                        ..ShaderStages::none()
                    }),
                    GraphicsShaderType::Fragment,
                )
            }
        }
    }

    // vulkano_shaders::shader! {
    //     ty: "fragment",
    //     path: "./src/shaders/draw/fs_draw.frag"
    // }
}

/// vertex shader for light systems
pub(crate) mod vs_lighting {
    use std::{sync::Arc, borrow::Cow, ffi::CStr};
    use vulkano::{pipeline::shader::{ShaderModule, GraphicsShaderType, ShaderInterfaceDef, ShaderInterfaceDefEntry, GraphicsEntryPoint}, OomError, device::Device, format::Format, descriptor::{descriptor::{ShaderStages, DescriptorDesc}, pipeline_layout::{PipelineLayoutDesc, PipelineLayoutDescPcRange}}};

    const SPIRV: &'static [u8] = &*include_bytes!("./spv_precompiles/vs_lighting.spv");

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
            if self.0 == 0 {
                self.0 += 1;
                return Some(ShaderInterfaceDefEntry {
                    location: 0..1,
                    format: Format::R32G32Sfloat,
                    name: Some(Cow::Borrowed("position")),
                });
            }
            None
        }

        #[inline]
        fn size_hint(&self) -> (usize, Option<usize>) {
            let len = (1 - self.0) as usize;
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
            None
        }

        #[inline]
        fn size_hint(&self) -> (usize, Option<usize>) {
            (0, Some(0))
        }
    }

    impl ExactSizeIterator for VertOutputIter {}

    // This structure describes layout of this stage.
    #[derive(Debug, Copy, Clone)]
    pub struct VertLayout(ShaderStages);
    unsafe impl PipelineLayoutDesc for VertLayout {
        // Number of descriptor sets it takes.
        fn num_sets(&self) -> usize {
            0
        }
        // Number of entries (bindings) in each set.
        fn num_bindings_in_set(&self, _set: usize) -> Option<usize> {
            None
        }
        // Descriptor descriptions.
        fn descriptor(&self, _set: usize, _binding: usize) -> Option<DescriptorDesc> {
            None
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
    //     path: "./src/shaders/lighting/vs_lighting.vert"
    // }
}

/// fragment shader for a point light
pub(crate) mod fs_lighting_point {
    use std::{borrow::Cow, sync::Arc, ffi::CStr};
    use vulkano::{pipeline::shader::{ShaderInterfaceDef, ShaderInterfaceDefEntry, ShaderModule, GraphicsEntryPoint, GraphicsShaderType}, format::Format, descriptor::{descriptor::{ShaderStages, DescriptorDesc, DescriptorDescTy, DescriptorImageDescArray}, pipeline_layout::{PipelineLayoutDesc, PipelineLayoutDescPcRange}}, device::Device, OomError};

    const SPIRV: &'static [u8] = &*include_bytes!("./spv_precompiles/fs_lighting_point.spv");
    
    pub mod ty {
        #[repr(C)]
        pub struct PushConstants {
            // Screen space to camera space projection matrix
            pub screen_to_camera: [[f32; 4]; 4],
            // The point lights color
            pub color: [f32; 4],
            // The position of the light in camera space
            pub position_camera_space: [f32; 4]
        }
    }

    // Same as with our vertex shader, but for fragment one instead.
    #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
    pub struct FragInput;
    unsafe impl ShaderInterfaceDef for FragInput {
        type Iter = FragInputIter;

        fn elements(&self) -> FragInputIter {
            FragInputIter(0)
        }
    }
    #[derive(Debug, Copy, Clone)]
    pub struct FragInputIter(u16);

    impl Iterator for FragInputIter {
        type Item = ShaderInterfaceDefEntry;

        #[inline]
        fn next(&mut self) -> Option<Self::Item> {
            None
        }

        #[inline]
        fn size_hint(&self) -> (usize, Option<usize>) {
            (0, Some(0))
        }
    }

    impl ExactSizeIterator for FragInputIter {}

    #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
    pub struct FragOutput;
    unsafe impl ShaderInterfaceDef for FragOutput {
        type Iter = FragOutputIter;

        fn elements(&self) -> FragOutputIter {
            FragOutputIter(0)
        }
    }

    #[derive(Debug, Copy, Clone)]
    pub struct FragOutputIter(u16);

    impl Iterator for FragOutputIter {
        type Item = ShaderInterfaceDefEntry;

        #[inline]
        fn next(&mut self) -> Option<Self::Item> {
            if self.0 == 0 {
                self.0 += 1;
                return Some(ShaderInterfaceDefEntry {
                    location: 0..1,
                    format: Format::R32G32B32A32Sfloat,
                    name: Some(Cow::Borrowed("f_color")),
                });
            }
            None
        }
        #[inline]
        fn size_hint(&self) -> (usize, Option<usize>) {
            let len = (1 - self.0) as usize;
            (len, Some(len))
        }
    }

    impl ExactSizeIterator for FragOutputIter {}

    // Layout same as with vertex shader.
    #[derive(Debug, Copy, Clone)]
    pub struct FragLayout(ShaderStages);
    unsafe impl PipelineLayoutDesc for FragLayout {
        fn num_sets(&self) -> usize {
            1
        }
        fn num_bindings_in_set(&self, _set: usize) -> Option<usize> {
            Some(4)
        }
        fn descriptor(&self, set: usize, binding: usize) -> Option<DescriptorDesc> {
            if set == 0 && binding == 0 {
                return Some(DescriptorDesc {
                    ty: DescriptorDescTy::InputAttachment{
                        multisampled: false,
                        array_layers: DescriptorImageDescArray::NonArrayed,
                    },
                    array_count: 1,
                    stages: ShaderStages{
                        fragment: true,
                        ..ShaderStages::none()
                    },
                    readonly: true
                });
            } else if set == 0 && binding <= 3 {
                return Some(DescriptorDesc {
                    ty: DescriptorDescTy::InputAttachment{
                        multisampled: false,
                        array_layers: DescriptorImageDescArray::NonArrayed,
                    },
                    array_count: 1,
                    stages: ShaderStages{
                        fragment: true,
                        ..ShaderStages::none()
                    },
                    readonly: true
                });
            } else {
                return None;
            }
        }
        fn num_push_constants_ranges(&self) -> usize {
            1
        }
        fn push_constants_range(&self, _num: usize) -> Option<PipelineLayoutDescPcRange> {
            Some(
                PipelineLayoutDescPcRange{
                    offset: 0,
                    size: 96,
                    stages: ShaderStages{
                        fragment: true,
                        ..ShaderStages::none()
                    },
                }
            )
        }
    }

    pub struct Shader {
        module: Arc<ShaderModule>
    }
    impl Shader {
        pub fn load(device: Arc<Device>) -> Result<Self, OomError> {
            Ok(Shader{ module: unsafe { ShaderModule::new(device.clone(), &SPIRV) }? })
        }
        pub fn main_entry_point(&self) -> GraphicsEntryPoint<(), FragInput, FragOutput, FragLayout> {
            unsafe{ 
                self.module.graphics_entry_point(
                    CStr::from_bytes_with_nul_unchecked(b"main\0"),
                    FragInput,
                    FragOutput,
                    FragLayout(ShaderStages {
                        fragment: true,
                        ..ShaderStages::none()
                    }),
                    GraphicsShaderType::Fragment,
                )
            }
        }
    }

    // vulkano_shaders::shader! {
    //     ty: "fragment",
    //     path: "./src/shaders/lighting/fs_lighting_point.frag"
    // }
}

/// fragment shader for directional lighting
pub(crate) mod fs_lighting_directional {
    use std::{borrow::Cow, sync::Arc, ffi::CStr};
    use vulkano::{pipeline::shader::{ShaderInterfaceDef, ShaderInterfaceDefEntry, ShaderModule, GraphicsEntryPoint, GraphicsShaderType}, format::Format, descriptor::{descriptor::{ShaderStages, DescriptorDesc, DescriptorDescTy, DescriptorImageDescArray}, pipeline_layout::{PipelineLayoutDesc, PipelineLayoutDescPcRange}}, device::Device, OomError};

    const SPIRV: &'static [u8] = &*include_bytes!("./spv_precompiles/fs_lighting_directional.spv");

    pub mod ty {
        #[repr(C)]
        pub struct PushConstants {
            // The color of the directional light
            pub color: [f32; 4],
            // The direction in which the directional light is shining
            pub direction: [f32; 4]
        }
    }

    // Same as with our vertex shader, but for fragment one instead.
    #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
    pub struct FragInput;
    unsafe impl ShaderInterfaceDef for FragInput {
        type Iter = FragInputIter;

        fn elements(&self) -> FragInputIter {
            FragInputIter(0)
        }
    }
    #[derive(Debug, Copy, Clone)]
    pub struct FragInputIter(u16);

    impl Iterator for FragInputIter {
        type Item = ShaderInterfaceDefEntry;

        #[inline]
        fn next(&mut self) -> Option<Self::Item> {
            None
        }

        #[inline]
        fn size_hint(&self) -> (usize, Option<usize>) {
            (0, Some(0))
        }
    }

    impl ExactSizeIterator for FragInputIter {}

    #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
    pub struct FragOutput;
    unsafe impl ShaderInterfaceDef for FragOutput {
        type Iter = FragOutputIter;

        fn elements(&self) -> FragOutputIter {
            FragOutputIter(0)
        }
    }

    #[derive(Debug, Copy, Clone)]
    pub struct FragOutputIter(u16);

    impl Iterator for FragOutputIter {
        type Item = ShaderInterfaceDefEntry;

        #[inline]
        fn next(&mut self) -> Option<Self::Item> {
            if self.0 == 0 {
                self.0 += 1;
                return Some(ShaderInterfaceDefEntry {
                    location: 0..1,
                    format: Format::R32G32B32A32Sfloat,
                    name: Some(Cow::Borrowed("f_color")),
                });
            }
            None
        }
        #[inline]
        fn size_hint(&self) -> (usize, Option<usize>) {
            let len = (1 - self.0) as usize;
            (len, Some(len))
        }
    }

    impl ExactSizeIterator for FragOutputIter {}

    // Layout same as with vertex shader.
    #[derive(Debug, Copy, Clone)]
    pub struct FragLayout(ShaderStages);
    unsafe impl PipelineLayoutDesc for FragLayout {
        fn num_sets(&self) -> usize {
            1
        }
        fn num_bindings_in_set(&self, _set: usize) -> Option<usize> {
            Some(2)
        }
        fn descriptor(&self, set: usize, binding: usize) -> Option<DescriptorDesc> {
            if set == 0 && binding == 0 {
                return Some(DescriptorDesc {
                    ty: DescriptorDescTy::InputAttachment{
                        multisampled: false,
                        array_layers: DescriptorImageDescArray::NonArrayed,
                    },
                    array_count: 1,
                    stages: ShaderStages{
                        fragment: true,
                        ..ShaderStages::none()
                    },
                    readonly: true
                });
            } else if set == 0 && binding == 1 {
                return Some(DescriptorDesc {
                    ty: DescriptorDescTy::InputAttachment{
                        multisampled: false,
                        array_layers: DescriptorImageDescArray::NonArrayed,
                    },
                    array_count: 1,
                    stages: ShaderStages{
                        fragment: true,
                        ..ShaderStages::none()
                    },
                    readonly: true
                });
            } else {
                return None;
            }
        }
        fn num_push_constants_ranges(&self) -> usize {
            1
        }
        fn push_constants_range(&self, _num: usize) -> Option<PipelineLayoutDescPcRange> {
            Some(
                PipelineLayoutDescPcRange{
                    offset: 0,
                    size: 32,
                    stages: ShaderStages{
                        fragment: true,
                        ..ShaderStages::none()
                    },
                }
            )
        }
    }

    pub struct Shader {
        module: Arc<ShaderModule>
    }
    impl Shader {
        pub fn load(device: Arc<Device>) -> Result<Self, OomError> {
            Ok(Shader{ module: unsafe { ShaderModule::new(device.clone(), &SPIRV) }? })
        }
        pub fn main_entry_point(&self) -> GraphicsEntryPoint<(), FragInput, FragOutput, FragLayout> {
            unsafe{ 
                self.module.graphics_entry_point(
                    CStr::from_bytes_with_nul_unchecked(b"main\0"),
                    FragInput,
                    FragOutput,
                    FragLayout(ShaderStages {
                        fragment: true,
                        ..ShaderStages::none()
                    }),
                    GraphicsShaderType::Fragment,
                )
            }
        }
    }

    // vulkano_shaders::shader! {
    //     ty: "fragment",
    //     path: "./src/shaders/lighting/fs_lighting_directional.frag"
    // }
}

/// fragment shader for ambient lighting 
pub(crate) mod fs_lighting_ambient {
    use std::{borrow::Cow, sync::Arc, ffi::CStr};
    use vulkano::{pipeline::shader::{ShaderInterfaceDef, ShaderInterfaceDefEntry, ShaderModule, GraphicsEntryPoint, GraphicsShaderType}, format::Format, descriptor::{descriptor::{ShaderStages, DescriptorDesc, DescriptorDescTy, DescriptorImageDescArray}, pipeline_layout::{PipelineLayoutDesc, PipelineLayoutDescPcRange}}, device::Device, OomError};

    const SPIRV: &'static [u8] = &*include_bytes!("./spv_precompiles/fs_lighting_ambient.spv");
    
    pub mod ty {
        #[repr(C)]
        pub struct PushConstants {
            // The color of the ambient light
            pub color: [f32; 4]
        }
    }

    // Same as with our vertex shader, but for fragment one instead.
    #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
    pub struct FragInput;
    unsafe impl ShaderInterfaceDef for FragInput {
        type Iter = FragInputIter;

        fn elements(&self) -> FragInputIter {
            FragInputIter(0)
        }
    }
    #[derive(Debug, Copy, Clone)]
    pub struct FragInputIter(u16);

    impl Iterator for FragInputIter {
        type Item = ShaderInterfaceDefEntry;

        #[inline]
        fn next(&mut self) -> Option<Self::Item> {
            None
        }

        #[inline]
        fn size_hint(&self) -> (usize, Option<usize>) {
            (0, Some(0))
        }
    }

    impl ExactSizeIterator for FragInputIter {}

    #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
    pub struct FragOutput;
    unsafe impl ShaderInterfaceDef for FragOutput {
        type Iter = FragOutputIter;

        fn elements(&self) -> FragOutputIter {
            FragOutputIter(0)
        }
    }

    #[derive(Debug, Copy, Clone)]
    pub struct FragOutputIter(u16);

    impl Iterator for FragOutputIter {
        type Item = ShaderInterfaceDefEntry;

        #[inline]
        fn next(&mut self) -> Option<Self::Item> {
            if self.0 == 0 {
                self.0 += 1;
                return Some(ShaderInterfaceDefEntry {
                    location: 0..1,
                    format: Format::R32G32B32A32Sfloat,
                    name: Some(Cow::Borrowed("f_color")),
                });
            }
            None
        }
        #[inline]
        fn size_hint(&self) -> (usize, Option<usize>) {
            let len = (1 - self.0) as usize;
            (len, Some(len))
        }
    }

    impl ExactSizeIterator for FragOutputIter {}

    // Layout same as with vertex shader.
    #[derive(Debug, Copy, Clone)]
    pub struct FragLayout(ShaderStages);
    unsafe impl PipelineLayoutDesc for FragLayout {
        fn num_sets(&self) -> usize {
            1
        }
        fn num_bindings_in_set(&self, _set: usize) -> Option<usize> {
            Some(3)
        }
        fn descriptor(&self, set: usize, binding: usize) -> Option<DescriptorDesc> {
            if set == 0 && binding == 0 {
                return Some(DescriptorDesc {
                    ty: DescriptorDescTy::InputAttachment{
                        multisampled: false,
                        array_layers: DescriptorImageDescArray::NonArrayed,
                    },
                    array_count: 1,
                    stages: ShaderStages{
                        fragment: true,
                        ..ShaderStages::none()
                    },
                    readonly: true
                });
            } else if set == 0 && binding <= 2 {
                return Some(DescriptorDesc {
                    ty: DescriptorDescTy::InputAttachment{
                        multisampled: false,
                        array_layers: DescriptorImageDescArray::NonArrayed,
                    },
                    array_count: 1,
                    stages: ShaderStages{
                        fragment: true,
                        ..ShaderStages::none()
                    },
                    readonly: true
                });
            } else {
                return None;
            }
        }
        fn num_push_constants_ranges(&self) -> usize {
            1
        }
        fn push_constants_range(&self, _num: usize) -> Option<PipelineLayoutDescPcRange> {
            Some(
                PipelineLayoutDescPcRange{
                    offset: 0,
                    size: 16,
                    stages: ShaderStages{
                        fragment: true,
                        ..ShaderStages::none()
                    },
                }
            )
        }
    }

    pub struct Shader {
        module: Arc<ShaderModule>
    }
    impl Shader {
        pub fn load(device: Arc<Device>) -> Result<Self, OomError> {
            Ok(Shader{ module: unsafe { ShaderModule::new(device.clone(), &SPIRV) }? })
        }
        pub fn main_entry_point(&self) -> GraphicsEntryPoint<(), FragInput, FragOutput, FragLayout> {
            unsafe{ 
                self.module.graphics_entry_point(
                    CStr::from_bytes_with_nul_unchecked(b"main\0"),
                    FragInput,
                    FragOutput,
                    FragLayout(ShaderStages {
                        fragment: true,
                        ..ShaderStages::none()
                    }),
                    GraphicsShaderType::Fragment,
                )
            }
        }
    }

    // vulkano_shaders::shader! {
    //     ty: "fragment",
    //     path: "./src/shaders/lighting/fs_lighting_ambient.frag"
    // }
}
