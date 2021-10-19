extern crate feo_oop_engine;

use {
    feo_oop_engine::{
        FeoEngine, 
        scene::{
            game_object::{
                camera::fpv_camera::FpvCamera, 
                light::{
                    ambient_light::AmbientLight, 
                    directional_light::DirectionalLight
                }, 
                obj::Obj
            }, 
            Scene, 
        }, 
        registration::relation::Parent,
        components::RGB
    },
    feo_math::{
        linear_algebra::vector3::Vector3, 
        rotation::quaternion::Quaternion,
        rotation::Rotation
    }
};

fn main(){
    // Create a Scene with a default worldspace
    // All objects will be created in worldspace.
    //
    // Y  Z  <|
    // |/_ X
    //
    // If you scale the worldspace all objects 
    // existing in the space the objects existing
    // in worldspace maintain their same coordinates 
    // in the space and therefore scale with the worldspace
    //
    // Yx2   /|
    // |  Z  \|
    // |/_ X
    //
    // The same is true for rotations and translations
    //
    let scene = Scene::new(None);

    // Create an engine with the defined scene.
    let mut engine = FeoEngine::init(scene, None);
    
    // Create an ambient light
    let ambient_light = AmbientLight::new(
        Some("ambient light"), 
        None, 
        1.0f32, 
        RGB::new(0.2, 0.2, 0.2), 
        None, 
        None, 
        None, 
        None, 
        engine.globals.clone()
    );
    engine.scene.write().unwrap().add_child(ambient_light);

    // Create a directional light
    let directional_light = DirectionalLight::new(
        Some("directional light"), 
        None,
        1.0f32, 
        RGB::new(0.4, 0.4, 0.4), 
        None, 
        Some(Quaternion::new(0.5, 1.0, 0.5, std::f32::consts::PI)), 
        None, 
        None, 
        engine.globals.clone()
    );
    engine.scene.write().unwrap().add_child(directional_light);

    // Create a camera and 
    let dimensions: [u32; 2] = engine.globals.surface.window().inner_size().into();
    let main_camera = FpvCamera::new(
        Some("main camera"),
        true,
        None,
        Some(Vector3(-15.0, 0.0, -9.0)), // zoom out tmr
        Some(Quaternion::camera_look_at_xy(Vector3(-15.0, 0.0, -9.0), Vector3(-1.0, 0.0, -1.0))),
        None,
        None,
        120,
        0.1,
        100.0,
        dimensions[0] as f32 / dimensions[1] as f32, 
        None,
        engine.globals.clone()
    );
    engine.scene.write().unwrap().set_main_camera(main_camera.unwrap());

    // Logo
    let logo = Obj::from_obj(
        Some("cube"), 
        "assets/standard-assets/models/logo/logo.obj",
        None,
        None,
        None,
        None,
        true,
        engine.globals.clone(),
        None
    );
    engine.scene.write().unwrap().add_child(logo.unwrap());
    
    // Plane
    let plane = Obj::from_obj(
        Some("cube"), 
        "assets/standard-assets/models/shapes/plane.obj",
        None,
        Some(Vector3(0.0, -5.0, 0.0)),
        None,
        None,
        true,
        engine.globals.clone(),
        None
    );
    engine.scene.write().unwrap().add_child(plane.unwrap());

    // Run
    engine.run();
}