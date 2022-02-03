//! A light that shines from a point
//! 
//! TODO
//! 
use {
    crate::{ 
        registration::{
            relation::{
                ParentWrapper, 
                Parent, Child
            },
            named::Named,
            id::ID
        },
        scripting::{
            Script,
            executor::Spawner,
            globals::{Global, EngineGlobals}, 
            Scriptable
        }, // proc-macro
        graphics::{ 
            Drawable, 
            draw_pass_manager::DrawPassManager,
            lighting_pass_manager::LightingPassManager,
            graphics_system::{GraphicsSystem, GraphicsSystemTrait}, 
            pass_builder::PassBuilder
        },
        shaders::{
            fs_lighting_point, 
            vs_lighting
        },  
        components::{
            triangle_mesh::TriangleMesh,
            RGB, 
            ScreenPos
        },
        event::UserEvent
    },
    vulkano::{
        buffer::{
            BufferUsage,
            CpuAccessibleBuffer,
        }, 
        command_buffer::{
            AutoCommandBuffer, 
            AutoCommandBufferBuilder,
            DynamicState
        }, 
        device::Queue,
        descriptor::{
            descriptor_set::PersistentDescriptorSet,
        },
        framebuffer::{
            RenderPassAbstract,
            Subpass
        },
        image::ImageViewAbstract,
        pipeline::{
            GraphicsPipeline, 
            GraphicsPipelineAbstract, 
            blend::{
                AttachmentBlend, 
                BlendFactor, 
                BlendOp
            }, 
            viewport::Viewport
        },
    },
    feo_math::{
        linear_algebra::{
            matrix4::Matrix4, 
            vector3::Vector3, 
            vector4::Vector4
        }, 
        rotation::quaternion::Quaternion, 
        utils::space::Space
    },
    winit::event::Event,
    std::{
        sync::{
            Arc, 
            RwLock
        },
        any::Any,
        mem
    },
    super::{
        super::{
            GameObject,
            camera::Camera,
        },
        Light
    }
};

/// Allows applying a directional light source to a scene.
#[derive(GameObject, Drawable, Parent, Child, Named, Scriptable)]
#[light]
pub struct PointLight {
    name: String,
    id: ID,

    pub subspace: Space,

    intensity: f32,
    color: RGB,

    parent: ParentWrapper,
    children: Vec<Arc<RwLock<dyn GameObject>>>,

    script: Option<Box<Script<Self>>>,
}

impl std::fmt::Debug for PointLight {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PointLight")
            .field("name", &self.name)
            .field("id", &self.id)
            .field("subspace", &self.subspace)
            .field("intensity", &self.intensity)
            .field("color", &self.color)
            .field("parent", &self.parent)
            .field("children", &self.children)
            .field("script", &self.script).finish()
    }
}

impl Clone for PointLight {
    fn clone(&self) -> Self {
        let id = self.id.get_system().take();
        PointLight{
            id,
            name: self.name.clone(),
            parent: self.parent.clone(),
            subspace: self.subspace,
            script: self.script.clone(),
            children: self.children.clone().into_iter().map(|_child| {
                // Dangerous
                todo!();
            }).collect::<Vec<Arc<RwLock<dyn GameObject>>>>(),
            intensity: self.intensity,
            color: self.color,
        }
    }
}

impl PointLight {
    /// Initializes the directional lighting system.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        name: Option<&str>,
        parent: Option<Arc<RwLock<dyn GameObject>>>,
        intensity: f32,
        color: RGB,
        
        position: Option<Vector3<f32>>, // no effect on actual lighting
        rotation: Option<Quaternion<f32>>,
        scale_factor: Option<Vector3<f32>>, // no effect on actual lighting
        
        script: Option<Box<Script<Self>>>,

