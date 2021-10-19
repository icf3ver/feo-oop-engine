use std::any::Any;

use feo_math::linear_algebra::vector3::Vector3;
use feo_oop_engine::{scene::game_object::{group::Group, obj::Obj}, scripting::Script};

use super::MyEvent;

use crate::feo_oop_engine::registration::relation::Parent;

use {
    feo_oop_engine::{
        scripting::{
            globals::{
                Global,
                EngineGlobals
            },
            Scriptable, 
            swap::Swap
        },
        event::UserEvent
    },
    std::sync::{
        Arc, 
        RwLock
    },
    winit::event::Event
};

#[derive(Debug, Clone, Global)] // Global is derivable
pub struct SpawnerGlobals{
    entity: Box<Obj>,
}

#[macro_rules_attribute(start_script!)]
pub async fn start<'r>(this: Arc<RwLock<Group>>, engine_globals: EngineGlobals) -> Swap {
    let spawner = this.clone();
    let mut spawner = spawner.write().unwrap();

    let entity = Obj::from_obj(
        Some("pew"),
        "assets/standard-assets/models/shapes/cube.obj", 
        Some(this.clone()),
        None,
        None,
        Some(Vector3::new(0.25, 0.25, 1.0)),
        true, 
        engine_globals,
        Some(Script::new_boxed(
            Box::pin(super::pew::start),
            Box::pin(super::pew::frame),
            None
        ))
    ).unwrap();

    spawner.set_globals(Box::new(SpawnerGlobals{
        entity: Box::new(entity.read().unwrap().clone()),
    }) as Box<dyn Global>).unwrap();

    Swap::None
}

#[macro_rules_attribute(frame_script!)]
pub async fn frame<'r>(_: Arc<RwLock<Group>>, _: EngineGlobals) -> Swap {
    Swap::None
}

#[macro_rules_attribute(event_handler!)]
pub async fn event_handler<'r>(this: Arc<RwLock<Group>>, _: EngineGlobals, event: Event<'static, UserEvent<Arc<dyn Any + Send + Sync>>>) -> Swap {
    let this = this.clone();
    let mut this = this.write().unwrap();
    let globals = downcast!(*this.get_globals().unwrap(), dyn Global, SpawnerGlobals);

    if let Event::UserEvent(
                UserEvent::UserEvent(
                    my_event
                )
            ) = event {
        let my_event = downcast!(my_event, Arc<dyn Any>, Arc<MyEvent>);
        match *my_event {
            MyEvent::NewPew(center, rotation) => {
                let mut new_entity = *globals.entity.clone();
                new_entity.subspace.center = center;
                new_entity.subspace.rotation = rotation;
                this.add_child(Arc::new(RwLock::new(new_entity)));
            }
        }
    }

    this.set_globals(Box::new(globals) as Box<dyn Global>).unwrap(); 
    Swap::None
}