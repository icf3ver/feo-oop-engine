mod scr;

#[macro_use] extern crate macro_rules_attribute;
#[macro_use] extern crate global_macro_derive;
#[macro_use] extern crate feo_oop_engine;

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
        scripting::Script,
        components::RGB
    },
    feo_math::{
        rotation::quaternion::Quaternion,
        rotation::Rotation,
        linear_algebra::vector3::Vector3
    }
};


fn main() {
    // Scene
    let scene = Scene::new(None);

    // Engine
    let mut engine = FeoEngine::init(scene, None);

    // Ambient Light
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

    // Directional Light
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

    // Camera
    let dimensions: [u32; 2] = engine.globals.surface.window().inner_size().into();
    let main_camera = FpvCamera::new(
        Some("main camera"),
        false,
        None,
        Some(Vector3(5.0, 3.0, -10.0)),
        Some(Quaternion::camera_look_at_xy(Vector3(5.0, 3.0, -10.0), Vector3(0.0, 0.0, 0.0))),
        None,
        None,
        120,
        0.1,
        100.0, 
        dimensions[0] as f32 / dimensions[1] as f32,
        Some(Script::new_boxed(Box::pin(scr::camera::start), Box::pin(scr::camera::frame), None)),
        engine.globals.clone()
    );
    engine.scene.write().unwrap().set_main_camera(main_camera.unwrap());
    
    // Axes visual
    let xyz = Obj::from_obj(
        Some("XYZ"),
        "assets/standard-assets/models/debugging/xyz.obj",
        None,
        None,
        None,
        None,
        true,
        engine.globals.clone(),
        None
    );
    engine.scene.write().unwrap().add_child(xyz.unwrap());

    // Cool spaceship models that I did not make.
    let sun_glyder = Obj::from_obj(
        Some("XYZ"),
        "assets/example-assets/models/Sun Glyder/Sun Glyder.obj",
        None,
        None,
        None,
        Some(Vector3(0.1, 0.1, 0.1)),
        true,
        engine.globals.clone(),
        None
    );
    engine.scene.write().unwrap().add_child(sun_glyder.unwrap());

    let aircraft = Obj::from_obj(
        Some("XYZ"),
        "assets/example-assets/models/E 45 Aircraft/E 45 Aircraft_obj.obj",
        None,
        Some(Vector3(10.0, 10.0, 10.0)),
        None,
        None,
        true,
        engine.globals.clone(),
        None
    );
    engine.scene.write().unwrap().add_child(aircraft.clone().unwrap());

    // run
    engine.run();
}