use std::any::Any;

use feo_oop_engine::scene::game_object::group::Group;



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
    feo_math::{
        rotation::quaternion::Quaternion,
        axes::Axes,
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
        KeyboardInput, 
        VirtualKeyCode, 
        WindowEvent
    }
};

#[derive(Clone, Copy, Debug, Global)] // Global is derivable
pub struct PlayerGlobals{
    time: Instant,
    cooldown: Instant,
    cooldown_time: f32,
    right: bool,
    left: bool,
    forward: bool,
    backward: bool,
    rot_right: bool,
    rot_left: bool,
    rot_up: bool,
    rot_down: bool,
    pew: bool,
}

#[macro_rules_attribute(start_script!)]
pub async fn start<'r>(this: Arc<RwLock<Group>>, _: EngineGlobals) -> Swap {
    let this = this.clone();
    let mut this = this.write().unwrap();

    this.set_globals(Box::new(PlayerGlobals{
        time: Instant::now(),
        cooldown: Instant::now(),
        cooldown_time: 0.25,
        right: false,
        left: false,
        forward: false,
        backward: false,
        rot_right: false,
        rot_left: false,
        rot_up: false,
        rot_down: false,
        pew: false,
    }) as Box<dyn Global>).unwrap();

    Swap::None
}

#[macro_rules_attribute(frame_script!)]
pub async fn frame<'r>(this: Arc<RwLock<Group>>, engine_globals: EngineGlobals) -> Swap {
    let (pewing, pos, rot) = {
        let this = this.clone();
        let mut this = this.write().unwrap();
        let mut globals = downcast!(*this.get_globals().unwrap(), dyn Global, PlayerGlobals);
        
        // Get delta and reset timer
        let elapsed = (globals.time.elapsed()); globals.time = Instant::now();
        let delta = (elapsed.as_secs() as f32 + (elapsed.subsec_nanos() as f32 / 1_000_000_000.0));
        let axes: Axes<f32> = Axes::<f32>::from(this.subspace.rotation);

        if globals.forward {
            this.subspace.center = this.subspace.center - axes.z * delta * 10.0;
        }
        if globals.backward {
            this.subspace.center = this.subspace.center + axes.z * delta * 10.0;
        }
        if globals.right {
            this.subspace.center = this.subspace.center + axes.x * delta * 10.0;
        }
        if globals.left {
            this.subspace.center = this.subspace.center - axes.x * delta * 10.0;
        }
        
        if globals.rot_up {
            this.subspace.rotate(Quaternion::new_axis_angle(axes.x, delta).unit_quaternion());
        }
        if globals.rot_down {
            this.subspace.rotate(Quaternion::new_axis_angle(-axes.x, delta).unit_quaternion());
        }
        if globals.rot_right {
            this.subspace.rotate(Quaternion::new(0.0, delta, 0.0, -1.0));
        }
        if globals.rot_left {
            this.subspace.rotate(Quaternion::new(0.0, delta, 0.0, 1.0));
        }

        let pewing = globals.pew && globals.cooldown.elapsed().subsec_nanos() as f32 / 1_000_000_000.0 >= globals.cooldown_time && { globals.cooldown = Instant::now(); true };

        this.set_globals(Box::new(globals) as Box<dyn Global>).unwrap();
        (pewing, this.subspace.center, this.subspace.rotation,)
    };

    if pewing {
        // unless adding to self you don't know if the lock has been lifted
        // to cleanly do this therefore rather than awaiting locks generate
        // an event, have the spawner listen for it and create the object.

        let mutex = engine_globals.event_loop_proxy;
        let x = mutex.lock().await;
        let _ = x.send_event(UserEvent::UserEvent(Arc::new(super::MyEvent::NewPew(pos, rot)) as Arc<dyn Any + Send + Sync>));
    }
 
    Swap::None
}

#[macro_rules_attribute(event_handler!)]
pub async fn event_handler<'r>(this: Arc<RwLock<Group>>, _: EngineGlobals, event: Event<'static, UserEvent<Arc<dyn Any + Send + Sync>>>) -> Swap {
    let this = this.clone();
    let mut this = this.write().unwrap();
    let mut globals = downcast!(*this.get_globals().unwrap(), dyn Global, PlayerGlobals);

    if let Event::WindowEvent { 
                event: WindowEvent::KeyboardInput {
                    input: KeyboardInput{
                        virtual_keycode: Some(key),
                        state,
                        ..
                    },
                    ..
                },
                ..
            } = event {
        let state = match state {
            winit::event::ElementState::Pressed => true,
            winit::event::ElementState::Released => false,
        };
        match key {
            VirtualKeyCode::W => globals.forward = state,
            VirtualKeyCode::A => globals.left = state,
            VirtualKeyCode::S => globals.backward = state,
            VirtualKeyCode::D => globals.right = state,

            VirtualKeyCode::Right => globals.rot_right = state,
            VirtualKeyCode::Left => globals.rot_left = state,
            VirtualKeyCode::Up => globals.rot_up = state,
            VirtualKeyCode::Down => globals.rot_down = state,
            
            VirtualKeyCode::Space => globals.pew = state,
            _ => {}
        }
    }
    
    this.set_globals(Box::new(globals) as Box<dyn Global>).unwrap(); 
    Swap::None
}