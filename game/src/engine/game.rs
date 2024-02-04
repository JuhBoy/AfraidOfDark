pub mod runtime {
    use crate::engine::{
        ecs::{config::EcsLateUpdateSchedule, config::EcsUpdateSchedule, time::Time},
        logging::{consts, logs::Logger, logs_traits::LoggerBase},
        rendering::renderer::Renderer,
        utils::app_settings::ApplicationSettings,
    };
    use bevy_ecs::prelude::*;
    use std::rc::Rc;

    pub struct App {
        _name: String,
        logs: Rc<dyn LoggerBase>,
        app_settings: ApplicationSettings,

        // Base systems
        renderer: Option<Renderer>,

        // Exposed system
        pub ecs_world: Option<Box<World>>,
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
                ecs_world: Option::None,
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
                ecs_world: Option::None,
            }
        }

        fn warm(&mut self) {
            let s: &ApplicationSettings = &self.app_settings;
            let renderer: Renderer = Renderer::init_with_glfw(&s.window, self.logs.clone());
            self.renderer = Option::from(renderer);

            // ECS -- Schedulers & world
            self.ecs_world = Option::from(Box::new(World::new()));
            let world = self.ecs_world.as_mut().unwrap();

            let update_schedule = Schedule::new(EcsUpdateSchedule);
            let late_update_schedule = Schedule::new(EcsLateUpdateSchedule);

            world.add_schedule(update_schedule);
            world.add_schedule(late_update_schedule);

            // Resources
            world.insert_resource::<Time>(Time {
                frames: 0f64,
                time: 0f64,
                delta_time: 0.02f32,
                fixed_delta_time: 0.02f32,
            });
        }

        pub fn run(&mut self) {
            self.warm();

            let renderer: &mut Renderer;
            let world: &mut World;

            if let Some(rdr) = self.renderer.as_mut() { 
                renderer = rdr;
            } else { 
                self.logs.error(consts::ENGINE_CATEGORY,"Renderer engine system could not be loaded.",);
                panic!();
            }

            if let Some(ecs) = self.ecs_world.as_mut() { 
                world = ecs;
            } else {
                self.logs.error(consts::ENGINE_CATEGORY, "ECS system not initialized");
                panic!();
            }

            // Game loop [WIP]
            while !renderer.window.should_close() {
                renderer.poll_events();

                world.run_schedule(EcsUpdateSchedule);
                world.run_schedule(EcsLateUpdateSchedule);
            }
        }
    }
}
