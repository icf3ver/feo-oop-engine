//! Scripting constructs
//! 
//! Note that the macros are in ::Macro.
//! 
//! ## Workflow
//! This library allows for the creation of scripts that govern 
//! the behavior of game_objects. Once the run function is called, 
//! it takes control of the thread until the window is closed. 
//! During this time it is still possible to create new game-objects 
//! and scripts that can be added to the scene.
//! 

use std::any::Any;

use {
    self::{
        executor::{Executor, Spawner}, 
        globals::{Global, EngineGlobals},
        swap::Swap
    },
    crate::{
        scene::{
            game_object::GameObject,
        },
        event::UserEvent,
    },
    std::{
        pin::Pin,
        sync::{
            Arc, 
            RwLock, 
            mpsc::sync_channel
        }
    },
    futures::future::BoxFuture,
    winit::event::Event,
};

pub mod globals;
pub mod executor;
pub mod swap;


/// A trait that provides scriptable functionality.
pub trait Scriptable {
    fn spawn_script_core(&mut self, this: Arc<RwLock<dyn GameObject>>, spawner: Spawner); // TODO: return result 
    fn spawn_script_handler(&mut self, this: Arc<RwLock<dyn GameObject>>, spawner: Spawner, event: Event<'static, UserEvent<Arc<dyn Any + Send + Sync>>>);
    fn get_globals(&self) -> Result<Box<dyn Global>, &'static str>;
    fn set_globals(&mut self, globals: Box<dyn Global>) -> Result<(), &'static str>;
}

pub type BoxedStartFn<T> = Pin<Box<fn(Arc<RwLock<T>>, EngineGlobals) -> BoxFuture<'static, Swap>>>;
pub type BoxedFrameFn<T> = Pin<Box<fn(Arc<RwLock<T>>, EngineGlobals) -> BoxFuture<'static, Swap>>>;
pub type BoxedEventHandlerFn<T> = Pin<Box<fn(Arc<RwLock<T>>, EngineGlobals, Event<'static, UserEvent<Arc<dyn Any + Send + Sync>>>) -> BoxFuture<'static, Swap>>>;

/// A struct that provides a container for a scripts datatypes.
pub struct Script<T> where T: ?Sized + Send + 'static{
    pub has_started: bool,
    pub globals: Option<Box<dyn Global>>,
    pub start: BoxedStartFn<T>,
    pub frame: BoxedFrameFn<T>,
    pub event_handler: Option<BoxedEventHandlerFn<T>>
}

impl<T: ?Sized + Send + 'static> std::fmt::Debug for Script<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Script")
            .field("has_started", &self.has_started)
            .field("globals", &self.globals)
            .field("start", &self.start)
            .field("frame", &self.frame)
            .field("event_handler", &self.event_handler).finish()
    }
}

impl<T> Script<T> where T: ?Sized + Send + 'static{
    pub fn new_boxed(
            start: BoxedStartFn<T>,
            frame: BoxedFrameFn<T>,
            event_handler: Option<BoxedEventHandlerFn<T>>) -> Box<Script<T>> {
        Box::new(Script{
            has_started: false,
            globals: None,
            start,
            frame,
            event_handler
        })
    }
}

impl<T> Clone for Script<T> where T: Clone + Send + 'static{
    fn clone(&self) -> Self {
        Script{
            has_started: self.has_started,
            globals: self.globals.clone(),
            start: self.start.clone(), //Box::pin(*self.start),
            frame: self.frame.clone(), //Box::pin(*self.frame),
            event_handler: self.event_handler.clone()
            /* match self.event_handler.as_deref() {
                Some(handler) => Some(Box::pin(*handler)),
                None => None
            } */
        }
    }
}

/// [backend] Creates a new executor and spawner for managing the asynchronous scripts. (TODO move to executor) 
pub fn new_executor_and_spawner(engine_globals: EngineGlobals) -> (Executor, Spawner) {
    const MAX_QUEUED_TASKS: usize = 10_000;
    let (task_sender, queue) = sync_channel(MAX_QUEUED_TASKS);
    (Executor {queue, ready: Vec::new() }, Spawner { task_sender, engine_globals })
}
