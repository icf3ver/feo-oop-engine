use std::sync::Arc;
use vulkano::{command_buffer::{AutoCommandBufferBuilder, SubpassContents}, device::Queue, format::Format, framebuffer::Framebuffer, framebuffer::{FramebufferAbstract, RenderPassAbstract, Subpass}, image::AttachmentImage, image::ImageUsage, image::{ImageViewAbstract, SwapchainImage, view::ImageView}, sync::GpuFuture};
use feo_math::{linear_algebra::{matrix4::Matrix4}, utils::space::Space};
use winit::window::Window;
use super::{draw_pass_manager::DrawPassManager, lighting_pass_manager::LightingPassManager, pass_builder::PassBuilder};

/// System that contains the necessary facilities for rendering a single frame.
pub struct FrameSystem {
    pub(crate) gfx_queue: Arc<Queue>,

    render_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
    framebuffers: Vec<Arc<dyn FramebufferAbstract + Send + Sync>>,

    pub(crate) diffuse_buffer: Arc<ImageView<Arc<AttachmentImage>>>,
    pub(crate) normals_buffer: Arc<ImageView<Arc<AttachmentImage>>>,
    pub(crate) ambient_buffer: Arc<ImageView<Arc<AttachmentImage>>>,
    pub(crate) specular_buffer: Arc<ImageView<Arc<AttachmentImage>>>,
    pub(crate) depth_buffer: Arc<ImageView<Arc<AttachmentImage>>>,
    
    // pub(crate) light_depth_buffer: Arc<ImageView<Arc<AttachmentImage>>>,
    
    pub(crate) lighting_pass_manager: LightingPassManager,
    pub(crate) draw_pass_manager: DrawPassManager,
}

impl FrameSystem {
    /// Creates and initializes a new frame system.
    pub fn new(gfx_queue: Arc<Queue>, final_output_format: Format, dims: [u32; 2]) -> FrameSystem {
        let render_pass = Arc::new(
            vulkano::ordered_passes_renderpass!(gfx_queue.device().clone(),
                attachments: {
                    // The final rendering
                    final_color: {
                        load: Clear,
                        store: Store,
                        format: final_output_format,
                        samples: 1,
                    },
                    // Will be bound to `self.normals_buffer`.
                    normals: {
                        load: Clear,
                        store: DontCare,
                        format: Format::R16G16B16A16Sfloat,
                        samples: 1,
                    },
                    // Will be bound to `self.diffuse_buffer`.
                    diffuse: {
                        load: Clear,
                        store: DontCare,
                        format: Format::A2B10G10R10UnormPack32,
                        samples: 1,
                    },
                    // ambient reflected
                    ambient: {
                        load: Clear,
                        store: DontCare,
                        format: Format::A2B10G10R10UnormPack32,
                        samples: 1,
                    },
                    // specular reflected
                    specular: {
                        load: Clear,
                        store: DontCare,
                        format: Format::A2B10G10R10UnormPack32,
                        samples: 1,
                    },
                    // Will be bound to `self.depth_buffer`.
                    depth: {
                        load: Clear,
                        store: DontCare,
                        format: Format::D16Unorm,
                        samples: 1,
                    }// ,
                    // // Will be bound to `self.depth_buffer`.
                    // light_depth: {
                    //     load: Clear,
                    //     store: DontCare,
                    //     format: Format::D16Unorm,
                    //     samples: 1,
                    // }
                },
                passes: [
                    // Write to the diffuse, normals and depth attachments.
                    {
                        color: [normals, diffuse, ambient, specular], // albedo
                        depth_stencil: {depth},
                        input: []
                    },
                    
                    // // TODO: Shadows
                    // {
                    //     color: [],
                    //     depth_stencil: {light_depth},
                    //     input: [todo]
                    // },

                    // Apply lighting by reading these three attachments and writing to `final_color`.
                    {
                        color: [final_color],
                        depth_stencil: {},
                        input: [normals, diffuse, ambient, specular, depth/*, light_depth*/]
                    }
                ]
            ).unwrap(),
        );

        let atch_usage = ImageUsage {
            transient_attachment: true,
            input_attachment: true,
            ..ImageUsage::none()
        };
        
        let depth_buffer = ImageView::new(
            AttachmentImage::with_usage(
                gfx_queue.device().clone(),
                dims,
                Format::D16Unorm,
                atch_usage,
            ).unwrap(),
        ).unwrap();
        // let light_depth_buffer = ImageView::new(
        //     AttachmentImage::with_usage(
        //         gfx_queue.device().clone(),
        //         dims,
        //         Format::D16Unorm,
        //         atch_usage,
        //     ).unwrap(),
        // ).unwrap();
        let normals_buffer = ImageView::new(
            AttachmentImage::with_usage(
                gfx_queue.device().clone(),
                dims,
                Format::R16G16B16A16Sfloat,
                atch_usage,
            ).unwrap(),
        ).unwrap();
        let diffuse_buffer = ImageView::new(
            AttachmentImage::with_usage(
                gfx_queue.device().clone(),
                dims,
                Format::A2B10G10R10UnormPack32,
                atch_usage,
            ).unwrap(),
        ).unwrap();
        let ambient_buffer = ImageView::new(
            AttachmentImage::with_usage(
                gfx_queue.device().clone(),
                dims,
                Format::A2B10G10R10UnormPack32,
                atch_usage,
            ).unwrap(),
        ).unwrap();
        let specular_buffer = ImageView::new(
            AttachmentImage::with_usage(
                gfx_queue.device().clone(),
                dims,
                Format::A2B10G10R10UnormPack32,
                atch_usage,
            ).unwrap(),
        ).unwrap();

        // TODO: Shadows
        // let shadow_subpass = Subpass::from(render_pass.clone(), 1).unwrap();
        // let shadow_system = AmbientLightingSystem::new(gfx_queue.clone(), lighting_subpass.clone());
        // let shadow_system = AmbientLightingSystem::new(gfx_queue.clone(), lighting_subpass.clone());
        let draw_pass_manager = DrawPassManager::new(gfx_queue.clone(), Subpass::from(render_pass.clone(), 0).unwrap(), dims);
        let lighting_pass_manager = LightingPassManager::new(gfx_queue.clone(), Subpass::from(render_pass.clone(), 1).unwrap());

        FrameSystem {
            gfx_queue,
            render_pass: render_pass as Arc<_>,
            framebuffers: Vec::new(),
            diffuse_buffer,
            ambient_buffer,
            specular_buffer,
            normals_buffer,
            depth_buffer,
            // light_depth_buffer, // TODO

            lighting_pass_manager,
            draw_pass_manager,
        }
    }

