use {
    feo_oop_engine::{
        scene::{
            game_object::camera::fpv_camera::FpvCamera,
        },
        scripting::{
            Scriptable, 
            swap::Swap,
            globals::{
                Global,
                EngineGlobals
            }
        }
    },
    feo_math::{
        rotation::quaternion::Quaternion,
        linear_algebra::vector3::Vector3,
        rotation::Rotation,
        axes::Axes,
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
pub struct CameraGlobals{
    time: Instant,
    distance_from_center: f32,
    // ...
}

#[macro_rules_attribute(start_script!)]
pub async fn start<'r>(this: Arc<RwLock<FpvCamera>>, _: EngineGlobals) -> Swap {
    let this = this.clone();
    let mut this = this.write().unwrap();

    this.set_globals(Box::new(CameraGlobals{
        time: Instant::now(),
        distance_from_center: 10.0,
    }) as Box<dyn Global>).unwrap();

    Swap::None
}

#[macro_rules_attribute(frame_script!)]
pub async fn frame<'r>(this: Arc<RwLock<FpvCamera>>, _: EngineGlobals) -> Swap {
    let this = this.clone();
    let mut this = this.write().unwrap();
    let mut globals = downcast!(*this.get_globals().unwrap(), dyn Global, CameraGlobals);
    
    // Get delta and reset timer
    let elapsed = (globals.time.elapsed()); globals.time = Instant::now();
    let rotation = (elapsed.as_secs() as f64 + (elapsed.subsec_nanos() as f64 / 1_000_000_000.0));

    let axes: Axes<f32> = Axes::<f32>::from(this.subspace.rotation);

    this.subspace.center = (this.subspace.center + axes.x.normalize(Some(rotation.tan() as f32 * globals.distance_from_center)))
        .normalize(Some(globals.distance_from_center));

    this.subspace.rotation = Quaternion::camera_look_at_xy(this.subspace.center, Vector3(0.0, 0.0, 0.0));

    this.set_globals(Box::new(globals) as Box<dyn Global>).unwrap();
    
    Swap::None
}