use bevy_ecs::entity::Entity;
use bevy_ecs::schedule::Schedules;
use bevy_ecs::system::{Query, Res};
use engine::ecs::components::{Rotation, Scale, SpriteRenderer2D, Transform};
use engine::ecs::config::{EcsFixedUpdateSchedule, EcsLateUpdateSchedule, EcsUpdateSchedule};
use engine::ecs::time::Time;
use engine::lib::runtime::App;

use engine::utils::app_settings::{ApplicationSettings, WindowMode, WindowSettings};

use crate::engine::ecs::components::Position;

pub mod engine;

#[derive(bevy_ecs::component::Component)]
pub struct ChangeChecker { 
    pub accumulated_time: u64,
}

pub fn update_system(time: Res<Time>, mut query: Query<(Entity, &mut Transform)>) {
    for (_entity, mut transform) in query.iter_mut() {
        let x: &f32 = &transform.position.x;
        transform.position = Position {
            x: x + 12f32 * &time.delta_time,
            y: transform.position.y,
            z: transform.position.z,
        };
    }
}

pub fn fixed_update_system(_time: Res<Time>) {
    // println!("Fixed update: {}", &time.fixed_delta_time);
}

pub fn late_update_system(_time: Res<Time>, mut query: Query<(&mut SpriteRenderer2D, &mut ChangeChecker)>) {
    //println!("Late update: {}", &time.delta_time);

    for (mut sprite, mut checker) in query.iter_mut() { 
        if checker.accumulated_time > 500 { 
            sprite.texture = Option::from(String::from("Dark/texture_05.png"));
            checker.accumulated_time = 0;
        }
        checker.accumulated_time += 1;
    }
}

fn main() {
    let app_settings = ApplicationSettings {
        window: WindowSettings {
            width: 800,
            height: 600,
            mode: WindowMode::Windowed,
        },
        app_name: String::from("Afraid of Dark"),
        target_frame_rate: 30f32,
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

        let _entity = world.spawn((
            Transform {
                position: Position::default(),
                rotation: Rotation::default(),
                scale: Scale::default(),
            },
            SpriteRenderer2D {
                texture: Option::from(String::from("Dark/texture_01.png")),
                material: None,
            },
            ChangeChecker { accumulated_time: 0 }
        ));
    }

    match app.run() {
        Ok(_) => println!("Game closes gracefully!"),
        Err(reason) => println!("Game closes with error: {}", reason)
    };
}
