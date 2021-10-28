use {
    feo_oop_engine::{
        scene::{
            game_object::camera::fpv_camera::FpvCamera,
        },
        scripting::{
            globals::{
                Global,
                EngineGlobals
            },
            Scriptable, 
            swap::Swap,
        },
        event::UserEvent
    },
    std::{
        sync::{
            Arc, 
            RwLock
        }, 
        time::Instant
    },
    winit::event::{
        Event, 
        WindowEvent
    }
};

#[derive(Clone, Copy, Debug, Global)] // Global is derivable
pub struct CameraGlobals{
    time: Instant,
}

#[macro_rules_attribute(start_script!)]
pub async fn start<'r>(this: Arc<RwLock<FpvCamera>>, _: EngineGlobals) -> Swap {
    let this = this.clone();
    let mut this = this.write().unwrap();

    this.set_globals(Box::new(CameraGlobals{
        time: Instant::now(),
    }) as Box<dyn Global>).unwrap();

    Swap::None
}

#[macro_rules_attribute(frame_script!)]
pub async fn frame<'r>(this: Arc<RwLock<FpvCamera>>, engine_globals: EngineGlobals) -> Swap {
    let a = {
        let this = this.clone();
        let this = this.write().unwrap();
        let globals = downcast!(*this.get_globals().unwrap(), dyn Global, CameraGlobals);

        let elapsed = globals.time.elapsed().as_secs();

        (elapsed+1)%15 == 0
    };
    match a { 
        true => {
            let mutex = engine_globals.event_loop_proxy;
            let window_id = engine_globals.surface.window().id();
            let x = mutex.lock().await;
            let event = UserEvent::WinitEvent(Event::WindowEvent{ window_id, event: WindowEvent::CloseRequested});
            x.send_event(event).unwrap();
        }, 
        false => {}
    };
    Swap::None
}