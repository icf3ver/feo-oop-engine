//! Components used by game objects

pub mod texture;
pub mod material;
pub mod triangle_mesh;
use std::{iter::FromIterator, num::ParseFloatError};

use feo_math::linear_algebra::vector3::Vector3;

// all in one TODO
// #[derive(Default, Debug, Copy, Clone)]
// pub struct VertexV2 {
//     pub position: [f32; 3],
//     pub normal: [f32; 3],
//     pub texture_index: [f32; 2],
// }
// vulkano::impl_vertex!(VertexV2, position, normal, texture_index);

#[derive(Default, Debug, Copy, Clone)]
pub struct ScreenPos {
    pub position: [f32; 2],
}
vulkano::impl_vertex!(ScreenPos, position);

#[derive(Default, Debug, Copy, Clone)]
pub struct Vertex{
    pub position: [f32; 3],
}
vulkano::impl_vertex!(Vertex, position);

impl Vertex {
    pub fn new(x: f32, y: f32, z: f32) -> Self { // rmb w
        // rmb color  find out what thats about
        // maybe vertex paint?
        Vertex{
            position: [x, y, z]
        }
    }
   
    // In my math library I sometimes use vectors as points but thats technically incorrect although it works and gets the job done 
    pub fn into_vector3(from: &Self, to: &Self) -> Vector3<f32> { // rmb w
        let to: Vector3<f32> = (*to).into();
        let from: Vector3<f32> = (*from).into();
        to - from
    }
}

impl From<Vertex> for Vector3<f32> {
    fn from(other: Vertex) -> Vector3<f32> {
        Vector3::from(other.position)
    }
}

#[derive(Default, Debug, Copy, Clone)]
pub struct Normal {
    pub normal: (f32, f32, f32)
}
vulkano::impl_vertex!(Normal, normal);

impl Normal {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Normal{
            normal: (x, y, z)
        }
    }
    
    // possible due to counterclockwise ordering
    pub fn calculate_normal(a: &Vertex, b: &Vertex, c: &Vertex) -> Self { // TODO: use & everywhere
        Normal::from(Vector3::<f32>::cross_product(
            Vertex::into_vector3(a, b), 
            Vertex::into_vector3(a, c))
        ) // TODO: right hand rule comment
    }
}

impl From<Vector3<f32>> for Normal {
    fn from(other: Vector3<f32>) -> Self {
        Normal{ normal: (other.0, other.1, other.2)}
    }
}

#[derive(Default, Debug, Copy, Clone)]
pub struct TextureIndex{
    pub texture_index: [f32; 2],
}
vulkano::impl_vertex!(TextureIndex, texture_index);

impl TextureIndex { // double check
    pub fn new(x: f32, y: f32) -> Self {
        TextureIndex{
            texture_index: [x, y]
        }
    }
    pub fn default() -> Self {
        TextureIndex{
            texture_index: [0.0, 0.0]
        }
    }
}

#[derive(Default, Debug, Copy, Clone)]
pub struct RGB { // actually use parts r g and b and add a to arr method
    pub r: f32,
    pub g: f32,
    pub b: f32,
}
struct CollectRGB(pub Result<RGB, ()>);

impl RGB {
    pub fn from_parts<'a, I>(parts_iter: I) -> Result<Self, ()> 
    where I: IntoIterator<Item = &'a str> {
        parts_iter.into_iter().map(|str| str.parse::<f32>()).collect::<CollectRGB>().0
    }
    pub fn new(r: f32, g: f32, b: f32) -> Self {
        RGB { r, g, b }
    }
}

impl FromIterator<Result<f32, ParseFloatError>> for CollectRGB{
    fn from_iter<T: IntoIterator<Item = Result<f32, ParseFloatError>>>(iter: T) -> Self {
        let mut iter = iter.into_iter();
        if let (Some(Ok(r)), Some(Ok(g)), Some(Ok(b)), None) = (iter.next(), iter.next(), iter.next(), iter.next()) {
            CollectRGB(Ok(RGB::new(r, g, b)))
        } else {
            CollectRGB(Err(())) // formatting error
        }
    }
}

impl From<RGB> for [f32; 3]{
    fn from(other: RGB) -> [f32; 3] {
        [other.r, other.g, other.b]
    }
}
