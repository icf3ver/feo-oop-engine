pub mod directional_light;
pub mod ambient_light;
pub mod point_light;

use std::{any::Any, sync::{Arc, RwLock}};

use crate::graphics::graphics_system::GraphicsSystemTrait;

use super::GameObject;

/// For structs capable of lighting up a scene.
pub trait Light: LightClone + GameObject + GraphicsSystemTrait {
    fn as_any(&self) -> &dyn Any;
    fn as_gameobject(&self) -> &dyn GameObject;
    fn cast_gameobject_arc_rwlock(&self, this: Arc<RwLock<dyn Light>>) -> Arc<RwLock<dyn GameObject>>;
}

/// Allows Box<dyn Light> to be cloneable.
pub trait LightClone {
    fn clone_camera(&self) -> Box<dyn Light>;
}

impl<T> LightClone for T where T: 'static + Light + Clone {
    fn clone_camera(&self) -> Box<dyn Light> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn Light> {
    fn clone(&self) -> Self {
        self.clone_camera()
    }
}