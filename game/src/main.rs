use bevy_ecs::entity::Entity;
use bevy_ecs::query::Without;
use bevy_ecs::schedule::Schedules;
use bevy_ecs::system::{Query, Res};
use engine::ecs::components::{Camera, Inputs, Rotation, Scale, SpriteRenderer2D, Transform};
use engine::ecs::config::{EcsFixedUpdateSchedule, EcsLateUpdateSchedule, EcsUpdateSchedule};
use engine::ecs::time::Time;
use engine::lib::runtime::App;

use crate::engine::ecs::components::Position;
use crate::engine::rendering::components::ARGB8Color;
use engine::utils::app_settings::{ApplicationSettings, WindowMode, WindowSettings};
use glm::{abs, vec4};
use rand::random;

pub mod engine;

#[derive(bevy_ecs::component::Component)]
pub struct ChangeChecker {
    pub accumulated_time: f32,
    pub color_timer: f32,
    pub flip_color: bool,
}

pub fn update_camera(inputs: Res<Inputs>, mut query: Query<(Entity, &mut Camera)>) {
    let mut camera_count: i32 = 0;

    for (_entity, mut camera) in query.iter_mut() {
        camera_count += 1;

        if camera_count > 1 {
            break;
        }

        let keyboard = inputs.keyboard.lock().unwrap();

        if keyboard.is_key_released(glfw::Key::Q) {
            println!("[Update Camera System] Updating camera viewport");
            camera.viewport = (0.5f32, 0f32, 1f32, 1f32);
        }

        if keyboard.is_key_released(glfw::Key::E) {
            println!("[Update Camera System] Reset camera viewport");
            camera.viewport = (0f32, 0f32, 1f32, 1f32);
        }

        if keyboard.is_key_released(glfw::Key::I) {
            camera.ppu = 380u32;
        }
        if keyboard.is_key_released(glfw::Key::K) {
            camera.ppu = 100u32;
        }
    }
}

pub fn move_camera_2d(
    time: Res<Time>,
    inputs: Res<Inputs>,
    mut query: Query<(&mut Transform, &mut Camera)>,
) {
    for (mut transform, _camera) in query.iter_mut() {
        let keyboard = inputs.keyboard.lock().unwrap();

        const SPEED: f32 = 18f32;
        if keyboard.is_key_pressed(glfw::Key::W) {
            transform.position.y += SPEED * time.delta_time;
        }
        if keyboard.is_key_pressed(glfw::Key::S) {
            transform.position.y -= SPEED * time.delta_time;
        }
        if keyboard.is_key_pressed(glfw::Key::D) {
            transform.position.x += SPEED * time.delta_time;
        }
        if keyboard.is_key_pressed(glfw::Key::A) {
            transform.position.x -= SPEED * time.delta_time;
        }
        if keyboard.is_key_pressed(glfw::Key::C) {
            transform.position.z -= SPEED * time.delta_time;
        }
        if keyboard.is_key_pressed(glfw::Key::V) {
            transform.position.z += SPEED * time.delta_time;
        }

        if keyboard.is_key_released(glfw::Key::C) || keyboard.is_key_released(glfw::Key::V) {
            println!("[Update Camera System] Moving Camera ends {:?}", transform);
        }
    }
}

pub fn update_system(time: Res<Time>, mut query: Query<(Entity, &mut Transform), Without<Camera>>) {
    for (_entity, mut transform) in query.iter_mut() {
        continue;
        let x: f32 = transform.position.x + (time.time.cos() as f32 * time.delta_time);

        transform.position = Position {
            x,
            y: transform.position.y,
            z: transform.position.z,
        };
    }
}

pub fn fixed_update_system(_time: Res<Time>) {
    // println!("Fixed update: {}", &time.fixed_delta_time);
}

pub fn late_update_system(
    _time: Res<Time>,
    mut query: Query<(&mut SpriteRenderer2D, &mut ChangeChecker)>,
) {
    for (mut sprite, mut checker) in query.iter_mut() {
        let color_white = vec4(1f32, 1f32, 1f32, 1f32);
        let color_other = vec4(0.1f32, 1f32, 0.8f32, 1f32);
        let colors = ["Dark", "Green", "Orange", "Purple", "Red"];

        checker.accumulated_time += _time.delta_time;
        checker.color_timer += _time.delta_time;

        if checker.accumulated_time >= 2f32 {
            let rng_idx = abs(random::<i32>() % 9);
            let rng_folder = abs(random::<i32>() % 5);

            let number = (rng_idx + 1).clamp(1, 9);
            let folder = colors[rng_folder as usize];
            sprite.texture = Option::from(format!("{}/texture_0{}.png", folder, number));
            checker.accumulated_time = 0f32;
        }

        // if checker.color_timer >= 2f32 {
        //     let color = checker
        //         .flip_color
        //         .then(|| color_other)
        //         .unwrap_or(color_white);
        //
        //     sprite.material.get_or_insert_with(Material::new).color = color;
        //
        //     checker.flip_color = !checker.flip_color;
        //     checker.color_timer = 0f32;
        // }
    }
}

fn main() {
    let app_settings = ApplicationSettings {
        window: WindowSettings {
            width: 1920,
            height: 1080,
            mode: WindowMode::Windowed,
        },
        app_name: String::from("Afraid of the Dark"),
        target_frame_rate: 120f32,
    };

    let mut app = App::with_appsettings(app_settings);

    app.warm();

    if let Some(world) = app.world.as_mut() {
        let mut schedules = world.resource_mut::<Schedules>();
        schedules
            .get_mut(EcsUpdateSchedule)
            .unwrap()
            .add_systems(update_system);
        schedules
            .get_mut(EcsFixedUpdateSchedule)
            .unwrap()
            .add_systems(fixed_update_system);
        schedules
            .get_mut(EcsLateUpdateSchedule)
            .unwrap()
            .add_systems(late_update_system);
        schedules
            .get_mut(EcsUpdateSchedule)
            .unwrap()
            .add_systems(update_camera);
        schedules
            .get_mut(EcsUpdateSchedule)
            .unwrap()
            .add_systems(move_camera_2d);

        let pos: f32 = 1.2f32;
        for i in 0..100 {
            let _entity = world.spawn((
                Transform {
                    position: Position {
                        x: (i as f32) * pos,
                        y: 0.0,
                        z: 0.0,
                    },
                    rotation: Rotation {
                        x: 0.0,
                        y: 0.0,
                        z: 0.0,
                    },
                    scale: Scale {
                        x: 1.0,
                        y: 1.0,
                        z: 1.0,
                    },
                },
                SpriteRenderer2D::from(String::from("Red/texture_08.png"), false),
                ChangeChecker {
                    accumulated_time: 0f32,
                    color_timer: 0f32,
                    flip_color: false,
                },
            ));
        }

        let cam = world.get_main_camera();
        let mut camera_entity = world.entity_mut(cam);
        let mut camera = camera_entity.get_mut::<Camera>().unwrap();

        camera.background_color = Option::from(ARGB8Color {
            r: 100,
            g: 149,
            b: 150,
            a: 255,
        });
    }

    match app.run() {
        Ok(_) => println!("Game closes gracefully!"),
        Err(reason) => println!("Game closes with error: {}", reason),
    };
}
