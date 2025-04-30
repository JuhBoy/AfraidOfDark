pub mod runtime {
    use bevy_ecs::schedule::Schedule;

    use crate::engine::ecs::resources::CameraCullingState;
    use crate::engine::{
        ecs::{
            components::{CameraBinding, Inputs},
            config::{EcsFixedUpdateSchedule, EcsLateUpdateSchedule, EcsUpdateSchedule},
            resources::{RenderingFrameData, Time},
            systems::{
                add_camera_2d_system, add_sprite_2d_system, changed_sprite_2d_system,
                update_camera_settings_system, update_camera_transform_system,
            },
        },
        logging::{logs::Logger, logs_traits::LoggerBase},
        rendering::renderer::Renderer,
        utils::{app_settings::ApplicationSettings, rendering_bridge::RenderingBridge},
    };
    use bevy_ecs::world::World;
    use std::cell::{RefCell, RefMut};
    use std::{rc::Rc, time::Duration};

    pub struct App {
        _name: String,
        logs: Rc<dyn LoggerBase>,
        app_settings: ApplicationSettings,

        // Internal systems
        renderer: Option<Renderer>,

        // Exposed system
        pub rendering_bridge: Option<RenderingBridge>,
        pub world: Option<Rc<RefCell<World>>>,
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
                rendering_bridge: Option::None,
                world: Option::None,
            }
        }

        pub fn with_settings(settings: ApplicationSettings) -> Self {
            Self {
                _name: settings.app_name.clone(),
                logs: Rc::new(Logger {
                    log_type: String::from(&settings.app_name),
                }),
                app_settings: settings,
                renderer: Option::None,
                rendering_bridge: None,
                world: Option::None,
            }
        }

        pub fn warm(&mut self) -> &mut Self {
            // Rendering
            let mut renderer: Renderer =
                Renderer::init_with_glfw(&self.app_settings.window, self.logs.clone());
            renderer.warm();
            self.renderer = Option::from(renderer);

            // ECS -- Bridge & world Init
            self.world = Option::from(Rc::new(RefCell::new(World::new())));
            self.rendering_bridge =
                Option::from(RenderingBridge::new(self.world.as_ref().unwrap().clone()));

            // Set up the ECS world & schedulers
            {
                let mut world = self.world.as_mut().unwrap().borrow_mut();

                let update_schedule = Schedule::new(EcsUpdateSchedule);
                let fixed_update_schedule = Schedule::new(EcsFixedUpdateSchedule);
                let mut late_update_schedule = Schedule::new(EcsLateUpdateSchedule);

                late_update_schedule.add_systems(changed_sprite_2d_system);
                late_update_schedule.add_systems(add_sprite_2d_system);
                late_update_schedule.add_systems(add_camera_2d_system);
                late_update_schedule.add_systems(update_camera_settings_system);
                late_update_schedule.add_systems(update_camera_transform_system);

                world.add_schedule(update_schedule);
                world.add_schedule(fixed_update_schedule);
                world.add_schedule(late_update_schedule);

                // Resources
                world.insert_resource::<Time>(Time {
                    frames: 0u64,
                    time: 0f64,
                    delta_time: 0.02f32,
                    fixed_delta_time: 0.02f32,
                });

                world.insert_resource::<Inputs>(Inputs {
                    keyboard: self.renderer.as_ref().unwrap().get_keyboard_inputs(),
                });

                let main_entity_camera = { RenderingBridge::build_camera(&mut *world) };
                world.insert_resource::<CameraBinding>(CameraBinding {
                    cameras: vec![(main_entity_camera, 0u32)],
                });
                
                let renderer = self.renderer.as_ref().unwrap();
                let camera_viewport = renderer.get_world_camera_viewport();

                world.insert_resource::<CameraCullingState>(CameraCullingState {
                    last_check_frame: 0.0,
                    camera_entity: Option::from(main_entity_camera),
                    camera_world_viewport: camera_viewport,
                    force_full_pass: false,
                    entities: Vec::with_capacity(1024),
                });

                world.insert_resource::<RenderingFrameData>(RenderingFrameData {
                    frame: 0f64,
                    new_2d_render: Vec::new(),
                    updated_2d_render: Vec::with_capacity(200),
                    deleted_2d_render: Vec::new(),
                    updated_camera_settings: Vec::new(),
                    updated_camera_transform: Vec::new(),
                });
            }

            self
        }

        pub fn run(&mut self) -> Result<(), &'static str> {
            assert!(
                self.renderer.is_some(),
                "Rendering has not been initialized"
            );
            assert!(self.world.is_some(), "ECS World has not been initialized");

            let renderer: &mut Renderer = &mut self.renderer.as_mut().unwrap();

            let framerate = self.app_settings.target_frame_rate;
            let mut time_point = std::time::Instant::now();
            let mut accumulated_time = 0.0f32;
            let fixed_delta_time = {
                let world = self.world.as_mut().unwrap().borrow();
                world.resource::<Time>().fixed_delta_time
            };

            // Game loop [WIP]
            while !renderer.window.should_close() {
                let mut world: RefMut<World> = self.world.as_mut().unwrap().borrow_mut();
                renderer.poll_events();

                let dt = std::time::Instant::elapsed(&time_point);
                time_point = std::time::Instant::now();

                let delta = dt.as_secs_f32();
                let fixed_delta = fixed_delta_time;

                let mut time_world = world.resource_mut::<Time>();
                time_world.delta_time = delta;
                accumulated_time += delta;
                time_world.time += delta as f64;
                time_world.frames = time_world.frames + 1u64;

                // Update game logic once
                world.run_schedule(EcsUpdateSchedule);

                // Update physic&fixed at same time step.
                while accumulated_time >= fixed_delta {
                    accumulated_time -= fixed_delta;
                    world.run_schedule(EcsFixedUpdateSchedule);
                }

                // Late update for UI.
                world.run_schedule(EcsLateUpdateSchedule);
                drop(world); // Free mutable ref because RenderingBridge hold a mut ref to World

                // Bakes rendering commands
                let rendering_bridge = self.rendering_bridge.as_mut().unwrap();
                rendering_bridge.inject_new_rendering_entities(renderer);
                rendering_bridge.flush_rendering_command_handles(renderer);
                rendering_bridge.flush_camera_changes(renderer);

                // Render and forward overflow time
                renderer.render(accumulated_time);

                let sleep_time: f32 = f32::max((1.0f32 / framerate) - delta, 0f32);
                std::thread::sleep(Duration::from_secs_f32(sleep_time));
            }

            Ok(())
        }
    }
}
