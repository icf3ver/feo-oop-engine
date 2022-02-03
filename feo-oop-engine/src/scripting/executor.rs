//! Managing the async script.
//! 
//! TODO
//! 
use {
    super::{
        swap::Swap,
        globals::EngineGlobals,
    },
    crate::{
        scene::{
            Scene,
            game_object::GameObject,
        },
        registration::{
            relation::{
                Parent, 
                ParentWrapper
            },
            id::ID,
        }
    },
    std::{
        future::Future,
        sync::mpsc::{
            Receiver,
            SyncSender
        },
        collections::HashMap,
        sync::{
            Arc,
            Mutex, 
            RwLock
        },
        task::{
            Context, 
            Poll
        }
    },
    futures::{
        future::{
            BoxFuture, 
            FutureExt
        },
        task::{
            waker_ref,
            ArcWake
        },
    },
    rayon::slice::ParallelSliceMut,
};

pub struct Executor {
    pub ready: Vec<Arc<RwLock<Box<dyn GameObject>>>>,
    pub queue: Receiver<Arc<Task>>,
}

#[derive(Clone)]
pub struct Spawner {
    pub engine_globals: EngineGlobals, // distributed to all frames
    pub task_sender: SyncSender<Arc<Task>>,
}

pub struct Task{
    future: Mutex<Option<BoxFuture<'static, Swap>>>,
    task_sender: SyncSender<Arc<Task>>,
}

impl Spawner {
    pub fn spawn(&self, future: impl Future<Output = Swap> + 'static + Send) {
        let future = future.boxed();
        let task = Arc::new(Task {
            future: Mutex::new(Some(future)),
            task_sender: self.task_sender.clone(),
        });
        self.task_sender.send(task).expect("too many tasks queued");
    }
}

impl ArcWake for Task {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        let cloned = arc_self.clone();
        arc_self
            .task_sender
            .send(cloned)
            .expect("too many tasks queued");
    }
}

