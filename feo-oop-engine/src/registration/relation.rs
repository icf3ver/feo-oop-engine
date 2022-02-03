//! Systems for defining relationships between objects. IE Parent/Child
//! 
//! TODO
//! 
use {
    crate::{
        scene::{
            game_object::GameObject, 
            Scene,
        },
        registration::named::Named
    },
    std::sync::{Arc, RwLock}
};

#[derive(Clone, Debug)]
pub enum ParentWrapper{ // make into a trait object and add as_parent
    GameObject(Arc<RwLock<dyn GameObject>>),
    Scene(Arc<RwLock<Scene>>)
}

pub trait Parent: Named {
    fn get_children(&self) -> Vec<Arc<RwLock<dyn GameObject /* dyn Child */>>>; // use dyn Child here and make gameobject castable to child using a new as_child() do same for parent except with as_parent and dyn Parent and not enum
    fn add_child(&mut self, child: Arc<RwLock<dyn GameObject>>);
    
    // unsafe because scripts running on child access (->) child -> parent or children -> new object or nothing 
    // alternative use the swap feature
    unsafe fn replace_child(&mut self, old: Arc<RwLock<dyn GameObject>>, new: Arc<RwLock<dyn GameObject>>) -> Result<(), ()>; // ensure the child extends child and set its parent as well
    unsafe fn remove_child(&mut self, child: Arc<RwLock<dyn GameObject>>) -> Result<(), ()>;

    fn get_child_by_name(&self, name: &str) -> Result<Arc<RwLock<dyn GameObject>>, &str> {
        let mut result = Err("No child with that name was found.");
        self.get_children().into_iter().for_each( |child|
            if child.read().unwrap().get_name() == name {
                match result {
                    Ok(_) => panic!("Two children share the same name"),
                    Err(_) => { result = Ok(child); }
                }
            }
        );
        result
    }
}

pub trait Child: Named {
    fn get_parent(&self) -> ParentWrapper; // await rmb

    // unsafe because you do not want to set
    unsafe fn set_parent(&mut self, parent: ParentWrapper);
}