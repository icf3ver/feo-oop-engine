use std::{iter, sync::{Arc, RwLock}};
use vulkano::{buffer::{BufferUsage, CpuAccessibleBuffer, CpuBufferPool}, command_buffer::AutoCommandBufferBuilder, command_buffer::{AutoCommandBuffer, DynamicState}, descriptor::{DescriptorSet, descriptor_set::PersistentDescriptorSet}, device::Queue, framebuffer::{RenderPassAbstract, Subpass}, pipeline::{GraphicsPipeline, GraphicsPipelineAbstract}};
use crate::{components::{Normal, TextureIndex, Vertex}, scene::game_object::camera::Camera, shaders::{fs_draw, vs_draw}};
use vulkano::pipeline::viewport::Viewport;
use super::three_vertex_buffers_definition::ThreeBuffersDefinition;

/// Manages the draw passes.
pub struct DrawPassManager {
    gfx_queue: Arc<Queue>,
    pub(crate) pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,

    pub(crate) vertex_buffers: Vec<Arc<CpuAccessibleBuffer<[Vertex]>>>,
    pub(crate) normal_buffers: Vec<Arc<CpuAccessibleBuffer<[Normal]>>>,
    pub(crate) texture_index_buffers: Vec<Arc<CpuAccessibleBuffer<[TextureIndex]>>>,

    pub(crate) scene_buffer: CpuBufferPool<vs_draw::ty::Camera>,
    pub(crate) scene_set: Option<Arc<dyn DescriptorSet + Send + Sync>>,

    pub(crate) world_buffers: CpuBufferPool<vs_draw::ty::World>,
    pub(crate) world_sets: Vec<Arc<dyn DescriptorSet + Send + Sync>>,

    pub(crate) material_buffers: CpuBufferPool<fs_draw::ty::Material>,
    pub(crate) material_sets: Vec<Arc<dyn DescriptorSet + Send + Sync>>,
}

impl DrawPassManager {
    /// Initializes a triangle drawing system.
    pub fn new<R>(gfx_queue: Arc<Queue>, subpass: Subpass<R>, viewport_dimensions: [u32; 2]) -> DrawPassManager 
    where R: RenderPassAbstract + Send + Sync + 'static {
        let scene_buffer = CpuBufferPool::<vs_draw::ty::Camera>::new(gfx_queue.device().clone(), BufferUsage::all());
        let world_buffers = CpuBufferPool::<vs_draw::ty::World>::new(gfx_queue.device().clone(), BufferUsage::all());
        let material_buffers = CpuBufferPool::<fs_draw::ty::Material>::new(gfx_queue.device().clone(), BufferUsage::all());

        let vs = vs_draw::Shader::load(gfx_queue.device().clone()).unwrap();
        let fs = fs_draw::Shader::load(gfx_queue.device().clone()).unwrap();

        let pipeline = Arc::new(
            GraphicsPipeline::start()
                .vertex_input(ThreeBuffersDefinition::<Vertex, Normal, TextureIndex>::new())  // TODO one buffer  
                .vertex_shader(vs.main_entry_point(), ())
                .triangle_list()
                .viewports_dynamic_scissors_irrelevant(1)
                .viewports(iter::once(Viewport {
                    origin: [0.0, 0.0],
                    dimensions: [viewport_dimensions[0] as f32, viewport_dimensions[1] as f32],
                    depth_range: 0.0..1.0,
                }))
                .fragment_shader(fs.main_entry_point(), ())
                .depth_write(true)
                .depth_stencil_simple_depth()
                .render_pass(subpass)
                .build(gfx_queue.device().clone()).unwrap()
        );

