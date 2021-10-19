use std::sync::Arc;

use vulkano::{buffer::CpuAccessibleBuffer, device::Queue, framebuffer::{RenderPassAbstract, Subpass}, pipeline::GraphicsPipelineAbstract};

use crate::components::ScreenPos;

use super::{pass_builder::PassBuilder};

/// Contains the necessary facilities to build a Graphics system.
#[derive(Clone)]
pub struct GraphicsSystem {
    pub(crate) gfx_queue: Arc<Queue>,
    pub(crate) vertex_buffer: Arc<CpuAccessibleBuffer<[ScreenPos]>>,
    pub(crate) pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
}


/// Contains the necessary facilities to execute a Graphics system.
pub trait GraphicsSystemTrait: Send + Sync {
    /// Creates a new system
    fn new_system<L>(gfx_queue: Arc<Queue>, subpass: Subpass<L>) -> GraphicsSystem
    where Self: Sized, L: RenderPassAbstract + Sync + Send + 'static;

    /// Returns the index of the system.
    fn get_system_num(&self) -> usize;

    /// Executes the system
    fn pass<'b, 'p : 'b>(&self, pass_builder: &'b mut PassBuilder<'p>, gfx_system: GraphicsSystem);
}