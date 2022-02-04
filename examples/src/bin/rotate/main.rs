use feo_oop_engine::{scene::game_object::light::{ambient_light::AmbientLight, directional_light::DirectionalLight}, components::RGB};

mod scr;

#[macro_use] extern crate macro_rules_attribute;
#[macro_use] extern crate feo_oop_engine_proc_macros;
#[macro_use] extern crate feo_oop_engine;

use {
    feo_oop_engine::{
        FeoEngine, 
        scene::{
            game_object::{
                camera::fpv_camera::FpvCamera, 
                obj::Obj
            },
            Scene, 
        },
        registration::relation::Parent,
        scripting::Script,
    },
    feo_math::{
        rotation::quaternion::Quaternion,
        linear_algebra::vector3::Vector3,
        axes
    }
};

fn main() {
    // Scene 
    let scene = Scene::new(None);

    // Engine
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
        RGB::new(0.9, 0.9, 0.9), 
        Some(Vector3(5.0, 3.0, 10.0)),
        Some(Quaternion::camera_look_at(Vector3(5.0, 3.0, -10.0), Vector3(0.0, 0.0, 0.0))),
        None, 
        Some(Script::new_boxed(Box::pin(scr::light::start), Box::pin(scr::light::frame), None)),
        engine.globals.clone()
    );
    engine.scene.write().unwrap().add_child(directional_light);
    
    // Camera
    let dimensions: [u32; 2] = engine.globals.surface.window().inner_size().into();
    let main_camera = FpvCamera::new(
        Some("main camera"),
        true,
        None,
        Some(Vector3(5.0, 3.0, -10.0)),
        Some(Quaternion::camera_look_at(Vector3(5.0, 3.0, -10.0), Vector3(0.0, 0.0, 0.0))),
        None,
        None,
        120,
        0.1,
        100.0, 
        dimensions[0] as f32 / dimensions[1] as f32,
        None,//Some(Script::new_boxed(Box::pin(scr::camera::start), Box::pin(scr::camera::frame), None)),
        engine.globals.clone()
    );
    engine.scene.write().unwrap().set_main_camera(main_camera.unwrap());
    
    // Axes visual
    let xyz = Obj::from_obj(
        Some("Axes"),
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


    // object
    let obj = Obj::from_obj(
        Some("roundabout"), 
        "assets/standard-assets/models/shapes/cube.obj",
        None,
        Some(Vector3::new( 2.0, 0.0, 2.0)),
        Some(Quaternion::new_axis_angle(axes::NORMAL_Y_AXIS, std::f32::consts::PI/2.0)),
        None,
        true,
        engine.globals.clone(),
        Some(Script::new_boxed( Box::pin(scr::rotate::start), Box::pin(scr::rotate::frame), None))
    ).unwrap();
    engine.scene.write().unwrap().add_child(obj.clone());


    // object
    let obj2 = Obj::from_obj(
        Some("two"), 
        "assets/standard-assets/models/shapes/cube.obj",
        Some(obj.clone()),
        Some(Vector3::new( 2.0, 0.0, 2.0)),
        None,
        None,
        true,
        engine.globals.clone(),
        None
    );
    obj.write().unwrap().add_child(obj2.unwrap());

    // run
    engine.run();
}