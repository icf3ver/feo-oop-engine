use feo_oop_engine::scene::game_object::light::point_light::PointLight;

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
        linear_algebra::vector3::Vector3,
    }
};

fn main() {
    // Scene
    let scene = Scene::new(None);

    // Engine
    let mut engine = FeoEngine::init(scene, Some(0));

    // Camera
    let dimensions: [u32; 2] = engine.globals.surface.window().inner_size().into();
    let main_camera = FpvCamera::new(
        Some("main camera"),
        true,
        None,
        Some(Vector3(5.0, 3.0, 10.0)),
        None,
        None,
        None,
        120,
        0.1,
        100.0, 
        dimensions[0] as f32 / dimensions[1] as f32,
        Some(Script::new_boxed(Box::pin(scr::camera::start), Box::pin(scr::camera::frame), Some(Box::pin(scr::camera::event_handler)))),
        engine.globals.clone()
    );
    engine.scene.write().unwrap().set_main_camera(main_camera.clone().unwrap());
    engine.scene.write().unwrap().add_child(main_camera.clone().unwrap());

    // Ambient Light
    let ambient_light = AmbientLight::new(
        Some("ambient light"), 
        None, 
        1.0f32, 
        RGB::new(0.05, 0.05, 0.05), 
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
        RGB::new(0.05, 0.05, 0.05), 
        None, 
        Some(Quaternion::new(0.5, 1.0, 0.5, std::f32::consts::PI)), 
        None, 
        None, 
        engine.globals.clone()
    );
    engine.scene.write().unwrap().add_child(directional_light);

    // Point Lights
    let point_light = PointLight::new(
        Some("point light"), 
        Some(main_camera.clone().unwrap()), 
        1.0_f32,
        RGB::new(0.5, 0.5, 0.5), 
        Some(Vector3(1.0, 1.0, 1.0)), 
        None,
        None,
        None, 
        engine.globals.clone()
    );
    main_camera.unwrap().write().unwrap().add_child(point_light);
    let point_light2 = PointLight::new(
        Some("point light"), 
        None, 
        1.0_f32,
        RGB::new(0.7, 0.0, 0.0), 
        Some(Vector3(5.0, 5.0, 5.0)), 
        None,
        None,
        None, 
        engine.globals.clone()
    );
    engine.scene.write().unwrap().add_child(point_light2);

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

    // Cube
    let obj = Obj::from_obj(
        Some("cube"), 
        "assets/standard-assets/models/shapes/cube.obj",
        None,
        Some(Vector3(5.0, 5.0, 5.9)),
        None,
        None,
        true,
        engine.globals.clone(),
        None
    );
    engine.scene.write().unwrap().add_child(obj.unwrap());

    // Cube
    let obj = Obj::from_obj(
        Some("cube"), 
        "assets/standard-assets/models/shapes/cube.obj",
        None,
        Some(Vector3(5.0, 5.0, 3.0)),
        None,
        None,
        true,
        engine.globals.clone(),
        None
    );
    engine.scene.write().unwrap().add_child(obj.unwrap());

    // Plane
    let obj = Obj::from_obj(
        Some("cube"), 
        "assets/standard-assets/models/shapes/cube.obj",
        None,
        Some(Vector3(5.0, 4.0, 5.0)),
        None,
        Some(Vector3(20.0, 1.0, 20.0)),
        true,
        engine.globals.clone(),
        None
    );
    engine.scene.write().unwrap().add_child(obj.unwrap());


    // run
    engine.run();
}