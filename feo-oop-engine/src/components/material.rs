//! GameObject material container
//! 
//! The only good mtl file documentation I found:
//! <http://paulbourke.net/dataformats/mtl/>
//! 
//! 
//! | illum | Interpretation                            |
//! |-------|-------------------------------------------|
//! | 0     | Color on and Ambient off                  |
//! | 1     | Color on and Ambient on                   |
//! | 2     | Highlight on                              |
//! | 3     | Reflection on and Ray trace on            |
//! | 4     | Transparency: Glass on                    |
//! |       | Reflection: Ray trace on                  |
//! | 5     | Reflection: Fresnel on and Ray trace on   |
//! | 6     | Transparency: Refraction on               |
//! |       | Reflection: Fresnel off and Ray trace on  |
//! | 7     | Transparency: Refraction on               |
//! |       | Reflection: Fresnel on and Ray trace on   |
//! | 8     | Reflection on and Ray trace off           |
//! | 9     | Transparency: Glass on                    |
//! |       | Reflection: Ray trace off                 |
//! | 10    | Casts shadows onto invisible surfaces     |

use std::{sync::Arc, collections::HashMap, fs};
use vulkano::{device::Queue, sync::{self, GpuFuture}};
use crate::{components::RGB, shaders::fs_draw};
use super::texture::Texture;


/// A material is a construct that describes the color and 
/// distortion of surface of a model.
#[derive(Debug, Clone)]
pub struct Material{
    name: String,
    illum: u8, // illumination model  
    
    kd: Option<RGB>, // diffuse color not rly an option
    ka: Option<RGB>, // ambient color
    ks: Option<RGB>, // specular color // TODO
    ke: Option<RGB>, // emissive color // TODO Unimplemented
    km: Option<f32>, // Bump strength // TODO
    ns: Option<f32>, // focus of specular highlights 0..1000
    ni: Option<f32>, // optical density / index of refraction 0..1000
    d: Option<f32>,  // dissolve / alpha transparency 0 (transparent) .. 1 (opaque)

    map_kd: Option<Arc<Texture>>,
    map_ka: Option<Arc<Texture>>,
    map_ks: Option<Arc<Texture>>,
    map_ke: Option<Arc<Texture>>,
    map_km: Option<Arc<Texture>>,
    map_ns: Option<Arc<Texture>>,
    // map_ni not sure if this would make sense
    map_d: Option<Arc<Texture>>,
    map_refl: Option<Arc<Texture>>,
    
    halo: bool,
}

impl Default for Material {
    fn default() -> Self {
        Material{
            name: String::from("default"),
            illum: 1,
            kd: Some(RGB::new(1.0, 0.0, 1.0)),
            ka: Some(RGB::new(1.0, 0.0, 1.0)),
            ks: None,
            ke: None,
            km: None,
            ns: None,
            ni: None,
            d: None,
            map_kd: None,
            map_ka: None,
            map_ks: None,
            map_ke: None,
            map_km: None,
            map_ns: None,
            map_d: None,
            map_refl: None,
            halo: false
        }
    }
}

impl Material {
    pub fn into_set(&self, queue: Arc<Queue>) -> (fs_draw::ty::Material, [Arc<Texture>; 4]) {
        let default_texture = Texture::default(queue);

        let (diffuse, diffuse_map): ([f32; 4], _) = match (self.kd, self.map_kd.clone()) {
            (Some(kd), None) => ([kd.r, kd.g, kd.b, 0.0], default_texture.clone()),
            (None, Some(map_kd)) => ([1.0, 1.0, 1.0, 1.0], map_kd),
            (Some(kd), Some(map_kd)) => ([kd.r, kd.g, kd.b, 1.0], map_kd),
            _ => panic!("Either Kd, map_Kd, or both must be defined")
        };

        let (ambient, ambient_map) = match (self.ka, self.map_ka.clone()) {
            (None, None) =>  ([0.0, 0.0, 0.0, 0.0], default_texture.clone()),
            (Some(ka), None) => ([ka.r, ka.g, ka.b, 1.0], default_texture.clone()),
            (None, Some(map_ka)) => ([1.0, 1.0, 1.0, 2.0], map_ka),
            (Some(ka), Some(map_ka)) => ([ka.r, ka.g, ka.b, 2.0], map_ka),
        };

        let (specular, specular_map, specular_highlight_focus_map) = match (self.ks, self.ns, self.map_ks.clone(), self.map_ns.clone()) { // TODO maps
            (Some(ks), Some(ns), Some(map_ks), Some(map_ns)) => ([ks.r, ks.g, ks.b, -(ns + 1001.0)], map_ks, map_ns),
            (Some(ks), Some(ns), Some(map_ks), None) => ([ks.r, ks.g, ks.b, -ns], map_ks, default_texture),
            (Some(ks), Some(ns), None, Some(map_ns)) => ([ks.r, ks.g, ks.b, ns + 1001.0], default_texture, map_ns),
            (Some(ks), Some(ns), None, None) => ([ks.r, ks.g, ks.b, ns], default_texture.clone(), default_texture),
            (None, Some(ns), Some(map_ks), Some(map_ns)) => ([1.0, 1.0, 1.0, -(ns + 1001.0)], map_ks, map_ns),
            (None, Some(ns), Some(map_ks), None) => ([1.0, 1.0, 1.0, -ns], map_ks, default_texture),
            (Some(ks), None, Some(map_ks), Some(map_ns)) => ([ks.r, ks.g, ks.b, 1001.0], map_ks, map_ns),
            (Some(ks), None, None, Some(map_ns)) => ([ks.r, ks.g, ks.b, 1001.0], default_texture, map_ns),
            (None, None, Some(map_ks), Some(map_ns)) => ([1.0, 1.0, 1.0, -1001.0], map_ks, map_ns),
            (_, None, _, None) | (None, _, None, _) => ([0.0, 0.0, 0.0, 0.0], default_texture.clone(), default_texture),
        };

        let other = [self.d.unwrap_or(1.0), self.ni.unwrap_or(0.0), 0.0, self.illum as f32];
        
        (
            fs_draw::ty::Material {
                diffuse,
                ambient,
                specular,
                other
            },
            [
                diffuse_map,
                ambient_map,
                specular_map,
                specular_highlight_focus_map,
            ]
        )
    }

