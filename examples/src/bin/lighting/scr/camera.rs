use std::any::Any;

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
            swap::Swap
        },
        event::UserEvent
    },
    feo_math::{
        rotation::quaternion::Quaternion,
        axes::Axes,
        linear_algebra::vector3::Vector3,
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
pub struct CameraGlobals{
    time: Instant,
    right: bool,
    left: bool,
    forward: bool,
    backward: bool,
    rot_right: bool,
    rot_left: bool,
    rot_up: bool,
    rot_down: bool,
    
    set_rot: bool,
}

#[macro_rules_attribute(start_script!)]
pub async fn start<'r>(this: Arc<RwLock<FpvCamera>>, _: EngineGlobals) -> Swap {
    let this = this.clone();
    let mut this = this.write().unwrap();

    this.set_globals(Box::new(CameraGlobals{
        time: Instant::now(),
        right: false,
        left: false,
        forward: false,
        backward: false,
        rot_right: false,
        rot_left: false,
        rot_up: false,
        rot_down: false,
        set_rot: false,
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
    let delta = (elapsed.as_secs() as f32 + (elapsed.subsec_nanos() as f32 / 1_000_000_000.0));
    //println!("{}", 1.0/delta);
    let axes: Axes<f32> = Axes::<f32>::from(this.subspace.rotation);
    // println!("{:?}", axes);
    
    if globals.forward {
        this.subspace.center = this.subspace.center - axes.z * delta * 10.0; // camera is pointing along the -z
    }
    if globals.backward {
        this.subspace.center = this.subspace.center + axes.z * delta * 10.0;
    }
    if globals.right {
        this.subspace.center = this.subspace.center + axes.x * delta * 10.0; // remember that the camera is pointing along the -z normally x is left
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
        this.subspace.rotate(Quaternion::new(0.0, delta, 0.0, -1.0).unit_quaternion());
    }
    if globals.rot_left {
        this.subspace.rotate(Quaternion::new(0.0, delta, 0.0, 1.0).unit_quaternion());
    }

    if globals.set_rot {
        this.subspace.rotation = Quaternion::camera_look_at(this.subspace.center, Vector3(0.0, 0.0, 0.0));
    }

    this.set_globals(Box::new(globals) as Box<dyn Global>).unwrap();    
    Swap::None
}

#[macro_rules_attribute(event_handler!)]
pub async fn event_handler<'r>(this: Arc<RwLock<FpvCamera>>, _: EngineGlobals, event:  Event<'static, UserEvent<Arc<dyn Any + Send + Sync>>>) -> Swap {
    let this = this.clone();
    let mut this = this.write().unwrap();
    let mut globals = downcast!(*this.get_globals().unwrap(), dyn Global, CameraGlobals);

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
            
            VirtualKeyCode::Space => globals.set_rot = state,
            
            _ => {}
        }
    }

    this.set_globals(Box::new(globals) as Box<dyn Global>).unwrap(); 
    Swap::None
}