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
    },
};

fn main() {
    // Scene
    let scene = Scene::new(None);

    // Engine
    let mut engine = FeoEngine::init(scene, None);
    
    // create ambient light
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

    // create a directional light
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
        Some(Vector3(0.0, 3.0, -10.0)),
        Some(Quaternion::camera_look_at_xy(Vector3(0.0, 3.0, -10.0), Vector3(0.0, 0.0, 0.0))),
        None,
        None,
        120,
        0.1,
        100.0, 
        dimensions[0] as f32 / dimensions[1] as f32, 
        Some(Script::new_boxed(Box::pin(scr::camera_swap::start), Box::pin(scr::camera_swap::frame), None)),
        engine.globals.clone()
    ).unwrap();
    engine.scene.write().unwrap().add_child(main_camera.clone());
    engine.scene.write().unwrap().set_main_camera(main_camera);
    
    // Axes visual
    let xyz = Obj::from_obj(
        Some("XYZ"),
        "assets/standard-assets/models/xyz.obj",
        None,
        None,
        None,
        None,
        true,
        engine.globals.clone(),
        Some(Script::new_boxed(Box::pin(scr::delete::start), Box::pin(scr::delete::frame), None))
    );
    engine.scene.write().unwrap().add_child(xyz.unwrap());


    // object
    let obj = Obj::from_obj( // err
        Some("roundabout"), 
        "assets/standard-assets/models/shapes/cube.obj",
        None,
        Some(Vector3::new( 2.0, 0.0, 2.0)),
        None, //Some(Quaternion::new_axis_angle(axes::NORMAL_X_AXIS, std::f32::consts::PI/2.0)),
        Some(Vector3(1.025, 1.025, 1.025)),
        true,
        engine.globals.clone(),
        Some(Script::new_boxed(Box::pin(scr::rotate_swap::start), Box::pin(scr::rotate_swap::frame), None))
    );
    engine.scene.write().unwrap().add_child(obj.unwrap());

    // run
    engine.run();
}