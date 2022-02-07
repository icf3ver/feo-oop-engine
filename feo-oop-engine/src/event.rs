//! Engine events and user defined events container.
//! 
use std::sync::{Arc, RwLock};
use crate::scene::game_object::GameObject;
use std::fmt::Display;

/// A wrapper for winit events.
/// 
/// This enum allows for the definition of custom 
/// events with the UserEvent enumeration constant.
#[derive(Debug, Clone)]
pub enum UserEvent<T: 'static + Clone + Send + Sync>{
    RebuildSwapchain,
    
    Collision(Arc<RwLock<dyn GameObject>>, Arc<RwLock<dyn GameObject>>),

    WinitEvent(winit::event::Event<'static, Box<UserEvent<T>>>),
    UserEvent(T),
    None,
}

pub trait Error: Sized + Display {}

impl<T: Sized + Display> Error for T {}