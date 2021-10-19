//! Everything used to render

pub mod frame_system;
pub mod three_vertex_buffers_definition;
pub mod graphics_system;
pub mod pass_builder;
pub mod draw_pass_manager;
pub mod lighting_pass_manager;

use {
    crate::{
        components::{
            Normal, 
            TextureIndex, 
            Vertex
        }, 
        registration::relation::ParentWrapper, 
        components::triangle_mesh::TriangleMesh, 
    },
    self::{
        draw_pass_manager::DrawPassManager, 
        lighting_pass_manager::LightingPassManager
    },
    std::sync::Arc,
    vulkano::buffer::CpuAccessibleBuffer
};

pub trait Drawable {
    /// Adds self's visible characteristics and its children to the draw pass.
    fn add_to_draw_pass_manager(&self, draw_pass_manager: &mut DrawPassManager);

    // fn push_set_and_buffers( // TODO: run draw command instead
    //     &self,
    //     vertex_buffers: &mut Vec<Arc<CpuAccessibleBuffer<[Vertex]>>>, 
    //     normal_buffers: &mut Vec<Arc<CpuAccessibleBuffer<[Normal]>>>, 
    //     texture_index_buffers: &mut Vec<Arc<CpuAccessibleBuffer<[TextureIndex]>>>, 
        
    //     world_sets: &mut Vec<Arc<dyn DescriptorSet + Send + Sync>>,
    //     world_buffer: CpuBufferPool<vs_draw::ty::World>,

    //     material_sets: &mut Vec<Arc<dyn DescriptorSet + Send + Sync>>, 
    //     material_buffer: CpuBufferPool<fs_draw::ty::Material>,
        
    //     pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>
    // );

    /// Adds self's relevant characteristics and its children to the pass managers.
    fn load_into_managers(
        &self,
        this: ParentWrapper,
        draw_pass_manager: &mut DrawPassManager,
        lighting_pass_manager: &mut LightingPassManager,
    );
    
    /// Checks if an object is visible
    fn get_visible(&self) -> bool;
    
    /// Adds a triangle mesh to the Drawable struct
    fn add_triangle_mesh(&mut self, triangle_mesh: Arc<TriangleMesh>) -> Result<(), ()>;
    
    fn get_triangle_mesh(&self) -> Vec<Arc<TriangleMesh>>;
    fn get_vertex_buffer(&self) -> Vec<Option<Arc<CpuAccessibleBuffer<[Vertex]>>>>;
    fn get_normals_buffer(&self) -> Vec<Option<Arc<CpuAccessibleBuffer<[Normal]>>>>;
    fn get_texture_indices_buffer(&self) -> Vec<Option<Arc<CpuAccessibleBuffer<[TextureIndex]>>>>;
}
