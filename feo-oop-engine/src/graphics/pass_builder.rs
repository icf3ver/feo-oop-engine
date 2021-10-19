
use std::sync::Arc;

use vulkano::{command_buffer::{AutoCommandBufferBuilder, SubpassContents, pool::standard::StandardCommandPoolBuilder}, framebuffer::FramebufferAbstract, sync::GpuFuture};
use feo_math::{linear_algebra::matrix4::Matrix4, utils::space::Space};
use super::frame_system::FrameSystem;

/// Represents the active process of rendering a frame.
///
/// This struct mutably borrows the `FrameSystem`.
pub struct PassBuilder<'p> {
    pub(crate) system: &'p mut FrameSystem,

    // Future to wait upon before the main rendering.
    pub(crate) before_main_cb_future: Option<Box<dyn GpuFuture>>,
    // Framebuffer that was used when starting the render pass.
    pub(crate) framebuffer: Arc<dyn FramebufferAbstract + Send + Sync>,
    // The command buffer builder that will be built during the lifetime of this object.
    pub(crate) command_buffer_builder: Option<AutoCommandBufferBuilder<StandardCommandPoolBuilder>>,
    // Matrix that converts screen coordinates and depth buffer values to coordinates in camera space
    pub(crate) screen_to_camera: Matrix4<f32>,
    // Matrix that converts ident to camera space
    pub(crate) to_camera_space: Space,
}

impl<'p> PassBuilder<'p>{
    /// Returns an enumeration containing the next pass of the rendering.
    pub fn build/*<'b>*/(&/*'b*/ mut self) -> Box<dyn GpuFuture> {
        // passes
    //     self.draw_pass();
    //     self.lighting_pass();
    //     self.render_pass()
    // }

    // #[inline]
    // fn draw_pass<'b>(&'b mut self) {
        let command_buffer = self.system.draw_pass_manager.draw();
        self.command_buffer_builder.as_mut().unwrap()
            .execute_commands(command_buffer).unwrap();
    // }
    
    // #[inline]
    // fn lighting_pass<'b>(&'b mut self) {
        self.command_buffer_builder.as_mut().unwrap()
            .next_subpass(SubpassContents::SecondaryCommandBuffers).unwrap();
        
        let lighting_pass_manager = self.system.lighting_pass_manager.clone();
        lighting_pass_manager.draw(self);
    // }
    
    // #[inline]
    // fn render_pass<'b>(&'b mut self) -> Box<dyn GpuFuture> {
        self.command_buffer_builder
            .as_mut()
            .unwrap()
            .end_render_pass()
            .unwrap();
        let command_buffer = self.command_buffer_builder.take().unwrap().build().unwrap();
        
        // Extract `before_main_cb_future` and append the command buffer execution to it.
        let after_main_cb = self
            .before_main_cb_future.take().unwrap()
            .then_execute(self.system.gfx_queue.clone(), command_buffer).unwrap();
        // We obtain `after_main_cb`, which we give to the user.
        Box::new(after_main_cb)
    }
}