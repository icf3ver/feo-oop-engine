//! A GameObject that can be built from an obj file.
//! 
//! TODO: explain OOP here
//! 
use {
    super::{
        GameObject,
        camera::Camera,
        light::Light,
    },
    crate::{
        registration::{
            relation::{ParentWrapper, Child, Parent},
            named::Named,
            id::{
                ID
            }
        },
        scripting::{
            Script,
            executor::Spawner,
            Scriptable, 
            globals::{
                EngineGlobals, 
                Global
            }
        },
        graphics::{
            Drawable,
            draw_pass_manager::DrawPassManager,
            lighting_pass_manager::LightingPassManager,
        },
        components::{
            Vertex,
            TextureIndex,
            Normal
        },
        term_ui,
        event::UserEvent,
        components::{material::Material, triangle_mesh::TriangleMesh}
    },
    feo_math::{
        utils::space::Space, 
        rotation::quaternion::Quaternion,
        linear_algebra::vector3::Vector3
    },
    std::{
        any::Any, 
        collections::HashMap, 
        fs, 
        io::stdout, 
        mem, 
        sync::{Arc, RwLock}
    },
    futures::executor::block_on,
    vulkano::{
        descriptor::{
            descriptor_set::PersistentDescriptorSet,
        },
        sync::GpuFuture
    },
    winit::event::Event
};

#[derive(Scriptable, GameObject, Drawable, Child, Parent, Named)] // import
pub struct Obj {
    pub id: ID,
    pub name: String,
    pub parent: ParentWrapper,

    pub visible: bool,

    pub subspace: Space, // note is the subspace within the parent space

    pub triangle_mesh: Vec<Arc<TriangleMesh>>,
    // pub material: Option<Material>, // object does not have material triangle mesh does

    pub script: Option<Box<Script<Self>>>,

    pub children: Vec<Arc<RwLock<dyn GameObject>>>,
}

impl std::fmt::Debug for Obj {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Obj")
            .field("id", &self.id)
            .field("name", &self.name)
            .field("parent", &self.parent)
            .field("visible", &self.visible)
            .field("subspace", &self.subspace)
            .field("triangle_mesh", &self.triangle_mesh)
            .field("script", &self.script)
            .field("children", &self.children).finish()
    }
}

impl Clone for Obj {
    fn clone(&self) -> Self {
        let id = self.id.get_system().take();
        Obj{
            id,
            name: self.name.clone(),
            parent: self.parent.clone(),
            visible: self.visible,
            subspace: self.subspace,
            triangle_mesh: self.triangle_mesh.clone(),
            script: self.script.clone(),
            children: self.children.clone().into_iter().map(|_child| {
                // Dangerous
                todo!();
            }).collect::<Vec<Arc<RwLock<dyn GameObject>>>>(),
        }
    }
}

impl PartialEq for Obj { // auto-generate this somehow
    fn eq(&self, other: &Self) -> bool {
        self.get_id() == other.get_id()
    }
}

// TODO: rename to builders and build gameobjects with them
impl Obj {
    #[allow(clippy::too_many_arguments, clippy::or_fun_call)]
    pub fn new_empty(
            name: Option<&str>,

            parent: Option<Arc<RwLock<dyn GameObject>>>, 

            position: Option<Vector3<f32>>, 
            rotation: Option<Quaternion<f32>>,
            scale_factor: Option<Vector3<f32>>,

            visible: bool,

            engine_globals: EngineGlobals,

            script: Option<Box<Script<Self>>>) -> Arc<RwLock<Self>> {
        let id = engine_globals.id_system.take();

        return Arc::new(RwLock::new( Obj {
            name: name.unwrap_or((String::from("obj_") + id.to_string().as_str()).as_str()).to_owned(),
            id,
            parent: match parent {
                Some(game_object) => {
                    ParentWrapper::GameObject(game_object)
                },
                None => {
                    ParentWrapper::Scene(engine_globals.scene)
                }
            },

            visible,

            subspace: Space::new(position, rotation, scale_factor),

            triangle_mesh: Vec::new(),

            script,

            children: Vec::new()
        }));
    }

