
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
    pub file_path: String,
    pub load_type: ShaderLoadType,
    pub shader_type: ShaderType
}