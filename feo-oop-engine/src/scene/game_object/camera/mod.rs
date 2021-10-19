use std::fmt;

pub mod fpv_camera;

use {
    super::GameObject,
    crate::shaders::vs_draw,
    feo_math::linear_algebra::matrix4::Matrix4,
    std::{
        any::Any,
        sync::{Arc, RwLock},
    }
};

/// For structs capable of projecting the scene to the Screen from a view-space.
pub trait Camera: CameraClone + GameObject {
    fn as_any(&self) -> &dyn Any;
    fn as_gameobject(&self) -> &dyn GameObject;
    fn cast_gameobject_arc_rwlock(&self, this: Arc<RwLock<dyn Camera>>) -> Arc<RwLock<dyn GameObject>>; 

    fn is_main(&self) -> bool;
    
    fn get_z_step(&self, z_buffer_size: usize) -> f32;

    fn build_projection(&self) -> Matrix4<f32>;
    fn build_viewspace(&self) -> Matrix4<f32>;
    fn create_uniforms(&self) -> vs_draw::ty::Camera;
}

impl PartialEq for dyn Camera {
    fn eq(&self, other: &Self) -> bool {
        self.get_id() == other.get_id()
    }
}

impl fmt::Debug for dyn Camera {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.get_id(), self.get_name())
    }
}

/// Allows Box<dyn Camera> to be cloneable.
pub trait CameraClone {
    fn clone_camera(&self) -> Box<dyn Camera>;
}

impl<T> CameraClone for T where T: 'static + Camera + Clone {
    fn clone_camera(&self) -> Box<dyn Camera> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn Camera> {
    fn clone(&self) -> Self {
        self.clone_camera()
    }
}