    /// Parse an mtl file
    pub fn from_mtllib(path: &str, gfx_queue: Arc<Queue>) -> HashMap<String, (Arc<Self>, Box<dyn GpuFuture>)> {
        let content = fs::read_to_string(path).unwrap_or_else(|_| panic!("Something went wrong when trying to read {}.", path));

        let mut mtls = content.split("newmtl ");
        mtls.next();

        mtls.into_iter().map(|block| {
            let block = String::from("newmtl ") + block;
            Material::from_mtlblock(block.as_str(), path, gfx_queue.clone())
        }).collect::<HashMap<String, (Arc<Material>, Box<dyn GpuFuture>)>>()
    }

    /// Parse an mtl block.
    pub fn from_mtlblock(block: &str, path: &str, gfx_queue: Arc<Queue>) -> (String, (Arc<Material>, Box<dyn GpuFuture>)) {
        let mut lines = block.lines().filter(|s| !(*s).is_empty() && !s.starts_with('#') );

        let name = lines.next().unwrap_or_else(|| panic!("formatting error in {}", path)).split_whitespace().nth(1).unwrap().to_string();
        
        let mut ka: Option<RGB> = None;
        let mut kd: Option<RGB> = None;
        let mut ks: Option<RGB> = None;
        let mut ke: Option<RGB> = None;
        let mut km: Option<f32> = None;
        let mut ns: Option<f32> = None;
        let mut ni: Option<f32> = None;
        let mut d: Option<f32> = None; // unset

        let mut map_kd: Option<Arc<Texture>> = None;
        let mut map_ka: Option<Arc<Texture>> = None;
        let mut map_ks: Option<Arc<Texture>> = None;
        let mut map_ke: Option<Arc<Texture>> = None;
        let mut map_km: Option<Arc<Texture>> = None;
        let mut map_ns: Option<Arc<Texture>> = None;
        let mut map_d: Option<Arc<Texture>> = None;
        let mut map_refl: Option<Arc<Texture>> = None;

        let mut illum: Option<u8> = None; // unset

        let mut halo: bool = false;

        let mut future: Box<dyn GpuFuture> = sync::now(gfx_queue.device().clone()).boxed();

        for line in lines {
            let mut line_parts = line.split_whitespace();
            match line_parts.next().unwrap() {
                "Kd" => kd = Some(RGB::from_parts(line_parts).unwrap_or_else(|_| panic!("formatting error in {}", path))),
                "Ka" => ka = Some(RGB::from_parts(line_parts).unwrap_or_else(|_| panic!("formatting error in {}", path))),
                "Ks" => ks = Some(RGB::from_parts(line_parts).unwrap_or_else(|_| panic!("formatting error in {}", path))),
                "Ke" => ke = Some(RGB::from_parts(line_parts).unwrap_or_else(|_| panic!("formatting error in {}", path))),
                "Km" => km = Some(line_parts.next().unwrap_or_else(|| panic!("formatting error in {}", path)).parse::<f32>().unwrap_or_else(|_| panic!("formatting error in {}", path))),
                "Ns" => ns = Some(line_parts.next().unwrap_or_else(|| panic!("formatting error in {}", path)).parse::<f32>().unwrap_or_else(|_| panic!("formatting error in {}", path))),
                "Ni" => ni = Some(line_parts.next().unwrap_or_else(|| panic!("formatting error in {}", path)).parse::<f32>().unwrap_or_else(|_| panic!("formatting error in {}", path))),
                "d" => d = match line_parts.clone().count() {
                    1 => Some(line_parts.next().unwrap().parse::<f32>().unwrap_or_else(|_| panic!("formatting error in {}", path))), // halo length
                    2 => {
                        line_parts.next().unwrap().to_string();
                        halo = true;
                        Some(line_parts.next().unwrap().parse::<f32>().unwrap_or_else(|_| panic!("formatting error in {}", path)))
                    },
                    _ => panic!("formatting error in {}", path)
                }, // rmb other transparency type

                // maps
                "map_Kd" => {
                    let (new_map_kd, tex_future) = Texture::from_mtl_line(&mut line_parts, path, gfx_queue.clone()).unwrap();
                    map_kd = Some(new_map_kd);
                    future = future.join(tex_future).boxed();
                },
                "map_Ka" => {
                    let (new_map_ka, tex_future) = Texture::from_mtl_line(&mut line_parts, path, gfx_queue.clone()).unwrap();
                    map_ka = Some(new_map_ka);
                    future = future.join(tex_future).boxed();
                },
                "map_Ks" => {
                    let (new_map_ks, tex_future) = Texture::from_mtl_line(&mut line_parts, path, gfx_queue.clone()).unwrap();
                    map_ks = Some(new_map_ks);
                    future = future.join(tex_future).boxed();
                },
                "map_Ke" => {
                    let (new_map_ke, tex_future) = Texture::from_mtl_line(&mut line_parts, path, gfx_queue.clone()).unwrap();
                    map_ke = Some(new_map_ke);
                    future = future.join(tex_future).boxed();
                },
                "map_Km" | "map_Bump" | "map_bump" => {
                    let (new_map_km, tex_future) = Texture::from_mtl_line(&mut line_parts, path, gfx_queue.clone()).unwrap();
                    map_km = Some(new_map_km);
                    future = future.join(tex_future).boxed();
                },
                "map_Ns" => {
                    let (new_map_ns, tex_future) = Texture::from_mtl_line(&mut line_parts, path, gfx_queue.clone()).unwrap();
                    map_ns = Some(new_map_ns);
                    future = future.join(tex_future).boxed();
                },
                "map_d" => {
                    let (new_map_d, tex_future) = Texture::from_mtl_line(&mut line_parts, path, gfx_queue.clone()).unwrap();
                    map_d = Some(new_map_d);
                    future = future.join(tex_future).boxed();
                },
                "map_refl" | "refl" => {
                    let (new_map_refl, tex_future) = Texture::from_mtl_line(&mut line_parts, path, gfx_queue.clone()).unwrap();
                    // TODO type
                    map_refl = Some(new_map_refl);
                    future = future.join(tex_future).boxed();
                },

                // required
                "illum" => illum = Some(line_parts.next().unwrap_or_else(|| panic!("formatting error in {}", path)).parse::<i8>().unwrap_or_else(|_| panic!("formatting error in {}", path)) as u8),

                // unsupported
                a => panic!("formatting error in {}. {} is not supported", path, a),
            };
        }

        let mut material = Material{
            name: name.clone(),
            illum: 11,
            kd,
            ka,
            ks,
            ke,
            km,
            ns,
            ni,
            d,

            map_kd,
            map_ka,
            map_ks,
            map_ke,
            map_km,
            map_ns,
            map_d,
            map_refl,
            
            halo,
        };

        material.set_illumination_model(illum).unwrap_or_else(|_| panic!("formatting error in {}", path));

        (name, (Arc::new(material), future))
    }

    pub fn set_illumination_model(&mut self, illum: Option<u8>) -> Result<(), ()> {
        match illum {
            Some(illum) => {
                self.illum = match illum {
                    0 => 0,
                    1 if self.ka.is_some() || self.map_ka.is_some() => 1,
                    2 if (self.ks.is_some() || self.map_ks.is_some()) && self.ns.is_some() => 2,
                    3 /* TODO */ => 3,
                    4 /* TODO */ => 4,
                    5 /* TODO */ => 5,
                    6 /* TODO */ => 6,
                    7 /* TODO */ => 7,
                    8 /* TODO */ => 8,
                    9 /* TODO */ => 9,
                    10 /* TODO */ => 10,
                    _ => return Err(())
                };
                Ok(())
            },
            None => { // chose the best model
                todo!()
            }
        }
    }
}
