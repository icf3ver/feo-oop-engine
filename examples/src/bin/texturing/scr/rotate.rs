use {
    feo_oop_engine::{
        scene::{
            game_object::obj::Obj,
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
    feo_math::{
        rotation::quaternion::Quaternion,
        linear_algebra::vector3::Vector3,
    },
    std::{
        sync::{
            Arc, 
            RwLock
        }, 
        time::Instant
    }
};

#[derive(Clone, Copy, Debug, Global)] // Global is derivable
pub struct Timer(Instant);

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
    let mut global_timer = downcast!(*this.get_globals().unwrap(), dyn Global, Timer);
    
    // Get delta and reset timer
    let elapsed = (global_timer.0.elapsed()); global_timer.0 = Instant::now(); 
    let rotation = (elapsed.as_secs() as f64 + (elapsed.subsec_nanos() as f64 / 1_000_000_000.0));

    this.subspace.rotate(Quaternion::new_axis_angle(Vector3(0.0, 1.0, 0.0), rotation as f32));

    this.set_globals(Box::new(global_timer) as Box<dyn Global>).unwrap();
    Swap::None
}

// pub async fn event_handler(event: Event<'a>){ // TODO
//     match event{
//         // ...
//     }
// }