        engine_globals: EngineGlobals) -> Arc<RwLock<PointLight>> {
        let id = engine_globals.id_system.take();
        Arc::new( RwLock::new( PointLight {
            name: match name {
                Some(name) => name.to_string(),
                None => String::from("point_light_") + id.to_string().as_str()
            },
            id, // TODO make id removable

            intensity,
            color,
            subspace: Space::new(position, rotation, scale_factor),

            parent: match parent{
                Some(game_object) => ParentWrapper::GameObject(game_object), 
                None => ParentWrapper::Scene(engine_globals.scene),
            },
            children: Vec::new(),

            script,
        }))
    }

    #[allow(clippy::too_many_arguments)]
    pub fn draw<N, C, S, D>(
        &self,
        graphics_system: GraphicsSystem, 
        viewport_dimensions: [u32; 2],
        normals_input: N,
        diffuse_input: C,
        specular_input: S,
        depth_input: D,
        screen_to_camera: Matrix4<f32>,
        to_camera_space: Space
    ) -> AutoCommandBuffer
    where
        N: ImageViewAbstract + Send + Sync + 'static,
        C: ImageViewAbstract + Send + Sync + 'static,
        S: ImageViewAbstract + Send + Sync + 'static,
        D: ImageViewAbstract + Send + Sync + 'static,
    {
        let position_camera_space = Vector4::from_vector3(
            (to_camera_space.build() * Vector4::from_vector3(self.get_subspace().center, 1.0)).into(), // move item into space
            0.0
        ).into();

        let push_constants = fs_lighting_point::ty::PushConstants {
            screen_to_camera: screen_to_camera.transpose().into(),
            color: [self.color.r, self.color.g, self.color.b, 1.0],
            position_camera_space, // note scale position
        };

        let layout = graphics_system.pipeline.descriptor_set_layout(0).unwrap();

        let descriptor_set = PersistentDescriptorSet::start(layout.clone())
            .add_image(diffuse_input).unwrap()
            .add_image(specular_input).unwrap()
            .add_image(normals_input).unwrap()
            .add_image(depth_input).unwrap()
            .build().unwrap();

        let dynamic_state = DynamicState {
            viewports: Some(vec![Viewport {
                origin: [0.0, 0.0],
                dimensions: [viewport_dimensions[0] as f32, viewport_dimensions[1] as f32],
                depth_range: 0.0..1.0,
            }]),
            ..DynamicState::none()
        };

        let mut builder = AutoCommandBufferBuilder::secondary_graphics(
            graphics_system.gfx_queue.device().clone(),
            graphics_system.gfx_queue.family(),
            graphics_system.pipeline.clone().subpass().clone(),
        ).unwrap();

        builder.draw(
            graphics_system.pipeline.clone(),
            &dynamic_state,
            vec![graphics_system.vertex_buffer.clone()],
            descriptor_set,
            push_constants,
            vec![],
        ).unwrap();
        
        builder.build().unwrap()
    }
}

impl Light for PointLight {
    fn as_any(&self) -> &dyn Any { self }

    fn as_gameobject(&self) -> &dyn GameObject { self }

    fn cast_gameobject_arc_rwlock(&self, this: Arc<RwLock<dyn Light>>) -> Arc<RwLock<dyn GameObject>> {
        let this= Arc::into_raw(this).cast::<RwLock<Self>>();
        let this = unsafe { Arc::from_raw(this) };
        this as Arc<RwLock<dyn GameObject>>
    }

}

impl GraphicsSystemTrait for PointLight {

    fn new_system<L>(gfx_queue: Arc<Queue>, subpass: Subpass<L>) -> crate::graphics::graphics_system::GraphicsSystem
    where L: RenderPassAbstract + Sync + Send + 'static {
        let vertex_buffer = {
            CpuAccessibleBuffer::from_iter(
                gfx_queue.device().clone(),
                BufferUsage::all(),
                false,
                [
                    ScreenPos {
                        position: [-1.0, -1.0],
                    },
                    ScreenPos {
                        position: [-1.0, 3.0],
                    },
                    ScreenPos {
                        position: [3.0, -1.0],
                    },
                ].iter().cloned(),
            ).expect("failed to create buffer")
        };

        let pipeline = {
            let vs = vs_lighting::Shader::load(gfx_queue.device().clone())
                .expect("failed to create shader");
            let fs = fs_lighting_point::Shader::load(gfx_queue.device().clone())
                .expect("failed to create shader");

            Arc::new(
                GraphicsPipeline::start()
                    .vertex_input_single_buffer::<ScreenPos>()
                    .vertex_shader(vs.main_entry_point(), ())
                    .triangle_list()
                    .viewports_dynamic_scissors_irrelevant(1)
                    .fragment_shader(fs.main_entry_point(), ())
                    .blend_collective(AttachmentBlend {
                        enabled: true,
                        color_op: BlendOp::Add,
                        color_source: BlendFactor::One,
                        color_destination: BlendFactor::One,
                        alpha_op: BlendOp::Max,
                        alpha_source: BlendFactor::One,
                        alpha_destination: BlendFactor::One,
                        mask_red: true,
                        mask_green: true,
                        mask_blue: true,
                        mask_alpha: true,
                    })
                    .render_pass(subpass)
                    .build(gfx_queue.device().clone()).unwrap(),
            ) as Arc<_>
        };

        GraphicsSystem {
            gfx_queue,
            vertex_buffer,
            pipeline,
        }
    }

    fn get_system_num(&self) -> usize { 2 }

    fn pass<'b, 'p : 'b>(&self, pass_builder: &'b mut PassBuilder<'p>, gfx_system: GraphicsSystem){
        let dims = pass_builder.framebuffer.dimensions();

        let command_buffer = self.draw(
            gfx_system,
            [dims[0], dims[1]],
            pass_builder.system.normals_buffer.clone(),
            pass_builder.system.diffuse_buffer.clone(),
            pass_builder.system.specular_buffer.clone(),
            pass_builder.system.specular_buffer.clone(),
            pass_builder.screen_to_camera,
            pass_builder.to_camera_space
        );

        pass_builder.command_buffer_builder
            .as_mut().unwrap()
            .execute_commands(command_buffer).unwrap();
    }
}