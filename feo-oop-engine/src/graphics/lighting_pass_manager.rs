use std::{sync::{Arc, RwLock}};
use vulkano::{device::Queue, framebuffer::{RenderPassAbstract, Subpass}};
use crate::scene::game_object::light::{Light, ambient_light::AmbientLight, directional_light::DirectionalLight, point_light::PointLight};
use super::{graphics_system::GraphicsSystem, pass_builder::PassBuilder};
use crate::graphics::graphics_system::GraphicsSystemTrait;

/// Manages the differed lighting passes.
#[derive(Clone)]
pub struct LightingPassManager{
    pub(crate) lights: Vec<Arc<RwLock<dyn Light>>>,
    light_systems: Vec<GraphicsSystem>,
}

impl LightingPassManager {
    /// Creates a LightingPassManager.
    pub fn new<R>(gfx_queue: Arc<Queue>, subpass: Subpass<R>) -> LightingPassManager
    where R: RenderPassAbstract + Send + Sync + Clone + 'static {
        let light_systems = vec![
            AmbientLight::new_system(gfx_queue.clone(), subpass.clone()),
            DirectionalLight::new_system(gfx_queue.clone(), subpass.clone()),
            PointLight::new_system(gfx_queue, subpass),
        ];
        LightingPassManager {
            lights: Vec::new(),
            light_systems,
        }
    }

    /// Resets the LightingPassManager 
    #[inline]
    pub fn clear(&mut self){
        // TODO: Use an iterator. You only need to access the lights once before needing to access
        // them again in the current system.
        self.lights = Vec::new(); 
    }

    /// Builds a secondary command buffer that draws the triangle on the current subpass.
    pub fn draw<'b>(&self, pass_manager: &'b mut PassBuilder) {
        self.lights.clone().into_iter().for_each(|light| {
            let light = light.read().unwrap();
            light.pass(pass_manager, self.light_systems[light.get_system_num()].clone())
        });
    }
}