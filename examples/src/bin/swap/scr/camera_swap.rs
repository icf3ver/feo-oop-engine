use {
    feo_oop_engine::{
        scene::game_object::{
            camera::fpv_camera::FpvCamera,
            GameObject
        },
        scripting::{
            Scriptable,
            swap::Swap,
            globals::{
                Global,
                EngineGlobals,
            }
        }
    },
    feo_math::{
        rotation::quaternion::Quaternion,
        rotation::Rotation,
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

#[derive(Debug, Clone, Global)] // Global is derivable
pub struct CameraGlobals{
    since_start: Instant,
    preloaded: Arc<RwLock<FpvCamera>>,
}

#[macro_rules_attribute(start_script!)]
pub async fn start<'r>(this: Arc<RwLock<FpvCamera>>, engine_globals: EngineGlobals) -> Swap {
    let this = this.clone();
    let mut this = this.write().unwrap();

    let dimensions: [u32; 2] = engine_globals.surface.window().inner_size().into();
    this.set_globals(Box::new(CameraGlobals{
        since_start: Instant::now(),
        preloaded: FpvCamera::new(
            Some("main camera"),
            true,
            None,
            Some(Vector3(5.0, 5.0, 10.0)),
            Some(Quaternion::camera_look_at_xy(Vector3(5.0, 5.0, 10.0), Vector3(0.0, 0.0, 0.0))),
            None,
            None,
            120,
            0.1,
            100.0, 
            dimensions[0] as f32 / dimensions[1] as f32, 
            None,
            engine_globals
        ).unwrap(),
    }) as Box<dyn Global>).unwrap();

    Swap::None
}

#[macro_rules_attribute(frame_script!)]
pub async fn frame<'r>(this: Arc<RwLock<FpvCamera>>, _: EngineGlobals) -> Swap {
    let this = this.clone();
    let mut this = this.write().unwrap();
    let globals = downcast!(*this.get_globals().unwrap(), dyn Global, CameraGlobals);
    
    let since_start = globals.since_start.elapsed().as_secs() + 1;

    if (since_start%16 == 0) {
        return Swap::SwapFull(this.get_id(), globals.preloaded);
    }
    this.set_globals(Box::new(globals) as Box<dyn Global>).unwrap();
    Swap::None
}