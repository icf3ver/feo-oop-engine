use feo_math::{linear_algebra::vector3::Vector3, rotation::quaternion::Quaternion};

pub mod player;
pub mod pew;
pub mod spawner;

#[derive(Clone, Copy, Debug)]
pub enum MyEvent{
    NewPew(Vector3<f32>, Quaternion<f32>),
}