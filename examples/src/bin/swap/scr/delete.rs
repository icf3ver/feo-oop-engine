use {
    feo_oop_engine::{
        scene::game_object::{
            GameObject,
            obj::Obj,
        },
        scripting::{
            Scriptable, 
            globals::{
                Global, 
                EngineGlobals
            },
            swap::Swap,
        }
    },
    std::{
        sync::{
            Arc, 
            RwLock
        }, 
        time::Instant
    }
};

#[derive(Clone, Debug, Global)] // Global is derivable
struct Timer(Instant);

#[macro_rules_attribute(start_script!)]
pub async fn start<'r>(this: Arc<RwLock<Obj>>, _: EngineGlobals) -> Swap { 
    let this = this.clone();
    let mut this = this.write().unwrap();

    this.set_globals(Box::new(Timer(Instant::now())) as Box<dyn Global>).unwrap();

    Swap::None
}

#[macro_rules_attribute(frame_script!)]
pub async fn frame<'r>(this: Arc<RwLock<Obj>>, _: EngineGlobals) -> Swap {
    let this = this.clone();
    let mut this = this.write().unwrap();
    let global_timer = downcast!(this.get_globals().unwrap(), dyn Global, Timer);

    let since_start = global_timer.0.elapsed().as_secs() + 1;

    println!("{}", since_start%21 );
    if (since_start%21 == 0) {
        return Swap::Delete(this.get_id());
    }
    this.set_globals(Box::new(global_timer) as Box<dyn Global>).unwrap();
    Swap::None
}