        DrawPassManager {
            gfx_queue,
            pipeline: pipeline as Arc<_>,

            vertex_buffers: Vec::new(),
            normal_buffers: Vec::new(),
            texture_index_buffers: Vec::new(),

            scene_buffer,
            scene_set: None,

            world_buffers,
            world_sets: Vec::new(),
            
            material_buffers,
            material_sets: Vec::new(),
        }
    }

    /// Recreates the Projection and view matrix.
    pub fn recreate_camera_set(&mut self, camera: Arc<RwLock<dyn Camera>>){
        self.scene_set = Some({
            let uniform_buffer_subbuffer = {
                let camera = camera.clone();
                let camera = camera.read().expect("Unable to acquire read lock on main camera"); // TODO: AM HERE
                let uniform_data: vs_draw::ty::Camera = camera.create_uniforms(); // TODO: rmv clone future rn needs to be mut

                self.scene_buffer.next(uniform_data).unwrap()
            };

            let layout = self.pipeline.descriptor_set_layout(0).unwrap();

            Arc::new(
                PersistentDescriptorSet::start(layout.clone())
                    .add_buffer(uniform_buffer_subbuffer).unwrap()
                    .build().unwrap()
            )
        });
    }

    /// Clear the buffer
    #[inline]
    pub fn clear(&mut self){
        self.vertex_buffers = Vec::new();
        self.normal_buffers = Vec::new();
        self.texture_index_buffers = Vec::new();

        self.world_sets = Vec::new();
        self.material_sets = Vec::new();
        
        self.scene_set = None;
    }

    // // Unused
    // pub fn initialize_sets_and_buffers(&mut self, scene: &Scene, main_camera: Option<Arc<RwLock<dyn Camera>>>) {
    //     self.clear();
        
    //     scene.push_set_and_buffers(
    //         &mut self.vertex_buffers, 
    //         &mut self.normal_buffers, 
    //         &mut self.texture_index_buffers,
    //         &mut self.world_sets, 
    //         self.world_buffers.clone(), 
    //         &mut self.material_sets, 
    //         self.material_buffers.clone(), 
    //         // &mut self.texture_sets,
    //         self.pipeline.clone()
    //     );

    //     self.scene_set = Some({
    //         let uniform_buffer_subbuffer = {
    //             let camera = main_camera.clone().unwrap();
    //             let camera = camera.read().expect("Unable to acquire read lock on main camera"); // TODO: AM HERE
    //             let uniform_data: vs_draw::ty::Camera = camera.create_uniforms(); // TODO: rmv clone future rn needs to be mut

    //             self.scene_buffer.next(uniform_data).unwrap()
    //         };

    //         let layout = self.pipeline.descriptor_set_layout(0).unwrap();

    //         Arc::new(
    //             PersistentDescriptorSet::start(layout.clone())
    //                 .add_buffer(uniform_buffer_subbuffer).unwrap()
    //                 .build().unwrap()
    //         )
    //     });
    // }

    /// Builds a secondary command buffer that draws the triangle on the current subpass.
    pub fn draw(&self) -> AutoCommandBuffer {
        let mut builder = AutoCommandBufferBuilder::secondary_graphics(
            self.gfx_queue.device().clone(),
            self.gfx_queue.family(),
            self.pipeline.clone().subpass().clone(),
        ).unwrap();
        
        (0..self.vertex_buffers.len()).for_each(|i| {
            builder.draw(
                self.pipeline.clone(),
                &DynamicState::none(),
                vec![
                    self.vertex_buffers[i].clone(), 
                    self.normal_buffers[i].clone(), 
                    self.texture_index_buffers[i].clone()
                ],
                vec![
                    self.scene_set.clone().unwrap(), 
                    self.world_sets[i].clone(),
                    self.material_sets[i].clone(),
                ],
                (),
                vec![],
            ).unwrap();
        });

        builder.build().unwrap()
    }

    /// rebuild the swapchain and linked
    pub fn rebuild(
            &mut self,
            dimensions: &[u32; 2],
            render_pass: Arc<dyn RenderPassAbstract + Send + Sync>) {
        let vs = vs_draw::Shader::load(self.gfx_queue.device().clone()).unwrap();
        let fs = fs_draw::Shader::load(self.gfx_queue.device().clone()).unwrap();
        
        self.pipeline = Arc::new(
            GraphicsPipeline::start()
                .vertex_input(ThreeBuffersDefinition::<Vertex, Normal, TextureIndex>::new())  // TODO one buffer  
                .vertex_shader(vs.main_entry_point(), ())
                .triangle_list()
                .viewports_dynamic_scissors_irrelevant(1)
                .viewports(iter::once(Viewport {
                    origin: [0.0, 0.0],
                    dimensions: [dimensions[0] as f32, dimensions[1] as f32],
                    depth_range: 0.0..1.0,
                }))
                .fragment_shader(fs.main_entry_point(), ())
                .depth_write(true)
                .depth_stencil_simple_depth()
                .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
                .build(self.gfx_queue.device().clone())
                .unwrap()
        );
    }
}
