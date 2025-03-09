use bevy_ecs::entity::Entity;
use bevy_ecs::query::Without;
use bevy_ecs::schedule::Schedules;
use bevy_ecs::system::{Query, Res};
use engine::ecs::components::{Camera, Inputs, Rotation, Scale, SpriteRenderer2D, Transform};
use engine::ecs::config::{EcsFixedUpdateSchedule, EcsLateUpdateSchedule, EcsUpdateSchedule};
use engine::ecs::time::Time;
use engine::lib::runtime::App;

use engine::rendering::shaders::Material;
use engine::utils::app_settings::{ApplicationSettings, WindowMode, WindowSettings};
use glm::vec4;

use crate::engine::ecs::components::Position;

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

        if keyboard.is_key_released(glfw::Key::A) {
            println!("[Update Camera System] Updating camera viewport");
            camera.viewport = (0.5f32, 0f32, 1f32, 1f32);
        }

        if keyboard.is_key_released(glfw::Key::Q) {
            println!("[Update Camera System] Reset camera viewport");
            camera.viewport = (0f32, 0f32, 1f32, 1f32);
        }
    }
}

pub fn update_system(time: Res<Time>, mut query: Query<(Entity, &mut Transform), Without<Camera>>) {
    for (_entity, mut transform) in query.iter_mut() {
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

        checker.accumulated_time += _time.delta_time;
        checker.color_timer += _time.delta_time;

        if checker.accumulated_time >= 2f32 {
            let number = (_time.frames % 9u64).clamp(1, 9);
            sprite.texture = Option::from(format!("Dark/texture_0{}.png", number));
            checker.accumulated_time = 0f32;
        }

        if checker.color_timer >= 2f32 {
            let color = checker
                .flip_color
                .then(|| color_other)
                .unwrap_or(color_white);

            sprite.material.get_or_insert_with(Material::new).color = color;

            checker.flip_color = !checker.flip_color;
            checker.color_timer = 0f32;
        }
    }
}

fn main() {
    let app_settings = ApplicationSettings {
        window: WindowSettings {
            width: 800,
            height: 600,
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

        let _entity = world.spawn((
            Transform {
                position: Position::default(),
                rotation: Rotation::default(),
                scale: Scale { x: 0.5, y: 0.5, z: 0.5 } ,
            },
            SpriteRenderer2D::from(String::from("Red/texture_08.png"), false),
            ChangeChecker {
                accumulated_time: 0f32,
                color_timer: 0f32,
                flip_color: false,
            },
        ));
    }

    match app.run() {
        Ok(_) => println!("Game closes gracefully!"),
        Err(reason) => println!("Game closes with error: {}", reason),
    };
}
