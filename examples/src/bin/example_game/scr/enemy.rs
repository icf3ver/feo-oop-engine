use feo_math::rotation::{Rotation, quaternion::Quaternion};
use feo_oop_engine::scene::game_object::{GameObject, group::Group, obj::Obj};

use crate::scr::TargetGlobal;

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
    },
    feo_math::axes::Axes,
    std::{
        sync::{
            Arc, 
            RwLock
        }, 
        time::Instant
    }
};

#[derive(Debug, Clone, Global)] // Global is derivable
pub struct EnemyGlobals{
    target: Arc<RwLock<Group>>,
    start: Instant,
    time: Instant,
    poof: bool,
}

#[macro_rules_attribute(start_script!)]
pub async fn start<'r>(this: Arc<RwLock<Obj>>, _: EngineGlobals) -> Swap {
    let this = this.clone();
    let mut this = this.write().unwrap();

    // pulling up global preset from initialization
    let target = downcast!(*this.get_globals().unwrap(), dyn Global, TargetGlobal).target;
    let target = downcast!(target, Arc<RwLock<dyn GameObject> >, Arc<RwLock<Group>>); // odd err tofix
    this.set_globals(Box::new(EnemyGlobals{
        target,
        start: Instant::now(),
        time: Instant::now(),
        poof: false
    }) as Box<dyn Global>).unwrap();

    Swap::None
}

#[macro_rules_attribute(frame_script!)]
pub async fn frame<'r>(this: Arc<RwLock<Obj>>, _: EngineGlobals) -> Swap {
    let this = this.clone();
    let mut this = this.write().unwrap();
    let mut globals = downcast!(*this.get_globals().unwrap(), dyn Global, EnemyGlobals);
    
    if globals.poof {
        Swap::Delete(this.get_id())
    } else {
        // Get delta and reset timer
        let elapsed = (globals.time.elapsed()); globals.time = Instant::now();
        let delta = (elapsed.as_secs() as f32 + (elapsed.subsec_nanos() as f32 / 1_000_000_000.0));

        this.subspace.rotation = Quaternion::look_at_xy(
            this.subspace.center, 
            globals.target.read().unwrap().get_subspace().center
        ); // deal with deadlock  /\/\/\

        let axes: Axes<f32> = Axes::<f32>::from(this.subspace.rotation);
        
        this.subspace.center = this.subspace.center + axes.z * delta * 11.0;

        this.set_globals(Box::new(globals) as Box<dyn Global>).unwrap();

        Swap::None
    }
}