    #[allow(clippy::too_many_arguments, clippy::or_fun_call)]
    pub fn from_triangle_mesh_vec(
            name: Option<&str>, 
            triangle_mesh_vec: Vec<Arc<TriangleMesh>>,

            parent: Option<Arc<RwLock<dyn GameObject>>>, 

            position: Option<Vector3<f32>>, 
            rotation: Option<Quaternion<f32>>,
            scale_factor: Option<Vector3<f32>>,

            visible: bool,

            engine_globals: EngineGlobals,

            script: Option<Box<Script<Self>>>) -> Arc<RwLock<Self>> {
        let id = engine_globals.id_system.take();
        return Arc::new(RwLock::new( Obj{
            name: name.unwrap_or((String::from("obj_") + id.to_string().as_str()).as_str()).to_owned(),
            id,
            parent: match parent {
                Some(game_object) => {
                    ParentWrapper::GameObject(game_object)
                },
                None => {
                    ParentWrapper::Scene(engine_globals.scene)
                }
            },

            visible,

            subspace: Space::new(position, rotation, scale_factor),

            triangle_mesh: triangle_mesh_vec,

            script,

            children: Vec::new(),
        }));
    }
    
    #[allow(clippy::too_many_arguments)]
    pub fn from_obj<'a>( // TODO: cut file into groups pass groups into triangle mesh for parsing
            name: Option<&str>, 
            path: &str, 

            parent: Option<Arc<RwLock<dyn GameObject>>>, 

            position: Option<Vector3<f32>>, 
            rotation: Option<Quaternion<f32>>,
            scale_factor: Option<Vector3<f32>>,
            
            visible: bool,

            engine_globals: EngineGlobals,

