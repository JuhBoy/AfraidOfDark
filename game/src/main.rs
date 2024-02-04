use engine::game::runtime::App;
use engine::utils::app_settings::{ApplicationSettings, WindowSettings, WindowMode};

pub mod engine;

fn main() {
    let app_settings = ApplicationSettings { 
        window: WindowSettings {
            width: 800,
            height: 600,
            mode: WindowMode::Windowed
        },
        app_name: String::from("Afraid of Dark")
    };
    
    App::with_appsettings(app_settings).run();
}
