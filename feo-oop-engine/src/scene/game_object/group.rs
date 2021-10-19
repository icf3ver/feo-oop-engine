use {
    super::{
        GameObject,
        camera::Camera,
        light::Light,
    },
    crate::{
        registration::{
            relation::{
                Child, Parent,
                ParentWrapper
            },
            named::Named,
            id::ID,
        },
        scripting::{
            Scriptable, 
            globals::{EngineGlobals, Global},
            Script,
            executor::Spawner,
        },
        graphics::{
            Drawable,
            draw_pass_manager::DrawPassManager,
            lighting_pass_manager::LightingPassManager,
        },
        event::UserEvent,
        components::triangle_mesh::TriangleMesh
    },
    std::{
        any::Any,
        mem, 
        sync::{Arc, RwLock}
    },
    feo_math::{
        utils::space::Space, 
        rotation::quaternion::Quaternion,
        linear_algebra::vector3::Vector3
    },
    winit::event::Event,
};

#[derive(Scriptable, Drawable, GameObject, Child, Parent, Named)]
pub struct Group {
    pub id: ID,
    pub name: String,
    pub parent: ParentWrapper,

    pub subspace: Space,

    pub visible: bool,

    pub script: Option<Box<Script<Self>>>,

    pub children: Vec<Arc<RwLock<dyn GameObject>>>
}

impl std::fmt::Debug for Group {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Group")
            .field("id", &self.id)
            .field("name", &self.name)
            .field("parent", &self.parent)
            .field("subspace", &self.subspace)
            .field("visible", &self.visible)
            .field("script", &self.script)
            .field("children", &self.children).finish()
    }
}

impl Clone for Group {
    fn clone(&self) -> Self {
        let id = self.id.get_system().take();
        Group{
            id,
            name: self.name.clone(),
            parent: self.parent.clone(),
            visible: self.visible,
            subspace: self.subspace,
            script: self.script.clone(),
            children: self.children.clone().into_iter().map(|_child| {
                // Dangerous
                todo!();
            }).collect::<Vec<Arc<RwLock<dyn GameObject>>>>(),
        }
    }
}

impl PartialEq for Group{
    fn eq(&self, other: &Self) -> bool {
        self.get_id() == other.get_id()
    }
}

impl Group {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
            name: Option<&str>,

            parent: Option<Arc<RwLock<dyn GameObject>>>,
        
            position: Option<Vector3<f32>>, 
            rotation: Option<Quaternion<f32>>,
            scale_factor: Option<Vector3<f32>>,

            visible: bool,

            engine_globals: EngineGlobals,
            script: Option<Box<Script<Self>>>) -> Arc<RwLock<Self>> {

        let id = engine_globals.id_system.take();
    
        Arc::new(RwLock::new(Group{
            name: match name {
                Some(name) => name.to_string(),
                None => String::from("group_") + id.to_string().as_str()
            },
            id,
            parent: match parent {
                Some(game_object) => ParentWrapper::GameObject(game_object),
                None => ParentWrapper::Scene(engine_globals.scene)
            },
            subspace: Space::new(position, rotation, scale_factor),
            visible,
            script,
            children: Vec::new()
        }))
    }
}