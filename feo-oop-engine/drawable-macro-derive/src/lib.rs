use proc_macro::TokenStream;
use quote::{quote, __private::Span};
use syn::{self, AngleBracketedGenericArguments, Data, DataStruct, Fields, FieldsNamed, GenericArgument, Ident, PathArguments, Type};

#[proc_macro_derive(Drawable, attributes(light))]
pub fn drawable_macro_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();

    impl_drawable_macro(&ast)
}

#[derive(Clone)]
enum Mesh{
    Vec(Ident),
    MultipleVec(Vec<Ident>),
    Mixed( /* vec */ Vec<Ident>, /* single */ Vec<Ident>),
    Multiple(Vec<Ident>),
    Some(Ident),
    None,
}

fn impl_drawable_macro(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let data = &ast.data;
    let attributes = &ast.attrs;

    let mut mesh: Mesh = Mesh::None;
    let mut visible_var: bool = false;

    match data {
        Data::Struct( DataStruct { fields, .. } ) => match fields {
            Fields::Named( FieldsNamed { named, .. } ) => {
                named.iter().for_each(|field| {
                    if let Type::Path(type_path) = field.ty.clone() {
                        let path = type_path.path;
                        if path.segments.iter().last().unwrap().ident == Ident::new("Arc", Span::call_site()) &&
                                match path.segments.iter().last().unwrap().arguments.clone() { // indentation bad
                                    PathArguments::AngleBracketed(AngleBracketedGenericArguments{args, ..}) => {
                                        match args.iter().clone().next().unwrap(){
                                            GenericArgument::Type(Type::Path(type_path)) => {
                                                type_path.path.segments.iter().last().unwrap().ident == Ident::new("TriangleMesh", Span::call_site())
                                            },
                                            _ => false
                                        } 
                                    },
                                    _ => unreachable!()
                                } {
                            match mesh.clone() {
                                Mesh::Multiple(single_vec) => {
                                    let mut single_vec = single_vec;
                                    single_vec.push(field.ident.clone().unwrap());
                                    mesh = Mesh::Multiple(single_vec);
                                },
                                Mesh::Some(single_ident) => {
                                    mesh = Mesh::Multiple(vec![single_ident, field.ident.clone().unwrap()]);
                                },
                                Mesh::None => { 
                                    mesh = Mesh::Some(field.ident.clone().unwrap());
                                },
                                Mesh::Vec(pass_back_vec_ident) => {
                                    mesh = Mesh::Mixed(vec![pass_back_vec_ident], vec![field.ident.clone().unwrap()]);
                                },
                                Mesh::MultipleVec(pass_back_vec_vec) => {
                                    mesh = Mesh::Mixed(pass_back_vec_vec, vec![field.ident.clone().unwrap()]);
                                },
                                Mesh::Mixed(pass_back_vec_vec, single_vec ) => {
                                    let mut single_vec = single_vec;
                                    single_vec.push(field.ident.clone().unwrap());
                                    mesh = Mesh::Mixed(pass_back_vec_vec, single_vec);
                                }
                            }
                        } else if path.segments.iter().last().unwrap().ident == Ident::new("Vec", Span::call_site()) && 
                                match path.segments.iter().last().unwrap().arguments.clone() {
                                    PathArguments::AngleBracketed(AngleBracketedGenericArguments{args, ..}) => {
                                        match args.iter().clone().next().unwrap() {
                                            GenericArgument::Type(Type::Path(type_path)) => {
                                                type_path.path.segments.iter().last().unwrap().ident == Ident::new("Arc", Span::call_site()) &&
                                                match type_path.path.segments.iter().last().unwrap().arguments.clone() { // indentation bad
                                                    PathArguments::AngleBracketed(AngleBracketedGenericArguments{args, ..}) => {
                                                        match args.iter().clone().next().unwrap(){
                                                            GenericArgument::Type(Type::Path(type_path)) => {
                                                                type_path.path.segments.iter().last().unwrap().ident == Ident::new("TriangleMesh", Span::call_site())
                                                            }
                                                            _ => false
                                                        } 
                                                    },
                                                    _ => unreachable!()
                                                }
                                            }
                                            _ => false
                                        } 
                                    },
                                    _ => unreachable!()
                                } {
                            match mesh.clone() {
                                Mesh::Multiple(pass_back_vec_single) => {
                                    mesh = Mesh::Mixed(vec![field.ident.clone().unwrap()], pass_back_vec_single);
                                },
                                Mesh::Some(pass_back_ident_single) => {
                                    mesh = Mesh::Mixed(vec![field.ident.clone().unwrap()], vec![pass_back_ident_single]);
                                },
                                Mesh::None => { 
                                    mesh = Mesh::Vec(field.ident.clone().unwrap());
                                },
                                Mesh::Vec(vec_ident) => { // confusion TODO: fix
                                    mesh = Mesh::MultipleVec(vec![vec_ident, field.ident.clone().unwrap()]);
                                },
                                Mesh::MultipleVec(vec_vec) => {
                                    let mut vec_vec = vec_vec;
                                    vec_vec.push(field.ident.clone().unwrap());
                                    mesh = Mesh::MultipleVec(vec_vec);
                                },
                                Mesh::Mixed(vec_vec, pass_back_single_vec ) => {
                                    let mut vec_vec = vec_vec;
                                    vec_vec.push(field.ident.clone().unwrap());
                                    mesh = Mesh::Mixed(vec_vec, pass_back_single_vec);
                                }
                            }
                        } else if path.is_ident("bool") && field.ident.clone().unwrap() == Ident::new("visible", Span::call_site()){
                            visible_var = true;
                        }
                    };
                });
            },
            _ => panic!("You can only derive gameObject on structs with named fields"), 
        },
        _ => panic!("You can only derive gameObject on structs"), 
    };

    let mesh_fns = match mesh.clone() {
        Mesh::Multiple(ident) => quote! {
            #[inline] fn get_triangle_mesh(&self) -> Vec<Arc<TriangleMesh>> { vec![#(self.#ident ,)*] }
            #[inline] fn get_vertex_buffer(&self) -> Vec<Option<std::sync::Arc<vulkano::buffer::CpuAccessibleBuffer<[crate::components::Vertex]>>>> { vec![#(self.#ident.vertex_buffer.clone() ,)*] }
            #[inline] fn get_normals_buffer(&self) -> Vec<Option<std::sync::Arc<vulkano::buffer::CpuAccessibleBuffer<[crate::components::Normal]>>>> { vec![#(self.#ident.normal_buffer.clone() ,)*] }
            #[inline] fn get_texture_indices_buffer(&self) -> Vec<Option<std::sync::Arc<vulkano::buffer::CpuAccessibleBuffer<[crate::components::TextureIndex]>>>> { vec![#(self.#ident.texture_indices_buffer.clone() ,)*] }
        },
        Mesh::Some(ident) => quote! {
            #[inline] fn get_triangle_mesh(&self) -> Vec<Arc<TriangleMesh>> { vec![self.#ident] }
            #[inline] fn get_vertex_buffer(&self) -> Vec<Option<std::sync::Arc<vulkano::buffer::CpuAccessibleBuffer<[crate::components::Vertex]>>>> { vec![self.#ident.vertex_buffer.clone()] }
            #[inline] fn get_normals_buffer(&self) -> Vec<Option<std::sync::Arc<vulkano::buffer::CpuAccessibleBuffer<[crate::components::Normal]>>>> { vec![self.#ident.normal_buffer.clone()] }
            #[inline] fn get_texture_indices_buffer(&self) -> Vec<Option<std::sync::Arc<vulkano::buffer::CpuAccessibleBuffer<[crate::components::TextureIndex]>>>> { vec![self.#ident.texture_indices_buffer.clone()] }
        },
        Mesh::None => quote! {
            #[inline] fn get_triangle_mesh(&self) -> Vec<Arc<TriangleMesh>> { Vec::new() }
            #[inline] fn get_vertex_buffer(&self) -> Vec<Option<std::sync::Arc<vulkano::buffer::CpuAccessibleBuffer<[crate::components::Vertex]>>>> { Vec::new() }
            #[inline] fn get_normals_buffer(&self) -> Vec<Option<std::sync::Arc<vulkano::buffer::CpuAccessibleBuffer<[crate::components::Normal]>>>> { Vec::new() }
            #[inline] fn get_texture_indices_buffer(&self) -> Vec<Option<std::sync::Arc<vulkano::buffer::CpuAccessibleBuffer<[crate::components::TextureIndex]>>>> { Vec::new() }
        },
        Mesh::Vec(ident) => quote! {

            #[inline]
            fn get_triangle_mesh(&self) -> Vec<Arc<TriangleMesh>> { self.#ident.clone() }

            #[inline]
            fn get_vertex_buffer(&self) -> Vec<Option<std::sync::Arc<vulkano::buffer::CpuAccessibleBuffer<[crate::components::Vertex]>>>> { 
                self.#ident.clone().into_iter().map(|mesh| {
                    mesh.vertex_buffer.clone()
                }).collect::<Vec<Option<std::sync::Arc<vulkano::buffer::CpuAccessibleBuffer<[crate::components::Vertex]>>>>>()
            }
            #[inline]
            fn get_normals_buffer(&self) -> Vec<Option<std::sync::Arc<vulkano::buffer::CpuAccessibleBuffer<[crate::components::Normal]>>>> { 
                self.#ident.clone().into_iter().map(|mesh| {
                    mesh.normal_buffer.clone()
                }).collect::<Vec<Option<std::sync::Arc<vulkano::buffer::CpuAccessibleBuffer<[crate::components::Normal]>>>>>()
            }
            
            #[inline]
            fn get_texture_indices_buffer(&self) -> Vec<Option<std::sync::Arc<vulkano::buffer::CpuAccessibleBuffer<[crate::components::TextureIndex]>>>> { 
                self.#ident.clone().into_iter().map(|mesh| {
                    mesh.texture_indices_buffer.clone()
                }).collect::<Vec<Option<std::sync::Arc<vulkano::buffer::CpuAccessibleBuffer<[crate::components::TextureIndex]>>>>>()
            }
        },
        Mesh::MultipleVec(idents) => quote! {
            
            #[inline]
            fn get_triangle_mesh(&self) -> Vec<Arc<TriangleMesh>> { 
                let merged = Vec::new();
                #(
                    merged.append(self.#idents);
                )*
                merged
            }
            #[inline]
            fn get_vertex_buffer(&self) -> Vec<Option<std::sync::Arc<vulkano::buffer::CpuAccessibleBuffer<[crate::components::Vertex]>>>> { 
                let merged = Vec::new();
                #(
                    merged.append(self.#idents);
                )*
                merged.into_iter().map(|mesh| {
                    mesh.vertex_buffer.clone()
                }).collect::<Vec<Option<std::sync::Arc<vulkano::buffer::CpuAccessibleBuffer<[crate::components::Vertex]>>>>>()
            }
            #[inline]
            fn get_normals_buffer(&self) -> Vec<Option<std::sync::Arc<vulkano::buffer::CpuAccessibleBuffer<[crate::components::Normal]>>>> { 
                let merged = Vec::new();
                #(
                    merged.append(self.#idents);
                )*
                merged.into_iter().map(|mesh| {
                    mesh.normal_buffer.clone()
                }).collect::<Vec<Option<std::sync::Arc<vulkano::buffer::CpuAccessibleBuffer<[crate::components::Normal]>>>>>()
            }
            #[inline]
            fn get_texture_indices_buffer(&self) -> Vec<Option<std::sync::Arc<vulkano::buffer::CpuAccessibleBuffer<[crate::components::TextureIndex]>>>> { 
                let merged = Vec::new();
                #(
                    merged.append(self.#idents);
                )*
                merged.into_iter().map(|mesh| {
                    mesh.texture_indices_buffer.clone()
                }).collect::<Vec<Option<std::sync::Arc<vulkano::buffer::CpuAccessibleBuffer<[crate::components::TextureIndex]>>>>>()
            }
        },
        Mesh::Mixed(idents_vec, idents_single) => quote! {

            #[inline]
            fn get_triangle_mesh(&self) -> Vec<Arc<TriangleMesh>> { 
                let merged = Vec::new();

                #(
                    merged.append(self.#idents_vec);
                )*
                
                #(
                    merged.push(self.#idents_single);
                )*

                merged
            }
            #[inline]
            fn get_vertex_buffer(&self) -> Vec<Option<std::sync::Arc<vulkano::buffer::CpuAccessibleBuffer<[crate::components::Vertex]>>>> { 
                let merged = Vec::new();

                #(
                    merged.append(self.#idents_vec);
                )*
                
                #(
                    merged.push(self.#idents_single);
                )*

                merged.into_iter().map(|mesh| {
                    mesh.vertex_buffer.clone()
                }).collect::<Vec<Option<std::sync::Arc<vulkano::buffer::CpuAccessibleBuffer<[crate::components::Vertex]>>>>>()
            }
            #[inline]
            fn get_normals_buffer(&self) -> Vec<Option<std::sync::Arc<vulkano::buffer::CpuAccessibleBuffer<[crate::components::Normal]>>>> { // TODO make normal not normals
                let merged = Vec::new();
                
                #(
                    merged.append(self.#idents_vec);
                )*
                
                #(
                    merged.push(self.#idents_single);
                )*

                merged.into_iter().map(|mesh| {
                    mesh.normal_buffer.clone()
                }).collect::<Vec<Option<std::sync::Arc<vulkano::buffer::CpuAccessibleBuffer<[crate::components::Normal]>>>>>()
            }
            #[inline]
            fn get_texture_indices_buffer(&self) -> Vec<Option<std::sync::Arc<vulkano::buffer::CpuAccessibleBuffer<[crate::components::TextureIndex]>>>> { 
                let merged = Vec::new();
                
                #(
                    merged.append(self.#idents_vec);
                )*
                
                #(
                    merged.push(self.#idents_single);
                )*
                
                merged.into_iter().map(|mesh| {
                    mesh.texture_indices_buffer.clone()
                }).collect::<Vec<Option<std::sync::Arc<vulkano::buffer::CpuAccessibleBuffer<[crate::components::TextureIndex]>>>>>()
            }
        }
    };

    let add_triangle_mesh_inner = match mesh.clone() {
        Mesh::Vec(ident_vec) => quote! { Ok(self.#ident_vec.push(mesh)) },
        Mesh::MultipleVec(_vec_vec) => quote! { Err(()) }, // which vec? TODO
        Mesh::Mixed(_vec_vec, _) => quote! { Err(()) },    // which vec?
        Mesh::Multiple(_) |
            Mesh::Some(_) | 
                Mesh::None => quote! { Err(()) } // TODO: check for uninitialized fields maybe
    };

    let visible = match visible_var {
        true => quote! { self.visible },
        false => quote! { true }
    };
    
    let render_self = match mesh {
        Mesh::None => quote! { /* none */ },
        _ => quote! {
            self.get_triangle_mesh().into_iter().for_each(|mesh| {
                if let (Some(vertex_buffer), Some(normal_buffer), Some(texture_indices_buffer), Some(material)) = (mesh.clone().vertex_buffer.clone(), mesh.clone().normal_buffer.clone(), mesh.clone().texture_indices_buffer.clone(), mesh.clone().material.clone()){
                    world_sets.push({
                        let subbuffer = world_buffers.next(crate::shaders::vs_draw::ty::World { object_to: self.get_subspace().build().transpose().into() }).unwrap();
                        let layout = pipeline.descriptor_set_layout(1).unwrap();
                
                        Arc::new(
                            PersistentDescriptorSet::start(layout.clone()).add_buffer(subbuffer).unwrap().build().unwrap()
                        )
                    });

                    material_sets.push({
                        let non_texture_subbuffer = material_buffers.next(material.0).unwrap();
                        
                        let layout = pipeline.descriptor_set_layout(2).unwrap();
                        
                        Arc::new(
                            PersistentDescriptorSet::start(layout.clone())
                                .add_buffer(non_texture_subbuffer).unwrap()
                                .add_sampled_image(material.1[0].img_view.clone(), material.1[0].sampler.clone()).unwrap()
                                .add_sampled_image(material.1[1].img_view.clone(), material.1[1].sampler.clone()).unwrap()
                                .add_sampled_image(material.1[2].img_view.clone(), material.1[2].sampler.clone()).unwrap()
                                .add_sampled_image(material.1[3].img_view.clone(), material.1[3].sampler.clone()).unwrap()
                                .build().unwrap()
                        )
                    });

                    // texture_sets.push({
                    //     let layout = pipeline.descriptor_set_layout(3).unwrap();
                    //     Arc::new(
                    //         PersistentDescriptorSet::start(layout.clone())
                    //             .add_sampled_image(material.1[0].img_view.clone(), material.1[0].sampler.clone()).unwrap()
                    //             .add_sampled_image(material.1[1].img_view.clone(), material.1[1].sampler.clone()).unwrap()
                    //             .build().unwrap()
                    //     )
                    // });
                    
                    vertex_buffers.push(vertex_buffer);
                    normal_buffers.push(normal_buffer);
                    texture_index_buffers.push(texture_indices_buffer);
                }
            });
        }
    };
    
    let mut load_light = quote!{ /* nothing */ };
    attributes.iter().for_each(|attr| {
        if attr.path.is_ident("light") {
            load_light = quote!{
                match this{
                    ParentWrapper::GameObject(game_object) => {
                        lighting_pass_manager.lights.push(self.cast_light_arc_rwlock(game_object).unwrap());
                    },
                    ParentWrapper::Scene(_) => unreachable!(),
                }
            };
        }
    });

    let gen = quote! {
        impl Drawable for #name {
            fn add_to_draw_pass_manager(&self, draw_pass_manager: &mut DrawPassManager){
                let vertex_buffers = &mut draw_pass_manager.vertex_buffers;
                let normal_buffers = &mut draw_pass_manager.normal_buffers;
                let texture_index_buffers = &mut draw_pass_manager.texture_index_buffers;

                let world_sets = &mut draw_pass_manager.world_sets;
                let world_buffers = draw_pass_manager.world_buffers.clone();

                let material_sets = &mut draw_pass_manager.material_sets;
                let material_buffers = draw_pass_manager.material_buffers.clone();

                let pipeline = draw_pass_manager.pipeline.clone();

                if #visible {
                    #render_self
      
                    for child in self.children.clone(){
                        child.read().unwrap().add_to_draw_pass_manager(
                            draw_pass_manager
                        );
                    }
                }
            }

            // fn push_set_and_buffers(&self, 
            //         vertex_buffers: &mut Vec<Arc<vulkano::buffer::CpuAccessibleBuffer<[crate::components::Vertex]>>>, 
            //         normal_buffers:  &mut Vec<Arc<vulkano::buffer::CpuAccessibleBuffer<[crate::components::Normal]>>>, 
            //         texture_index_buffers:  &mut Vec<Arc<vulkano::buffer::CpuAccessibleBuffer<[crate::components::TextureIndex]>>>, 

            //         world_sets: &mut Vec<Arc<dyn DescriptorSet + Send + Sync>>, 
            //         world_buffers: CpuBufferPool<crate::shaders::vs_draw::ty::World>, 
                    
            //         material_sets: &mut Vec<Arc<dyn DescriptorSet + Send + Sync>>, 
            //         material_buffers: CpuBufferPool<crate::shaders::fs_draw::ty::Material>, 

            //         // texture_sets: &mut Vec<Arc<dyn DescriptorSet + Send + Sync>>,

            //         pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>) {
            //     if #visible {
            //         #render_self
      
            //         for child in self.children.clone(){
            //             child.read().unwrap().push_set_and_buffers(
            //                 vertex_buffers, 
            //                 normal_buffers, 
            //                 texture_index_buffers, 
            //                 world_sets, 
            //                 world_buffers.clone(), 
            //                 material_sets, 
            //                 material_buffers.clone(), 
            //                 // texture_sets,
            //                 pipeline.clone()
            //             );
            //         }
            //     }
            // }

            fn load_into_managers(&self, this: ParentWrapper, draw_pass_manager: &mut DrawPassManager, lighting_pass_manager: &mut LightingPassManager){
                if #visible {
                    #load_light
                    
                    let vertex_buffers = &mut draw_pass_manager.vertex_buffers;
                    let normal_buffers = &mut draw_pass_manager.normal_buffers;
                    let texture_index_buffers = &mut draw_pass_manager.texture_index_buffers;
            
                    let world_sets = &mut draw_pass_manager.world_sets;
                    let world_buffers = draw_pass_manager.world_buffers.clone();
            
                    let material_sets = &mut draw_pass_manager.material_sets;
                    let material_buffers = draw_pass_manager.material_buffers.clone();

                    // let texture_sets = &mut draw_pass_manager.texture_sets;

                    let pipeline = draw_pass_manager.pipeline.clone();
                    
                    #render_self
        
                    for child in self.children.clone(){
                        child.read().unwrap().load_into_managers(
                            ParentWrapper::GameObject(child.clone()),
                            draw_pass_manager,
                            lighting_pass_manager
                        );
                    }
                }
            }
            
            fn add_triangle_mesh(&mut self, mesh: Arc<TriangleMesh>) -> Result<(), ()>{
                #add_triangle_mesh_inner
            }
        
            #[inline]
            fn get_visible(&self) -> bool { #visible }

            #mesh_fns
        }
    };

    gen.into()
}
