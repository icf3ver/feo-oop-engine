//! Mesh that has a material
//! 
//! TODO
//! 
use vulkano::sync;

use {
    super::{material::Material, texture::Texture},
    crate::{
        shaders::fs_draw,
        components::{Normal, TextureIndex, Vertex}
    },
    std::{
        sync::Arc,
        collections::HashMap
    },
    vulkano::{
        buffer::{
            BufferUsage, 
            CpuAccessibleBuffer
        }, 
        device::Queue,
        sync::GpuFuture
    }
};

/// The visible mesh made up of triangles belonging to a gameobject.
#[derive(Clone)]
pub struct TriangleMesh {
    pub(crate) vertex_buffer: Option<Arc<CpuAccessibleBuffer<[Vertex]>>>, // make into buffer
    pub(crate) normal_buffer: Option<Arc<CpuAccessibleBuffer<[Normal]>>>,
    pub(crate) texture_indices_buffer: Option<Arc<CpuAccessibleBuffer<[TextureIndex]>>>,
    
    pub(crate) material: Option<(fs_draw::ty::Material, [Arc<Texture>; 4])>,
}

impl std::fmt::Debug for TriangleMesh {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TriangleMesh debug todo")
    }
}

impl TriangleMesh{
    /// Create a new empty TriangleMesh
    pub fn new_empty() -> Self{
        TriangleMesh {
            vertex_buffer: None,
            normal_buffer: None,
            texture_indices_buffer: None,
            material: None,
        }
    }

    /// Create a new TriangleMesh
    pub fn new(
            ordered_vertices: Vec<Vertex>,
            ordered_normals: Vec<Normal>,
            ordered_texture_indices: Vec<TextureIndex>,
            material: Arc<Material>,
            
            queue: Arc<Queue>) -> Self {

        TriangleMesh{
            vertex_buffer: Some(CpuAccessibleBuffer::from_iter(queue.device().clone(), BufferUsage::all(), false, ordered_vertices.iter().cloned()).unwrap()),
            normal_buffer: Some(CpuAccessibleBuffer::from_iter(queue.device().clone(), BufferUsage::all(), false, ordered_normals.iter().cloned()).unwrap()),
            texture_indices_buffer: Some(CpuAccessibleBuffer::from_iter(queue.device().clone(), BufferUsage::all(), false, ordered_texture_indices.iter().cloned()).unwrap()),
            material: Some(material.into_set(queue.clone())),
        }
    }

    /// Create a triangle mesh from a section of an obj file.
    pub fn from_obj_block<'a>(block: &[&str], mtls_hashmap: &mut HashMap<String, (Arc<Material>, Box<dyn GpuFuture>)>, vertex_data: (&Vec<Box<Vertex>>, &Vec<Box<TextureIndex>>, &Vec<Box<Normal>>), queue: Arc<Queue>) -> Result<Self, &'a str> {
        // Ordered mesh data
        let mut ordered_vertices = Vec::new();
        let mut ordered_normals = Vec::new();
        let mut ordered_texture_indices = Vec::new();

        let mut current_material: Arc<Material> = Arc::new(Material::default());
        
        block.iter().for_each(|line| {
            if !(*line).is_empty() {
                let mut e = line.split_whitespace();
                let ty: &str = e.next().unwrap();
                match &*ty {

                    //   Faces   //

                    "f" => {
                        let mut tris = Vec::new();
                        let mut i = 0;
                        for coord in &mut e{
                            i += 1;
                            if i > 3{ // not perfect but good enough for now
                                tris.push(tris[0]);
                                tris.push(tris[i - 2]);
                            }
                            tris.push(coord);
                        }

                        let mut vertex_fmt: i8 = -1;
                        let mut developing_normal: Vec<Vertex> = Vec::new();
                        for raw in tris{
                            let part = raw.split('/').collect::<Vec<&str>>();
                            
                            if vertex_fmt != part.len() as i8 {
                                if vertex_fmt == -1 {
                                    vertex_fmt = part.len() as i8;
                                }else {
                                    panic! ("Inconsistent face vertex format.")
                                }
                            }
                            
                            let position = *vertex_data.0[part[0].parse::<usize>().unwrap() - 1_usize].clone();

                            let texture_index = if vertex_fmt > 1 && !part[1].is_empty() {
                                    *vertex_data.1[part[1].parse::<usize>().unwrap() - 1_usize].clone()
                                } else {
                                    TextureIndex::new(0.0, 0.0)
                                };

                            if developing_normal.is_empty() && vertex_fmt == 3 && !part[2].is_empty() { // a false second case is a result of improper formatting
                                ordered_normals.push(*vertex_data.2[part[2].parse::<usize>().unwrap() - 1_usize].clone());
                            } else {
                                developing_normal.push(position);
                            }

                            ordered_vertices.push(position);
                            ordered_texture_indices.push(texture_index);
                        }

                        if developing_normal.len() > 2 {
                            let normal = Normal::calculate_normal(&developing_normal[0], &developing_normal[1], developing_normal.last().unwrap());

                            for _ in 0..developing_normal.len() {
                                ordered_normals.push(normal);
                            }
                        }
                    },
                    
                    //   Materials   //

                    "usemtl" => {
                        let key = e.next().expect("formatting error");
                        let (cm, fut ) = mtls_hashmap.remove(key).unwrap();
                        current_material = cm.clone();
                        if fut.queue().is_some() {
                            let _ = Arc::new(fut.then_signal_fence_and_flush().unwrap()).wait(None); // for now state does not matter                            
                        }
                        mtls_hashmap.insert(key.to_string(), (cm, sync::now(queue.device().clone()).boxed()));
                    },

                    //   Other   //

                    &_ => {
                        panic!("Formatting error"); 
                    }
                    
                    //   TODO: Other Geometry   //

                    #[allow(unreachable_patterns)] // fr now just ignore it TODO: fix

                    "line" => {
                        todo!();
                    },
                };
            }
            
        });

        Ok(Self::new(ordered_vertices, ordered_normals, ordered_texture_indices, current_material, queue))
    }
}
