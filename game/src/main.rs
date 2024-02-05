use bevy_ecs::schedule::Schedules;
use bevy_ecs::system::Res;
use engine::ecs::config::{EcsFixedUpdateSchedule, EcsLateUpdateSchedule, EcsUpdateSchedule};
use engine::ecs::time::Time;
use engine::game::runtime::App;
use engine::utils::app_settings::{ApplicationSettings, WindowMode, WindowSettings};

pub mod engine;

pub fn update_system(time: Res<Time>) {
    println!("Update: {}", &time.delta_time);
}

pub fn fixed_update_system(time: Res<Time>) {
    println!("Fixed update: {}", &time.fixed_delta_time);
}

pub fn late_update_system(time: Res<Time>) {
    println!("Late update: {}", &time.delta_time);
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

    if let Some(world) = app.ecs_world.as_mut() {
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
    }

    app.run();
}
