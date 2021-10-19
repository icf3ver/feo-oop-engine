use feo_oop_engine::{scene::game_object::{group::Group, obj::Obj}, scripting::Script};

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
        registration::relation::Parent
    },
    std::{
        sync::{
            Arc, 
            RwLock
        }, 
        time::Instant
    },
};

#[derive(Debug, Clone, Global)] // Global is derivable
pub struct SpawnerGlobals{
    delay: Instant,
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
        None,
        true, 
        engine_globals,
        Some(Script::new_boxed(
            Box::pin(super::enemy::start),
            Box::pin(super::enemy::frame),
            None
        ))
    ).unwrap();
    entity.write().unwrap().set_globals(spawner.get_globals().unwrap()).unwrap();

    spawner.set_globals(Box::new(SpawnerGlobals{
        delay: Instant::now(), 
        entity: Box::new(entity.read().unwrap().clone()),
    }) as Box<dyn Global>).unwrap();

    Swap::None
}

#[macro_rules_attribute(frame_script!)]
pub async fn frame<'r>(this: Arc<RwLock<Group>>, _: EngineGlobals) -> Swap {
    let this = this.clone();
    let mut this = this.write().unwrap();
    let mut globals = downcast!(*this.get_globals().unwrap(), dyn Global, SpawnerGlobals);
    
    let since_start = globals.delay.elapsed().as_secs();

    if (since_start != 0) {
        let new_entity = *globals.entity.clone();
        this.add_child(Arc::new(RwLock::new(new_entity)));
        globals.delay = Instant::now();
    }

    this.set_globals(Box::new(globals) as Box<dyn Global>).unwrap();
    Swap::None
}