impl Executor {
    pub fn run(&self, scene: Arc<RwLock<Scene>>) {
        let mut swaps = Vec::new();
        
        while let Ok(task) = self.queue.recv() {
            let mut future_slot = task.future.lock().unwrap();
            if let Some(mut future) = future_slot.take() {
                let waker = waker_ref(&task);
                let context = &mut Context::from_waker(&*waker);
                match future.as_mut().poll(context) {
                    Poll::Pending => { *future_slot = Some(future); },
                    Poll::Ready(swap_status) => {
                        match swap_status {
                            Swap::None => {},
                            _ => { swaps.push(swap_status); }
                        }
                    },
                }
            }
        }

        if !swaps.is_empty() {
            Self::sort(&mut swaps, scene.clone());

            swaps.into_iter().for_each( |swap| {
                match swap {
                    // By swapping out the physical pointers rather than the interior it makes it possible to store a pointer to swap back in later
                    Swap::SwapParent(id, replacement) => { //
                        let mut found = false;
                        let mut queue = scene.read().unwrap().get_children().clone();
                        
                        while let (Some(old), false) = (queue.pop(), found) {
                            let read = old.read().unwrap(); // will not actually be changing the target object
                            queue.append(&mut read.get_children());
                            if read.get_id() == id {
                                let children = read.get_children();
                                let parent = read.get_parent();
    
                                // direct children to the replacement
                                children.into_iter().for_each(|child| {
                                    // set child object's parent object to be the new object
                                    unsafe { child.write().unwrap().set_parent(ParentWrapper::GameObject(replacement.clone()))};
                                    
                                    // set replacement's child objects to be the new object
                                    replacement.write().unwrap().add_child(child);
                                });
    
                                // direct parent to the replacement
                                match parent.clone() { // shadowing
                                    ParentWrapper::GameObject(parent) => unsafe { 
                                        parent.write().unwrap().replace_child(old.clone(), replacement.clone()).unwrap();
                                    },
                                    ParentWrapper::Scene(p) => unsafe {
                                        p.write().unwrap().replace_child(old.clone(), replacement.clone()).unwrap();
                                    }
                                }
                                unsafe { replacement.write().unwrap().set_parent(parent); }
    
                                found = true;
                            }
                        }
                        
                        // Check Camera
                        let mut scene_rw = scene.write().unwrap();
                        if let Some(old_camera) = scene_rw.main_camera.clone() {
                            let old_camera_read = old_camera.read().unwrap();
                            if id == old_camera_read.get_id() { 
                                
                                // Note that swapping a main camera parent does not make sense unless the main camera is part of the game_object tree since those objects would not be drawn.
                                // If your intent is to toggle this group the objects and toggle their visibility attribute instead.
                                assert!(found);
                                // // direct children to the replacement
                                // let children = old_camera_read.get_children();
                                // let parent = old_camera_read.get_parent();
                                // let as_gameobject = old_camera_read.cast_gameobject_arc_rwlock(old_camera.clone());
                                // children.into_iter().for_each(|child| {
                                //     // set child object's parent object to be the new object
                                //     unsafe { child.write().unwrap().set_parent(ParentWrapper::GameObject(replacement.clone())) };
                                //     // set replacement's child objects to be the new object
                                //     replacement.write().unwrap().add_child(child);
                                // });
    
                                // // direct parent to the replacement
                                // match parent.clone() {
                                //     ParentWrapper::GameObject(p) => unsafe { 
                                //         p.write().unwrap().replace_child(as_gameobject.clone(), replacement.clone()).unwrap();
                                //     },
                                //     ParentWrapper::Scene(_) => unsafe { 
                                //         scene_rw.replace_child(as_gameobject, replacement.clone()).unwrap();
                                //     },
                                // }
                                // unsafe { replacement.write().unwrap().set_parent(parent); }
    
                                // replace the old camera with the replacement in the main_camera slot
                                scene_rw.main_camera = Some(replacement.clone().read().unwrap().cast_camera_arc_rwlock(replacement.clone()).unwrap()); 
    
                                found = true;
                            }
                        }
                        
                        if !found {
                            panic!("could not find the ID.");
                        } 
                    },
                    // cutting is fine here
                    Swap::SwapFull(id, replacement) => {
                        let mut found = false;
                        let mut queue = scene.read().unwrap().get_children().clone();
                        
                        while let (Some(old), false) = (queue.pop(), found) {
                            let read = old.read().unwrap(); // will not actually be changing the target object
                            queue.append(&mut read.get_children());
                            if read.get_id() == id {
                                let parent = read.get_parent();
    
                                // direct parent to the replacement
                                match parent.clone() { // shadowing
                                    ParentWrapper::GameObject(parent) => unsafe { 
                                        parent.write().unwrap().replace_child(old.clone(), replacement.clone()).unwrap();
                                    },
                                    ParentWrapper::Scene(p) => unsafe {
                                        p.write().unwrap().replace_child(old.clone(), replacement.clone()).unwrap();
                                    }
                                }
                                unsafe { replacement.write().unwrap().set_parent(parent); }
    
                                found = true;
                            }
                        }
                        
                        // Check Camera
                        let mut scene_rw = scene.write().unwrap();
                        if let Some(old_camera) = scene_rw.main_camera.clone() {
                            let old_camera_read = old_camera.read().unwrap();
                            if id == old_camera_read.get_id() {
                                let parent = old_camera_read.get_parent();
                                // Not needed because It is done in above loop based on children of parents
                                // let as_gameobject = old_camera_read.cast_gameobject_arc_rwlock(old_camera.clone());
                                // match parent.clone() {
                                //     ParentWrapper::GameObject(p) => unsafe { 
                                //         p.write().unwrap().replace_child(as_gameobject.clone(), replacement.clone()).unwrap();
                                //     },
                                //     ParentWrapper::Scene(_) => unsafe { 
                                //         scene_rw.replace_child(as_gameobject.clone(), replacement.clone()).unwrap();
                                //     },
                                // }

                                // needed in the case that someone does not wish to have scripts on a main camera with scene parent and thus does not 
                                // tell parent that camera is its child.
                                unsafe { replacement.write().unwrap().set_parent(parent); }

                                scene_rw.main_camera = Some(replacement.clone().read().unwrap().cast_camera_arc_rwlock(replacement.clone()).unwrap()); 

                                found = true;
                            }
                        }
                        
                        if !found {
                            panic!("could not find the ID.");
                        } 
                    },
                   
                    Swap::Delete(id) => {
                        let mut found = false;
                        let mut queue = scene.read().unwrap().get_children().clone();
                        while let Some(old) = queue.pop() {
                            let read = old.read().unwrap(); // will not actually be changing the target object
                            queue.append(&mut read.get_children());
                            if read.get_id() == id {
                                let parent = read.get_parent();
    
                                // direct parent to the replacement
                                match parent.clone() { // shadowing
                                    ParentWrapper::GameObject(p) => unsafe { 
                                        p.write().unwrap().remove_child(old.clone()).unwrap();
                                    },
                                    ParentWrapper::Scene(p) => unsafe {
                                        p.write().unwrap().remove_child(old.clone()).unwrap();
                                    }
                                }
    
                                found = true;
                            }
                        }
                        
                        // Check Camera
                        if let Some(camera) = scene.read().unwrap().main_camera.clone() {
                            if id == camera.read().expect("camera in use").get_id() { 
                                panic!("You can not delete the main camera. It can only be replaced.");
                            }
                        }
                        
                        if !found {
                            panic!("could not find the ID.");
                        } 
                    },
                    _ => { panic!("not possible"); }
                }
            });
        }
    }

    #[inline(always)]
    fn order_id(game_object: Arc<RwLock<dyn GameObject>>, n: &mut usize) -> Vec<(ID, usize)>{
        let read_lock = game_object.read().unwrap();
        let mut result: Vec<(ID, usize)> = vec![(read_lock.get_id(), *n)];
        read_lock.get_children().into_iter().for_each(|child| {
            *n -= 1;
            result.append(&mut Self::order_id(child, n));
        });
        result
    }

    fn sort(swaps: &mut Vec<Swap>, scene: Arc<RwLock<Scene>>) {
        let read_lock = scene.read().unwrap();
        
        // sort by parent, children... , parent2, children2...
        let mut order: Vec<(ID, usize)> = Vec::new();

        // reverse to get  ...children2, parent2, ...children, parent
        let mut n = usize::MAX;
        read_lock.get_children().into_iter().for_each(|child| {
            order.append(&mut Self::order_id(child, &mut n));
        });

        let hash_map: HashMap<ID, usize> = order.into_iter().collect();

        swaps.par_sort_unstable_by_key(|swap| hash_map.get(swap.get_id().unwrap()));
    }
}
