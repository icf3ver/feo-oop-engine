use std::sync::{Arc, RwLock};

use feo_oop_engine::scene::game_object::{GameObject, group::Group};
use scr::TargetGlobal;

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
        scripting::{
            Script,
            Scriptable,
            globals::Global
        },
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

    // Ambient Light
    let ambient_light = AmbientLight::new(
        Some("ambient light"), 
        None, 
        1.0f32, 
        RGB::new(0.5, 0.5, 0.5), 
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
        RGB::new(0.8, 0.8, 0.8), 
        None, 
        Some(Quaternion::new(0.5, 1.0, 0.5, std::f32::consts::PI)), 
        None, 
        None, 
        engine.globals.clone()
    );
    engine.scene.write().unwrap().add_child(directional_light);
    
    // Axes visual
    let xyz = Obj::from_obj(
        Some( "XYZ"),
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

    let player_container = Group::new(
        Some("Player"),
        None,
        Some(Vector3(6.0, 0.0, -5.0)),
        None,
        None,
        true,
        engine.globals.clone(),
        Some(
            Script::new_boxed(
                Box::pin(scr::player::start), 
                Box::pin(scr::player::frame), 
                Some(Box::pin(scr::player::event_handler))
            )
        )
    );
    engine.scene.write().unwrap().add_child(player_container.clone());

    let player_model = Obj::from_obj(
        Some("XYZ"),
        "assets/standard-assets/models/shapes/cube.obj",
        Some(player_container.clone()),
        None,
        None,
        None,
        true,
        engine.globals.clone(),
        None
    ).unwrap();
    player_container.write().unwrap().add_child(player_model);

    let dimensions: [u32; 2] = engine.globals.surface.window().inner_size().into();
    let main_camera = FpvCamera::new(
        Some("main camera"),
        true,
        Some(player_container.clone()),
        Some(Vector3(0.0, 3.0, 10.0)),
        None,
        None,
        None,
        120,
        0.1,
        100.0, 
        dimensions[0] as f32 / dimensions[1] as f32,
        None,
        engine.globals.clone()
    ).unwrap();
    engine.scene.write().unwrap().set_main_camera(main_camera.clone());
    player_container.write().unwrap().add_child(main_camera); // fix

    let pew_container = Group::new(
        Some("Pew Container"),
        None,
        None,
        None,
        None,
        true,
        engine.globals.clone(),
        Some(
            Script::new_boxed(
                Box::pin(scr::pew_spawner::start), 
                Box::pin(scr::pew_spawner::frame), 
                Some(Box::pin(scr::pew_spawner::event_handler))
            )
        )
    );
    engine.scene.write().unwrap().add_child(pew_container);

    // note both spawner and layer of existence
    let enemy_spawner = Group::new(
        Some("Pew Container"),
        None,
        None,
        None,
        None,
        true,
        engine.globals.clone(),
        Some(
            Script::new_boxed(
                Box::pin(scr::enemy_spawner::start), 
                Box::pin(scr::enemy_spawner::frame), 
                None
            )
        )
    );
    enemy_spawner.write().unwrap().set_globals(Box::new(TargetGlobal{
        target: player_container as Arc<RwLock<dyn GameObject>>
    }) as Box<dyn Global>).unwrap();
    engine.scene.write().unwrap().add_child(enemy_spawner);

    // run
    engine.run();
}