//! Constructs that let you interface with the engine to safely swap and delete GameObjects.
//! 
//! TODO
//! 
use {
    crate::{
        scene::game_object::GameObject, 
        registration::id::ID
    },
    std::sync::{Arc, RwLock}
};

pub enum Swap{
    SwapParent(
        ID, // replace thing with ID 
        Arc<RwLock<dyn GameObject>>  // with this
    ), // replaces the parent object but keeps the child objects
    SwapFull(
        ID, // replace thing with ID 
        Arc<RwLock<dyn GameObject>>  // with this 
    ), // replaces the object and all its child objects
    Delete(ID), // Deletes the object with ID 
    None // don't swap
}

impl Swap{
    pub fn get_id(&self) -> Result<&ID, &'static str> {
        match self{ 
            Swap::SwapParent(id, _) => Ok(id),
            Swap::SwapFull(id, _) => Ok(id),
            Swap::Delete(id) => Ok(id),
            Swap::None => Err("None type has no ID")
        }
    }
}