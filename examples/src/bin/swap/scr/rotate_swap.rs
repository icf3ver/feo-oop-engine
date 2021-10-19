use {
    feo_oop_engine::{
        scene::game_object::{
            GameObject, 
            obj::Obj
        },
        scripting::{
            Scriptable,
            globals::{
                Global, 
                EngineGlobals
            }, 
            swap::Swap
        }
    },
    feo_math::{
        rotation::quaternion::Quaternion,
        linear_algebra::vector3::Vector3
    },
    std::{
        sync::{
            Arc, 
            RwLock
        }, 
        time::Instant
    }
};

#[derive(Debug, Clone, Global)] // Global is derivable
pub struct ObjGlobals{
    frame: Instant,
    since_start: Instant,
    preloaded: Arc<RwLock<Obj>>,
}

#[macro_rules_attribute(start_script!)]
pub async fn start<'r>(this: Arc<RwLock<Obj>>, engine_globals: EngineGlobals) -> Swap { 
    let this = this.clone();
    let mut this = this.write().unwrap();

    this.set_globals(Box::new(ObjGlobals{
        frame: Instant::now(), 
        since_start: Instant::now(),
        preloaded: Obj::from_obj( // err
            Some("oloid"), 
            "assets/standard-assets/models/shapes/cube.obj",
            None,
            Some(Vector3::new( 2.0, 0.0, 2.0)),
            None,
            None, //Some(Vector3(0.025, 0.025, 0.025)),
            true,
            engine_globals,
            None,//Some(Script::new_boxed(Box::pin(start), Box::pin(frame)))
        ).unwrap(),
    }) as Box<dyn Global> ).unwrap();

    Swap::None
}

#[macro_rules_attribute(frame_script!)]
pub async fn frame<'r>(this: Arc<RwLock<Obj>>, _: EngineGlobals) -> Swap {
    let this = this.clone();
    let mut this = this.write().unwrap();
    let mut global_timer = downcast!(this.get_globals().unwrap(), dyn Global, ObjGlobals);
    
    // Get delta and reset timer
    let elapsed = (global_timer.frame.elapsed()); global_timer.frame = Instant::now(); 
    let rotation = (elapsed.as_secs() as f64 + (elapsed.subsec_nanos() as f64 / 1_000_000_000.0));

    this.subspace.rotate(Quaternion::new_axis_angle(Vector3(0.0, 1.0, 0.0), rotation as f32));

    let elapsed = (global_timer.since_start.elapsed());
    let since_start = elapsed.as_secs() + 1;

    // println!("{}",since_start%11);
    if (since_start%11 == 0) {
        return Swap::SwapFull(this.get_id(), global_timer.preloaded)
    }
    this.set_globals(Box::new(global_timer) as Box<dyn Global>).unwrap();
    Swap::None
}