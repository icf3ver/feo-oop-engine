//! Engine events and user defined events container.
//! 
//! TODO
//! 
use std::sync::{Arc, RwLock};
use crate::scene::game_object::GameObject;
use winit::event::Event;
use std::fmt::Display;

#[derive(Debug, Clone)]
pub enum UserEvent<T: 'static + Clone + Send + Sync>{ 
    RebuildSwapchain,
    
    Collision(Arc<RwLock<dyn GameObject>>, Arc<RwLock<dyn GameObject>>),

    WinitEvent(Event<'static, Box<UserEvent<T>>>),
    UserEvent(T),
    None,
}

pub trait Error: Sized + Display {}

impl<T: Sized + Display> Error for T {}