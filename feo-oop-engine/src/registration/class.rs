use std::{collections::HashMap, sync::{Arc, RwLock}};

use crate::scene::game_object::GameObject;

#[derive(Debug, Clone, Copy)]
pub struct Class(u8);

impl Class{
    pub fn add_class(){
        // add class to class list
    }

    pub fn get_class(_class: &str) -> Result<Class, ()> {
        // get a class from the class list if it does not exist return Err
        Err(())
    }

    pub fn get_value(&self) -> String{
        // get the string value of a class
        String::from("")
    }
}

impl PartialEq for Class{
    fn eq(&self, other: &Self) -> bool {
        self.get_value() == other.get_value()
    }
}

pub struct ClassSystem<'a>{
    classes: HashMap<Vec<Arc<RwLock<dyn GameObject>>>, &'a str>,
}

impl ClassSystem {
    pub fn give_class (){
        todo!()
    }

    pub fn purge(){
        todo!()
    }
}