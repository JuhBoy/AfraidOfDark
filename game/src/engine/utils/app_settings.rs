
#[derive(Debug, Copy, Clone)]
pub enum WindowMode { 
    Windowed,
    FullScreen
}

#[derive(Debug, Copy, Clone)]
pub struct WindowSettings { 
    pub width: u32,
    pub height: u32,
    pub mode: WindowMode
}

pub struct ApplicationSettings { 
    pub window: WindowSettings,
    pub app_name: String
}

impl ApplicationSettings { 
    pub fn default() -> ApplicationSettings { 
        ApplicationSettings { 
            app_name: String::from("Default"),
            window: WindowSettings { 
                width: 800,
                height: 600,
                mode: WindowMode::Windowed
            }
        }
    }
}