            script: Option<Box<Script<Self>>>) -> Result<Arc<RwLock<Self>>, &'a str>{
        
        //   Data Pools   //

        let mut vertex_positions= Vec::new();
        let mut texture_indices = Vec::new();
        let mut normals = Vec::new();
        
        let mut mtls_hashmap: HashMap<String, (Arc<Material>, Box<dyn GpuFuture>)> = HashMap::new();

        //   Read In String Data   //
        
        let content = fs::read_to_string(path)
            .unwrap_or_else(|_| panic!("Something went wrong when trying to read {}.", path));
        let mut last_block = Box::new(Vec::new());
        let lines: Vec<(&str, Vec<&str>)> = content.lines().filter_map(|line| {
            if !line.is_empty() {
                let mut e = line.split_whitespace();
                let ty: &str = e.next().unwrap();
                match &*ty {

                    "" | "#" | "s" | "l" => None,

                    //   Vertex Data   //

                    "v" => {
                        vertex_positions.push(Box::new(Vertex::new(
                            e.next().unwrap().parse::<f32>().unwrap(),
                            e.next().unwrap().parse::<f32>().unwrap(),
                            e.next().unwrap().parse::<f32>().unwrap()
                        )));

                        None
                    },
                    "vt" => {
                        texture_indices.push(Box::new(TextureIndex::new(
                            e.next().unwrap().parse::<f32>().unwrap(),
                            e.next().unwrap().parse::<f32>().unwrap()
                        )));

                        None
                    },
                    "vn" => {
                        normals.push(Box::new(Normal::new(
                            e.next().unwrap().parse::<f32>().unwrap(),
                            e.next().unwrap().parse::<f32>().unwrap(),
                            e.next().unwrap().parse::<f32>().unwrap()
                        )));
                        
                        None
                    },

                    //   Materials   //

                    "mtllib" => {
                        let files = e.fold(String::new(), |mut a, b| {
                            a.reserve(b.len() + 1);
                            a.push_str(b);
                            a.push(' ');
                            a
                        });

                        let files = files.trim_end_matches(".mtl ");

                        files.split(".mtl ").for_each(|file| {
                            let file = "/".to_owned() + file + ".mtl";
                            let path = path.rsplitn(2, '/').nth(1).unwrap();
                            let path = path.to_owned() + file.as_str();

                            mtls_hashmap.extend(Material::from_mtllib(&path, engine_globals.clone().queue));
                        });

                        None
                    },

                    
                    //   Faces and Materials   //

                    "f" | "usemtl" => {
                        last_block.push(line);
                        None
                    },
                    
                    //   Groupings   //

                    "g" | "o" => {
                        let tmp_block = *last_block.clone();
                        last_block = Box::new(Vec::new());
                        Some((line, tmp_block))
                    },

                    //   Other   //

                    a => {println!("\n\nA: {} \n", a); None}
                }
            } else {
                None
            }
        }).collect();
        
        //   Create A New Container For The Model   //

        let name = name.unwrap_or_else(|| path.split('/').last().unwrap()).to_string();
        let this = Obj::new_empty(
            Some(name.as_str()), 
            parent, 
            position, 
            rotation, 
            scale_factor, 
            visible, 
            engine_globals.clone(), 
            script
        );

        //   Group Data   //

        // The current group/container being written to
        let mut current_group: Option<Arc<RwLock<dyn GameObject>>> = None;

        //   Loading Bar   //

        // standard output handle
        let mut stdout = stdout();

        // name of file and current group to be displayed
        let file_name = path.split('/').last().unwrap();
        let current_group_name: Option<&str> = None;

        // line number and total number of lines
        let mut line_n: usize = 0;
        let file_size = lines.len();

        // how often to update the loading bar
        let update = file_size / 100; // or just use 500 very small impact

        lines.into_iter().for_each(|line| {

            //   Loading Bar   //
            
            line_n += 1;
            
            // Get the terminal width
            let terminal_width = terminal_size::terminal_size().unwrap().0.0 as usize;

            // Don't always draw. unless on the final stretch
            let draw = update == 0 || line_n % update == 4; // || line_n >= file_size - update;

            // store the current_group name so it is 'static
            let future_s_current_group_name = &*current_group_name.unwrap_or("");

            // create the loading bar future
            let future = async {
                if draw {
                    term_ui::progress_bar(&mut stdout, file_name, future_s_current_group_name, line_n, file_size, terminal_width).await
                }
            };
            
            let mut e = line.0.split_whitespace();
            #[allow(clippy::or_fun_call)]
            let ty: &str = e.next().ok_or(format!("error on line {} of {}", line_n, path).as_str()).unwrap();


            // replaces flush
            if let Some(group) = current_group.clone() {
                let mut group_write_lock = group.write().unwrap();
                // create the triangle mesh and add it to the current group
                let triangle_mesh = TriangleMesh::from_obj_block(&line.1, &mut mtls_hashmap, (&vertex_positions, &texture_indices, &normals), engine_globals.queue.clone()).unwrap();
                
                group_write_lock.add_triangle_mesh( 
                    Arc::new(triangle_mesh)
                ).unwrap();
            }

            match &*ty {
                "g" => {
                    current_group = Some(this.clone() as Arc<RwLock<dyn GameObject>>);
                    let mut groups = e.collect::<Vec<&str>>();
                    while let Some(group_name) = groups.pop() {
                        let group = current_group.clone().unwrap();
                        let mut wlock_current_group = group.write().unwrap();
                        if let Ok(group) = wlock_current_group.get_child_by_name(group_name) {
                            drop(wlock_current_group);
                            current_group = Some(group);
                        } else {
                            let new_group = Obj::new_empty(
                                Some(group_name),
                                current_group.clone(),
                                None,
                                None,
                                None,
                                true,
                                engine_globals.clone(),
                                None
                            );
                            wlock_current_group.add_child(new_group.clone());
                            drop(wlock_current_group);
                            current_group = Some(new_group);
                        }
                    }
                },
                "o" => {
                    current_group = Some(this.clone() as Arc<RwLock<dyn GameObject>>);
                    
                    let object_name = e.next().expect("The o tag does not permit default/no names");
                    let new_object = Obj::new_empty(
                        Some(object_name),
                        current_group.clone(),
                        None,
                        None,
                        None,
                        true,
                        engine_globals.clone(),
                        None
                    );

                    current_group.clone().unwrap().write().unwrap().add_child(new_object.clone());
                    current_group = Some(new_object);
                },

                _ => unreachable!()
            }

            block_on(future);
        });

        let group = match current_group {
            Some(group) => group.clone(),
            None => this.clone()
        };

        let mut group_write_lock = group.write().unwrap();
        // create the triangle mesh and add it to the current group
        let triangle_mesh = TriangleMesh::from_obj_block(&last_block, &mut mtls_hashmap, (&vertex_positions, &texture_indices, &normals), engine_globals.queue.clone()).unwrap();
        
        group_write_lock.add_triangle_mesh( 
            Arc::new(triangle_mesh)
        ).unwrap();

        Ok(this)
    }
    
    // pub fn from_obj_old<'a>( // couple mistakes I caught are still in here
    //         name: Option<&str>, 
    //         path: &str, 

    //         parent: Option<Arc<RwLock<dyn GameObject>>>, 

    //         position: Option<Vector3<f32>>, 
    //         rotation: Option<Quaternion<f32>>,
    //         scale_factor: Option<Vector3<f32>>,
            
    //         visible: bool,

    //         engine_globals: EngineGlobals,

    //         script: Option<Box<Script<Self>>>) -> Result<Arc<RwLock<Self>>, &'a str>{
        
    //     //   Read In String Data   //
        
    //     let content = fs::read_to_string(path)
    //         .expect(&format!("Something went wrong when trying to read {}.", path));
    //     let lines = content.lines();

    //     //   Create A New Container For The Model   //

    //     let name = name.unwrap_or(path.split('/').last().unwrap()).to_string();
    //     let this = Obj::new_empty(
    //         Some(name.as_str()), 
    //         parent, 
    //         position, 
    //         rotation, 
    //         scale_factor, 
    //         visible, 
    //         engine_globals.clone(), 
    //         script
    //     );

    //     //   Group Data   //

    //     // The current group/container being written to
    //     let mut current_group = None;

    //     // Ordered mesh data
    //     let mut ordered_vertices = Vec::new();
    //     let mut ordered_normals = Vec::new();
    //     let mut ordered_texture_indices = Vec::new();

    //     let mut current_material: Option<Arc<Material>> = None;
        
    //     /// fn that flushes the ordered_vertices, ordered_normals, and ordered_texture_indices buffers 
    //     /// and writes the data into the current group
    //     /// Note it does not reset the current group
    //     fn flush(
    //             current_group: Option<Arc<RwLock<dyn GameObject>>>, 

    //             ordered_vertices: &mut Vec<Vertex>, 
    //             ordered_normals: &mut Vec<Normal>, 
    //             ordered_texture_indices: &mut Vec<TextureIndex>, 

    //             current_material: &mut Option<Arc<Material>>,
                
    //             engine_globals: EngineGlobals) {
    //         if let Some(group) = current_group.clone() {
    //             let group = group.clone();
    //             let mut group_write_lock = group.write().unwrap();
    //             // create the triangle mesh and add it to the current group
    //             let triangle_mesh = TriangleMesh::new(
    //                     ordered_vertices.clone(),
    //                     ordered_normals.clone(), 
    //                     ordered_texture_indices.clone(),
    //                     current_material.clone().unwrap_or(Arc::new(Material::default())),
    //                     engine_globals.queue
    //                 );
                
    //             group_write_lock.add_triangle_mesh( 
    //                 Arc::new(triangle_mesh)
    //             ).unwrap();

    //             // Reset the ordered vecs
    //             *ordered_vertices = Vec::new();
    //             *ordered_normals = Vec::new();
    //             *ordered_texture_indices = Vec::new();

    //             // reset the current material
    //             *current_material = None;
    //         }
    //     }

    //     //   Loading Bar   //

    //     // standard output handle
    //     let mut stdout = stdout();

    //     // name of file and current group to be displayed
    //     let file_name = path.split('/').last().unwrap();
    //     let current_group_name: Option<&str> = None;

    //     // line number and total number of lines
    //     let mut line_n: usize = 0;
    //     let file_size = lines.clone().count();

    //     // how often to update the loading bar
    //     let update = file_size / 100; // or just use 500 very small impact

    //     //   Data Pools   //

    //     let mut vertex_positions: Vec<Vertex> = Vec::new();
    //     let mut texture_indices: Vec<TextureIndex> = Vec::new();
    //     let mut normals: Vec<Normal> = Vec::new();
        
    //     let mut mtls_hashmap: HashMap<String, (Arc<Material>, Box<dyn GpuFuture>)> = HashMap::new();


        
    //     lines.for_each(|line| {

    //         //   Loading Bar   //
            
    //         line_n += 1;
            
    //         // Get the terminal width
    //         let terminal_width = terminal_size::terminal_size().unwrap().0.0 as usize;

    //         // Don't always draw. unless on the final stretch
    //         let draw = update == 0 || line_n % update == 4; // || line_n >= file_size - update;

    //         // store the current_group name so it is 'static
    //         let future_s_current_group_name = &*current_group_name.unwrap_or("");

    //         // create the loading bar future
    //         let future = async {
    //             if draw {
    //                 term_ui::progress_bar(&mut stdout, file_name, future_s_current_group_name, line_n, file_size, terminal_width).await
    //             }
    //         };
            
    //         if !line.is_empty() {
    //             let mut e = line.split_whitespace();
    //             let ty: &str = e.next().ok_or(format!("error on line {} of {}", line_n, path).as_str()).unwrap();
    //             match &*ty {

    //                 //   Groupings   //

    //                 "g" => {
    //                     flush(current_group.clone(), &mut ordered_vertices, &mut ordered_normals, &mut ordered_texture_indices, &mut current_material, engine_globals.clone());
    //                     current_group = Some(this.clone() as Arc<RwLock<dyn GameObject>>);
    //                     let mut groups = e.collect::<Vec<&str>>();
    //                     while let Some(group_name) = groups.pop() {
    //                         let group = current_group.clone().unwrap();
    //                         let mut wlock_current_group = group.write().unwrap();
    //                         if let Ok(group) = wlock_current_group.get_child_by_name(group_name) {
    //                             drop(wlock_current_group);
    //                             current_group = Some(group);
    //                         } else {
    //                             let new_group = Obj::new_empty(
    //                                 Some(group_name),
    //                                 current_group.clone(),
    //                                 None,
    //                                 None,
    //                                 None,
    //                                 true,
    //                                 engine_globals.clone(),
    //                                 None
    //                             );
    //                             wlock_current_group.add_child(new_group.clone());
    //                             drop(wlock_current_group);
    //                             current_group = Some(new_group);
    //                         }
    //                     }
    //                 },
    //                 "o" => {
    //                     flush(current_group.clone(), &mut ordered_vertices, &mut ordered_normals, &mut ordered_texture_indices, &mut current_material, engine_globals.clone());
    //                     current_group = Some(this.clone() as Arc<RwLock<dyn GameObject>>);
                        
    //                     let object_name = e.next().expect("The o tag does not permit default/no names");
    //                     let new_object = Obj::new_empty(
    //                         Some(object_name),
    //                         current_group.clone(),
    //                         None,
    //                         None,
    //                         None,
    //                         true,
    //                         engine_globals.clone(),
    //                         None
    //                     );

    //                     current_group.clone().unwrap().write().unwrap().add_child(new_object.clone());
    //                     current_group = Some(new_object);
    //                 },

    //                 //   Vertex Data   //

    //                 "v" => {
    //                     vertex_positions.push(Vertex::new(
    //                         e.next().unwrap().parse::<f32>().unwrap(),
    //                         e.next().unwrap().parse::<f32>().unwrap(),
    //                         e.next().unwrap().parse::<f32>().unwrap()
    //                     ));
    //                 },
    //                 "vt" => {
    //                     texture_indices.push(TextureIndex::new(
    //                         e.next().unwrap().parse::<f32>().unwrap(),
    //                         e.next().unwrap().parse::<f32>().unwrap()
    //                     ));
    //                 },
    //                 "vn" => {
    //                     normals.push(Normal::new(
    //                         e.next().unwrap().parse::<f32>().unwrap(),
    //                         e.next().unwrap().parse::<f32>().unwrap(),
    //                         e.next().unwrap().parse::<f32>().unwrap()
    //                     ));
    //                 },

    //                 //   Faces   //

    //                 "f" => {
    //                     let mut tris = Vec::new();
    //                     let mut i = 0;
    //                     for coord in &mut e {
    //                         i += 1;
    //                         if i > 3{ // not perfect but good enough for now
    //                             tris.push(tris[0]);
    //                             tris.push(tris[i - 2]);
    //                         }
    //                         tris.push(coord);
    //                     }

    //                     let mut vertex_fmt: i8 = -1;
    //                     let mut developing_normal: Vec<Vertex> = Vec::new();
    //                     for raw in tris{
    //                         let part = raw.split('/').collect::<Vec<&str>>();
                            
    //                         if vertex_fmt != part.len() as i8 {
    //                             if vertex_fmt == -1 {
    //                                 vertex_fmt = part.len() as i8;
    //                             }else {
    //                                 panic! ("Inconsistent face vertex format in {}.", path)
    //                             }
    //                         }
                            
    //                         let position = vertex_positions[part[0].parse::<usize>().unwrap() - 1_usize];

    //                         let mut texture_index = TextureIndex::new(0.0, 0.0);

    //                         if vertex_fmt > 1 && !part[1].is_empty() {
    //                             texture_index = texture_indices[part[1].parse::<usize>().unwrap() - 1_usize];
    //                         }
    //                         // I could have sworn I have separated these multiple times now
    //                         if developing_normal.is_empty() && vertex_fmt == 3 && !part[2].is_empty() { // a false second case is a result of improper formatting
    //                             ordered_normals.push(normals[part[2].parse::<usize>().unwrap() - 1_usize]);
    //                         } else {
    //                             developing_normal.push(position);
    //                         }

    //                         ordered_vertices.push(position);
    //                         ordered_texture_indices.push(texture_index);
    //                     }

    //                     if developing_normal.len() > 2 {
    //                         let normal = Normal::calculate_normal(&developing_normal[0], &developing_normal[1], developing_normal.last().unwrap());

    //                         for _ in 0..developing_normal.len() {
    //                             ordered_normals.push(normal);
    //                         }
    //                     }
    //                 },
                    
    //                 //   Materials   //

    //                 "mtllib" => {
    //                     let files = e.fold(String::new(), |mut a, b| {
    //                         a.reserve(b.len() + 1);
    //                         a.push_str(b);
    //                         a.push(' ');
    //                         a
    //                     });

    //                     let files = files.trim_end_matches(".mtl ");

    //                     files.split(".mtl ").for_each(|file| {
    //                         let file = "/".to_owned() + file + ".mtl";
    //                         let path = path.rsplitn(2, '/').nth(1).unwrap();
    //                         let path = path.to_owned() + file.as_str();

    //                         mtls_hashmap.extend(Material::from_mtllib(&path, engine_globals.clone().queue));
    //                     });
    //                 },
    //                 "usemtl" => {
    //                     let key = e.next().expect(format!("formatting error in {}", path).as_str());
    //                     let (cm, fut ) = mtls_hashmap.remove(key).unwrap();
    //                     current_material = Some(cm.clone());
    //                     if fut.queue().is_some() {
    //                         let _ = Arc::new(fut.then_signal_fence_and_flush().unwrap()).wait(None); // for now state does not matter                            
    //                     }
    //                     mtls_hashmap.insert(key.to_string(), (cm, sync::now(engine_globals.queue.device().clone()).boxed()));
                        
    //                 },

    //                 //   Other   //

    //                 "#" => (),
    //                 "" => (),
    //                 &_ => {
    //                     //panic!(format!("unsupported type on line {} of {}", line_n, path)); 
    //                 }
                    
    //                 //   TODO: Other Geometry   //

    //                 #[allow(unreachable_patterns)] // fr now just ignore it TODO: fix

    //                 "line" => {
    //                     /* remember that a normal of 0.0, 0.0, 0.0 is perfect because it is visible from any angle */
    //                     todo!();
    //                 },
    //             };
    //         }
            
    //         block_on(future);
    //     });

    //     flush(current_group, &mut ordered_vertices, &mut ordered_normals, &mut ordered_texture_indices, &mut current_material, engine_globals);

    //     Ok(this)
    // }
}