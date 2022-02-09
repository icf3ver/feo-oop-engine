//! A construct within which a world can exist.
//! 
//! TODO
//! 
pub mod game_object;

use std::any::Any;

use vulkano::{swapchain::SwapchainAcquireFuture, sync::GpuFuture};
use winit::window::Window;

use crate::graphics::{draw_pass_manager::DrawPassManager, frame_system::FrameSystem, lighting_pass_manager::LightingPassManager};

use game_object::camera::Camera;

use {
    crate::{
        scene::{
            game_object::GameObject,
        }, 
        registration::{
            relation::{Parent, ParentWrapper},
            named::Named
        },
        scripting::executor::Spawner,
        components::{
            triangle_mesh::TriangleMesh 
        },
        event::UserEvent,
        graphics::{
            Drawable,
        },
    },
    feo_math::{
        linear_algebra::matrix4::Matrix4,
        utils::space::Space
    },
    std::{
        mem,
        sync::{Arc, RwLock}
    },
    winit::event::Event
};

/// A scene in which GameObjects can exist.
#[derive(Clone, Parent, Drawable, Debug)]
pub struct Scene {
    pub worldspace: Space,
    pub children: Vec<Arc<RwLock<dyn GameObject>>>,
    pub main_camera: Option<Arc<RwLock<dyn Camera>>>,
}

impl Scene {
    const NAME: &'static str = "Scene";

    /// Create a new Scene.
    /// # Arguments
    /// * `worldspace` - The mathematical space off of which you will work.
    /// # Examples
    /// ```no_run
    /// # use feo_oop_engine::scene::Scene;
    /// let scene = Scene::new(None); // Creates a new scene with a default worldspace.
    /// ```
    pub fn new(worldspace: Option<Space>) -> Arc<RwLock<Self>>{
        Arc::new(RwLock::new(Scene{
            worldspace: worldspace.unwrap_or_else(|| Space::new(None, None, None)),
            children: Vec::new(),
            main_camera: None,
        }))
    }
    
    /// Sets the main camera of the Scene.
    /// # Arguments
    /// * `main_camera` - The new main camera.
    pub fn set_main_camera(&mut self, main_camera: Arc<RwLock<impl Camera>>){
        self.main_camera = Some(main_camera as Arc<RwLock<dyn Camera>>);
        // let temp = main_camera.clone();
        // let temp = temp.read().unwrap();
        // self.add_child(temp.cast_gameobject_arc_rwlock(main_camera.clone())); // require not do finish impl for everything tree structure rmb
        // unsafe{self.replace_child( main_camera.clone(), main_camera.clone()).unwrap();}
    }

    /// \[backend\] Spawns the core scripts. i.e. Spawn start and frame. 
    pub fn spawn_script_cores(&self, spawner: Spawner){
        self.children.clone().into_iter().for_each(|game_object| {
            let game_object_template = game_object.clone();
            game_object_template.write().unwrap().spawn_script_core(game_object, spawner.clone());
        }); // Make sure camera is added to the scene or within the game_object tree otherwise Its scripts wont be run
        // let main_camera_template = self.main_camera.clone().expect("No camera defined");
        // let main_camera_gameobject = main_camera_template.read().unwrap().cast_gameobject_arc_rwlock(main_camera_template.clone());
        // main_camera_template.clone().write().unwrap().spawn_script_core( main_camera_gameobject, spawner);
    }
    
    /// \[backend\] Spawns the event_handler of the scripts.
    pub fn spawn_script_handlers(&self, spawner: Spawner, event: Event<'static, UserEvent<Arc<dyn Any + Send + Sync>>>){
        self.children.clone().into_iter().for_each(|game_object| {
            game_object.clone().write().unwrap().spawn_script_handler(game_object, spawner.clone(), event.clone());
        }); // Make sure camera is added to the scene or within the game_object tree otherwise Its scripts wont be run
        // let main_camera_template = self.main_camera.clone().expect("No camera defined");
        // let main_camera_gameobject = main_camera_template.read().unwrap().cast_gameobject_arc_rwlock(main_camera_template.clone());
        // let write_lock = main_camera_template.clone();
        // let mut write_lock = write_lock.write().unwrap();
        // write_lock.spawn_script_handler(main_camera_gameobject, spawner, event);
    }

    /// \[backend\] Renders the scene.
    #[inline]
    pub fn render(&self,
            this: Arc<RwLock<Scene>>,
            frame_system: &mut FrameSystem,
            image_num: usize,
            acquire_future: SwapchainAcquireFuture<Window>, 
            previous_frame_end: &mut Option<Box<dyn GpuFuture>> ) -> Box<dyn GpuFuture> {
        
        frame_system.draw_pass_manager.clear();
        frame_system.lighting_pass_manager.clear();

        self.load_into_managers(ParentWrapper::Scene(this), &mut frame_system.draw_pass_manager, &mut frame_system.lighting_pass_manager);

        let main_camera = self.main_camera.clone().expect("No camera defined");
        frame_system.draw_pass_manager.recreate_camera_set(main_camera.clone());

        let main_camera_read = main_camera.read().unwrap();
        let future = previous_frame_end.take().unwrap().join(acquire_future);
        let mut builder = frame_system.pass_builder(
            future, 
            image_num,
            main_camera_read.build_projection().inverse().transpose(),
            main_camera_read.get_inversed_subspace()
        );

        builder.build()
    }

    #[inline]
    pub fn build_space(&mut self) -> Matrix4<f32>{
        self.worldspace.build()
    }
}

impl Named for Scene {
    fn get_name(&self) -> &str { Scene::NAME }
}