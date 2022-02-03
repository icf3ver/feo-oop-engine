//! Engine/Game objects that can exist within a scene.
//! 
//! TODO: explain OOP here
//! 
use core::fmt;

pub mod light;
pub mod camera;
pub mod group;
pub mod obj;

use {
    self::{
        camera::Camera,
        light::Light,
    },

    crate::{
        registration::{
            relation::{Child, Parent},
            id::ID,
        },
        scripting::Scriptable,
        graphics::{
            Drawable
        },
    },
    feo_math::{
        utils::space::Space,
    },
    std::{
        any::Any,
        sync::{
            Arc, 
            RwLock
        }
    },
};

// A construct that exists in the game.
pub trait GameObject: 
        GameObjectBoxClone +
        Scriptable + 
        Drawable + 
        Parent +
        Child + 
        Any + 'static + 
        Send + Sync {
    fn as_any(&self) -> &dyn Any;
    fn cast_camera_arc_rwlock(&self, this: Arc<RwLock<dyn GameObject>>) -> Result<Arc<RwLock<dyn Camera>>, ()>; 
    fn cast_light_arc_rwlock(&self, this: Arc<RwLock<dyn GameObject>>) -> Result<Arc<RwLock<dyn Light>>, ()>; 

    fn get_id(&self) -> ID;
    fn get_subspace(&self) -> Space;
    fn get_inversed_subspace(&self) -> Space;
}

impl PartialEq for dyn GameObject{
    fn eq(&self, other: &Self) -> bool {
        self.get_id() == other.get_id()
    }
}

impl fmt::Debug for dyn GameObject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.get_id(), self.get_name())
    }
}

/// Allows Box<dyn GameObject> to be clonable
pub trait GameObjectBoxClone {
    fn clone_game_object(&self) -> Box<dyn GameObject>;
}

impl<T> GameObjectBoxClone for T where T: 'static + GameObject + Clone {
    fn clone_game_object(&self) -> Box<dyn GameObject> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn GameObject> {
    fn clone(&self) -> Self {
        self.clone_game_object()
    }
}