    // Builds a passbuilder for a pass
    pub fn pass_builder<F>(
        &mut self,
        before_future: F,
        img_num: usize,
        screen_to_camera: Matrix4<f32>,
        to_camera_space: Space) -> PassBuilder
    where F: GpuFuture + 'static {
        let framebuffer = self.framebuffers[img_num].clone();

        // Start the command buffer builder that will be filled throughout the frame handling.
        let mut command_buffer_builder = AutoCommandBufferBuilder::primary_one_time_submit(
            self.gfx_queue.device().clone(),
            self.gfx_queue.family(),
        ).unwrap();

        command_buffer_builder.begin_render_pass(
            framebuffer.clone(),
            SubpassContents::SecondaryCommandBuffers,
            vec![
                [0.0, 0.0, 0.0, 0.0].into(),
                [0.0, 0.0, 0.0, 0.0].into(),
                [0.0, 0.0, 0.0, 0.0].into(),
                [0.0, 0.0, 0.0, 0.0].into(),
                [0.0, 0.0, 0.0, 0.0].into(),
                1.0_f32.into(),
            ],
        ).unwrap();

        PassBuilder {
            system: self,
            before_main_cb_future: Some(Box::new(before_future)),
            framebuffer,
            command_buffer_builder: Some(command_buffer_builder),
            screen_to_camera,
            to_camera_space
        }
    }
    
    // Rebuilds the FrameSystem with the required dimentions
    pub fn rebuild_dims(&mut self, images: &[Arc<ImageView<Arc<SwapchainImage<Window>>>>]){
        let dimensions = &images[0].clone().image().dimensions().width_height();
        self.draw_pass_manager.rebuild(dimensions, self.render_pass.clone());
        
        let atch_usage = ImageUsage {
            transient_attachment: true,
            input_attachment: true,
            ..ImageUsage::none()
        };
        
        self.depth_buffer = ImageView::new(
            AttachmentImage::with_usage(
                self.gfx_queue.device().clone(),
                *dimensions,
                Format::D16Unorm,
                atch_usage,
            ).unwrap(),
        ).unwrap();
        self.normals_buffer = ImageView::new(
            AttachmentImage::with_usage(
                self.gfx_queue.device().clone(),
                *dimensions,
                Format::R16G16B16A16Sfloat,
                atch_usage,
            ).unwrap(),
        ).unwrap();
        self.diffuse_buffer = ImageView::new(
            AttachmentImage::with_usage(
                self.gfx_queue.device().clone(),
                *dimensions,
                Format::A2B10G10R10UnormPack32,
                atch_usage,
            ).unwrap(),
        ).unwrap();
        self.ambient_buffer = ImageView::new(
            AttachmentImage::with_usage(
                self.gfx_queue.device().clone(),
                *dimensions,
                Format::A2B10G10R10UnormPack32,
                atch_usage,
            ).unwrap(),
        ).unwrap();
        self.specular_buffer = ImageView::new(
            AttachmentImage::with_usage(
                self.gfx_queue.device().clone(),
                *dimensions,
                Format::A2B10G10R10UnormPack32,
                atch_usage,
            ).unwrap(),
        ).unwrap();

        self.framebuffers = images.iter()
            .map(|image| 
                Arc::new(
                    Framebuffer::start(self.render_pass.clone())
                        .add(image.clone()).unwrap()
                        .add(self.normals_buffer.clone()).unwrap()
                        .add(self.diffuse_buffer.clone()).unwrap()
                        .add(self.ambient_buffer.clone()).unwrap()
                        .add(self.specular_buffer.clone()).unwrap()
                        .add(self.depth_buffer.clone()).unwrap()
                        .build().unwrap(),
                ) as Arc<dyn FramebufferAbstract + Send + Sync>
            ).collect::<Vec<_>>();
    }
}