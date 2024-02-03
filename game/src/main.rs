use glfw::{Action, Context, Key, WindowEvent};
use engine::{game::runtime::App, logging::logs::Logger};

extern crate glfw;

pub mod engine;

fn main() {

    let mut app: App<'_> = App::new("Walk in the dark");

    let mut glfwInit: glfw::Glfw = glfw::init(glfw::fail_on_errors).unwrap();

    let (mut window, events) = glfwInit
        .create_window(900, 900, "Nemo Editor", glfw::WindowMode::Windowed)
        .expect("Failed to create window");

    window.set_key_polling(true);
    window.make_current();

    while !window.should_close() {
        glfwInit.poll_events();
        for (_, event) in glfw::flush_messages(&events) {
            handle_window_event(&mut window, event);
        }
    }
}

fn handle_window_event(window: &mut glfw::Window, event: glfw::WindowEvent) {
    match event {
        WindowEvent::Key(Key::Escape, _, Action::Press, _) => window.set_should_close(true),
        _ => {
            match event {
                WindowEvent::Key(k, a, b, c) => {
                    let name: String;

                    if let Some(s) = k.get_name() {
                        name = s.to_owned();
                    } else {
                        name = String::from("unknow touch");
                    }

                    let phrase = "Key not handled by engine ";
                    let mess = phrase.to_owned() + &name + "\n";
                }
                _ => {}
            }
        }
    }
}
