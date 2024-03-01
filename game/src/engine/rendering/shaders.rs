#[derive(Debug, Copy, Clone)]
pub enum ShaderLoadType {
    AOT,
    OnDemand
}

#[derive(Copy, Clone, Debug)]
pub enum ShaderType {
    Vertex,
    Fragment
}

#[derive(Debug)]
pub struct ShaderInfo {
    pub file_name: String,
    pub load_type: ShaderLoadType,
    pub shader_type: ShaderType
}

#[derive(Debug)]
pub struct ShaderPack { 
    pub vertex: Option<ShaderInfo>,
    pub fragment: Option<ShaderInfo>
}

#[derive(Debug)]
pub struct Texture {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub channels: u32
}

#[derive(Debug)]
pub struct Material {
    pub color: glm::Vec4,
    pub render_priority: i8,
    pub main_texture: Option<String>,
    pub shaders: ShaderPack,
    pub pixel_per_unit: u8
}

impl ShaderInfo { 
    pub fn default(shader_type: ShaderType) -> ShaderInfo { 
        ShaderInfo {
            file_name: String::from("[[default]]"),
            load_type: ShaderLoadType::OnDemand,
            shader_type
        }
    }
}

impl Material {
    pub fn default(texture: Option<String>) -> Material {
        Material {
            color: glm::vec4(1.0, 1.0, 1.0, 1.0),
            render_priority: 0,
            main_texture: texture,
            shaders: ShaderPack { vertex: Option::Some(ShaderInfo::default(ShaderType::Vertex)), fragment: Option::Some(ShaderInfo::default(ShaderType::Fragment)) },
            pixel_per_unit: 100
        }
    }
}