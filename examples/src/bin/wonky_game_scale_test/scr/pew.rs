use feo_oop_engine::scene::game_object::{GameObject, obj::Obj};

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

#[derive(Clone, Copy, Debug, Global)] // Global is derivable
pub struct PewGlobals{
    start: Instant,
    time: Instant,
}

#[macro_rules_attribute(start_script!)]
pub async fn start<'r>(this: Arc<RwLock<Obj>>, _: EngineGlobals) -> Swap {
    let this = this.clone();
    let mut this = this.write().unwrap();

    this.set_globals(Box::new(PewGlobals{
        start: Instant::now(),
        time: Instant::now(),
    }) as Box<dyn Global>).unwrap();

    Swap::None
}

#[macro_rules_attribute(frame_script!)]
pub async fn frame<'r>(this: Arc<RwLock<Obj>>, _: EngineGlobals) -> Swap {
    let this = this.clone();
    let mut this = this.write().unwrap();
    let mut globals = downcast!(*this.get_globals().unwrap(), dyn Global, PewGlobals);
    
    if globals.start.elapsed().as_secs() >= 5 {
        Swap::Delete(this.get_id())
    } else {
        // Get delta and reset timer
        let elapsed = (globals.time.elapsed()); globals.time = Instant::now();
        let delta = (elapsed.as_secs() as f32 + (elapsed.subsec_nanos() as f32 / 1_000_000_000.0));
        let axes: Axes<f32> = Axes::<f32>::from(this.subspace.rotation);
        
        this.subspace.center = this.subspace.center - axes.z * delta * 20.0;

        this.set_globals(Box::new(globals) as Box<dyn Global>).unwrap();

        Swap::None
    }
}