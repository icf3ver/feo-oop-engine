use std::sync::{Arc, RwLock};
use feo_math::{linear_algebra::vector3::Vector3, rotation::quaternion::Quaternion};
use feo_oop_engine::{scene::game_object::GameObject, scripting::globals::Global};

pub mod player;
pub mod pew;
pub mod pew_spawner;
pub mod enemy;
pub mod enemy_spawner;

#[derive(Clone, Copy, Debug)]
pub enum MyEvent{
    NewPew(Vector3<f32>, Quaternion<f32>),
}


/// Global for passing forward target objects
#[derive(Clone, Debug, Global)]
pub struct TargetGlobal{
    pub target: Arc<RwLock<dyn GameObject>>,
}