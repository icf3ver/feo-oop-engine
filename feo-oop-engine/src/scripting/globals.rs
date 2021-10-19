use std::fmt::Debug;

use vulkano::device::Queue;

use crate::registration::id::IDSystem;

use {
    crate::{
        scene::Scene,
        event::UserEvent,
    },
    std::{
        any::Any,
        sync::{Arc, RwLock}
    },
    vulkano::swapchain::Surface,
    winit::{
        event_loop::EventLoopProxy,
        window::Window
    }
};

#[allow(clippy::type_complexity)]
#[derive(Clone, Debug, Global)]
pub struct EngineGlobals {
    pub queue: Arc<Queue>,
    pub surface: Arc<Surface<Window>>,
    pub scene: Arc<RwLock<Scene>>,
    pub event_loop_proxy: Arc<futures::lock::Mutex<EventLoopProxy<UserEvent<Arc<dyn Any + 'static + Send + Sync>>>>>,
    pub id_system: IDSystem,
}

pub trait Global: GlobalClone + Debug + Any + Send + Sync + 'static {
    fn as_any(&self) -> &dyn Any;
}

pub trait GlobalClone {
    fn clone_global(&self) -> Box<dyn Global>;
}

impl<T> GlobalClone for T where T: 'static + Global + Clone {
    fn clone_global(&self) -> Box<dyn Global> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn Global> {
    fn clone(&self) -> Self {
        self.clone_global()
    }
}