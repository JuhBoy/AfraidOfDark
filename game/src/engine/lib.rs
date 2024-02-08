pub mod runtime {
    use bevy_ecs::schedule::Schedule;

    use crate::engine::{
        ecs::{
            config::{EcsFixedUpdateSchedule, EcsLateUpdateSchedule, EcsUpdateSchedule},
            systems::{add_sprite_2d_system, changed_sprite_2d_system},
            time::{RenderingResourcesContainer, Time},
        },
        logging::{consts, logs::Logger, logs_traits::LoggerBase},
        rendering::renderer::Renderer,
        utils::{app_settings::ApplicationSettings, world::World},
    };
    use std::{rc::Rc, time::Duration};

    pub struct App {
        _name: String,
        logs: Rc<dyn LoggerBase>,
        app_settings: ApplicationSettings,

        // Base systems
        renderer: Option<Renderer>,

        // Exposed system
        pub world: Option<Box<World>>,
    }

    impl App {
        pub fn new(name: &str) -> Self {
            Self {
                _name: String::from(name),
                logs: Rc::new(Logger {
                    log_type: String::from(name),
                }),
                app_settings: ApplicationSettings::default(),
                renderer: Option::None,
                world: Option::None,
            }
        }

        pub fn with_appsettings(settings: ApplicationSettings) -> Self {
            Self {
                _name: settings.app_name.clone(),
                logs: Rc::new(Logger {
                    log_type: String::from(&settings.app_name),
                }),
                app_settings: settings,
                renderer: Option::None,
                world: Option::None,
            }
        }

        pub fn warm(&mut self) -> &mut Self {
            let s: &ApplicationSettings = &self.app_settings;

            // Rendering
            let mut renderer: Renderer = Renderer::init_with_glfw(&s.window, self.logs.clone());
            renderer.warm();
            self.renderer = Option::from(renderer);

            // ECS -- Schedulers & world
            self.world = Option::from(Box::new(World::new()));
            let world = self.world.as_mut().unwrap();

            let update_schedule = Schedule::new(EcsUpdateSchedule);
            let fixed_update_schedule = Schedule::new(EcsFixedUpdateSchedule);
            let mut late_update_schedule = Schedule::new(EcsLateUpdateSchedule);

            late_update_schedule.add_systems(changed_sprite_2d_system);
            late_update_schedule.add_systems(add_sprite_2d_system);

            world.add_schedule(update_schedule);
            world.add_schedule(fixed_update_schedule);
            world.add_schedule(late_update_schedule);

            // Resources
            world.insert_resource::<Time>(Time {
                frames: 0f64,
                time: 0f64,
                delta_time: 0.02f32,
                fixed_delta_time: 0.02f32,
            });

            world.insert_resource::<RenderingResourcesContainer>(RenderingResourcesContainer {
                frame: 0f64,
                new_2d_render: Vec::new(),
                updated_2d_render: Vec::new(),
                deleted_2d_render: Vec::new(),
            });

            self
        }

        pub fn run(&mut self) -> Result<(), &'static str> {
            let renderer: &mut Renderer;
            let world: &mut World;

            if let Some(rdr) = self.renderer.as_mut() {
                renderer = rdr;
            } else {
                self.logs.error(
                    consts::ENGINE_CATEGORY,
                    "Renderer engine system could not be loaded.",
                );
                return Err("Renderer engine system could not be loaded.");
            }

            if let Some(ecs) = self.world.as_mut() {
                world = ecs.as_mut()
            } else {
                self.logs
                    .error(consts::ENGINE_CATEGORY, "ECS system not initialized");
                return Err("ECS system not initialized");
            }

            let framerate = self.app_settings.target_frame_rate;
            let mut time_point = std::time::Instant::now();
            let mut accumulated_time = 0.0f32;
            let fixed_delta_time = world.resource::<Time>().fixed_delta_time;

            // Game loop [WIP]
            while !renderer.window.should_close() {
                renderer.poll_events();

                let dt = std::time::Instant::elapsed(&time_point);
                time_point = std::time::Instant::now();

                let delta = dt.as_secs_f32();
                let fixed_delta = fixed_delta_time;

                let mut time_world = world.resource_mut::<Time>();
                time_world.delta_time = delta;
                accumulated_time += delta;

                // Update game logic once
                world.run_schedule(EcsUpdateSchedule);

                // Update physic&fixed at same time step.
                while accumulated_time >= fixed_delta {
                    accumulated_time -= fixed_delta;
                    world.run_schedule(EcsFixedUpdateSchedule);
                }

                // Late update for UI.
                world.run_schedule(EcsLateUpdateSchedule);

                // Bakes rendering commands
                world.inject_new_rendering_entities(renderer);
                world.flush_rendering_command_handles(renderer);

                // Render and forward overflow time
                renderer.render(accumulated_time);

                let sleep_time: f32 = f32::max((1.0f32 / framerate) - delta, 0f32);
                std::thread::sleep(Duration::from_secs_f32(sleep_time));
            }

            Ok(())
        }

        pub fn clear_new_rendering_entity(world: &mut World) {
            let mut container = world.resource_mut::<RenderingResourcesContainer>();

            if container.new_2d_render.is_empty() {
                return;
            }
            container.new_2d_render.clear();
        }
    }
}
