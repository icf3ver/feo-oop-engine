//! First Person View Camera GameObject that can capture a scene
//! 
//! TODO
//! 
use {
    super::{
        Camera,
        super::{
            GameObject,
            light::Light,
        }
    },
    crate::{
        registration::{
            relation::{
                Child, Parent,
                ParentWrapper
            },
            named::Named,
            id::ID
        },
        scripting::{
            Script,
            executor::Spawner,
            Scriptable, 
            globals::{
                EngineGlobals, 
                Global
            }
        },
        components::{
            triangle_mesh::TriangleMesh,
        },
        graphics::{
            Drawable,
            draw_pass_manager::DrawPassManager,
            lighting_pass_manager::LightingPassManager,
        },
        event::UserEvent,
        shaders::vs_draw,
    },
    feo_math::{
        linear_algebra::{
            vector3::Vector3,
            matrix4::Matrix4
        },
        utils::space::Space,
        rotation::quaternion::Quaternion
    },
    std::{
        any::Any,
        sync::{
            Arc, 
            RwLock
        },
        mem
    },
    winit::event::Event,
};

#[derive(Scriptable, GameObject, Parent, Child, Named, Drawable)]
#[camera]
pub struct FpvCamera{
    id: ID,
    name: String,
    parent: ParentWrapper,

    main: bool,

    offset: Option<Vector3<f32>>, // offset should be defined by subspace itself

    fov: i32,
    near_plane: f32,
    far_plane: f32,
    aspect_ratio: f32,

    pub subspace: Space,

    script: Option<Box<Script<Self>>>,

    children: Vec<Arc<RwLock<dyn GameObject>>>,
}

impl std::fmt::Debug for FpvCamera {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FpvCamera")
            .field("id", &self.id)
            .field("name", &self.name)
            .field("parent", &self.parent)
            .field("main", &self.main)
            .field("offset", &self.offset)
            .field("fov", &self.fov)
            .field("near_plane", &self.near_plane)
            .field("far_plane", &self.far_plane)
            .field("aspect_ratio", &self.aspect_ratio)
            .field("subspace", &self.subspace)
            .field("script", &self.script)
            .field("children", &self.children).finish()
    }
}

impl Clone for FpvCamera {
    fn clone(&self) -> Self {
        let id = self.id.get_system().take();
        FpvCamera{
            id,
            name: self.name.clone(),
            parent: self.parent.clone(),
            subspace: self.subspace,
            script: self.script.clone(),
            children: self.children.clone().into_iter().map(|_child| {
                // Dangerous
                todo!();
            }).collect::<Vec<Arc<RwLock<dyn GameObject>>>>(),
            main: self.main,
            offset: self.offset,
            fov: self.fov,
            near_plane: self.near_plane,
            far_plane: self.far_plane,
            aspect_ratio: self.aspect_ratio,
        }
    }
}

impl PartialEq for FpvCamera{
    fn eq(&self, other: &Self) -> bool {
        self.get_id() == other.get_id()
    }
}

impl FpvCamera {
    #[allow(clippy::too_many_arguments)]
    pub fn new( 
            name: Option<&str>,

            main: bool,

            parent: Option<Arc<RwLock<dyn GameObject>>>, // TODO: automatic not great use parent wrapper

            position: Option<Vector3<f32>>,
            rotation: Option<Quaternion<f32>>,
            scale_factor: Option<Vector3<f32>>,

            offset: Option<Vector3<f32>>, 
            
            fov: i32, 
            near_plane: f32, 
            far_plane: f32,
            aspect_ratio: f32,
            
            script: Option<Box<Script<Self>>>,
            
            engine_globals: EngineGlobals//&VulkanoEngine
            ) -> Result<Arc<RwLock<Self>>, &'static str> { // TODO: pass surface
        let id = engine_globals.id_system.take();
        let subspace = Space::new(position, rotation, scale_factor);
        
        Ok(Arc::new(RwLock::new( FpvCamera {
            name: match name {
                Some(name) => name.to_string(),
                None => String::from("fpv_camera_") + id.to_string().as_str()
            },
            id,
            parent: match parent {
                Some(game_object) => {
                    ParentWrapper::GameObject(game_object)
                },
                None => {
                    ParentWrapper::Scene(engine_globals.scene)
                }
            },

            main,
            offset,
            fov,
            near_plane,
            far_plane,
            aspect_ratio,
            subspace,

            script,

            children: Vec::new()
        })))
    }
}

impl Camera for FpvCamera {
    fn as_any(&self) -> &dyn Any { self }
    fn as_gameobject(&self) -> &dyn GameObject { self }
    
    fn cast_gameobject_arc_rwlock(&self, this: Arc<RwLock<dyn Camera>>) -> Arc<RwLock<dyn GameObject>> { 
        let this= Arc::into_raw(this).cast::<RwLock<Self>>();
        let this = unsafe { Arc::from_raw(this) };
        this as Arc<RwLock<dyn GameObject>>
    }

    fn is_main(&self) -> bool {
        self.main
    }
    
    fn get_z_step (&self, z_buffer_size: usize) -> f32 {
        ((self.far_plane / self.near_plane) * 0.5).powi(z_buffer_size as i32)
    }

    fn build_projection(&self) -> Matrix4<f32> {
        let half_h = self.near_plane * (self.fov as f32 * 0.5).tan();
        let half_w = half_h * self.aspect_ratio;

        Matrix4::new(
            [ self.near_plane / half_w,                       0.0,                                                                      0.0,                                                                            0.0],
            [                      0.0, self.near_plane / -half_h, /* <- flipped y to account for vulkano axes */                       0.0,                                                                            0.0],
            [                      0.0,                       0.0, -(self.near_plane + self.far_plane) / (self.far_plane - self.near_plane), (-2.0 * self.far_plane * self.near_plane) / (self.far_plane - self.near_plane)],
            [                      0.0,                       0.0,                                                                     -1.0,                                                                           0.0] 
        )
    }
  
    fn build_viewspace(&self) -> Matrix4<f32> {
        self.get_inversed_subspace().build()
    }

    fn create_uniforms(&self) -> vs_draw::ty::Camera {
        vs_draw::ty::Camera {
            to_view: self.build_viewspace().transpose().into(),
            view_to_screen: self.build_projection().transpose().into()
        }
    }
}