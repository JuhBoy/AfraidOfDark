use std::{env, fs::File, io::{Read, Write}, path::PathBuf};

static ASSETS_PATH: &str = "assets/";
static SHADER_PATH: &str = "shaders/";
static TEXTURE_PATH: &str = "textures/";
static MESH_PATH: &str = "meshes/";
static MATERIAL_PATH: &str = "materials/";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType { 
    Shader,
    Texture,
    Mesh,
    Material
}

pub struct FileSystem;

impl FileSystem {
    pub fn load_file(file_path: &str, f_type: FileType) -> Result<String, String> {
        let file_path: String = FileSystem::get_path(file_path, f_type);

        match File::open(&file_path).as_mut() {
            Ok(file) => {
                let mut file_content: String = String::new();
                file.read_to_string(&mut file_content).expect(format!("Failed to read file {}", &file_path).as_str());
                Ok(file_content)
            }
            Err(_) => Err(String::from(format!("Could not open file {}", &file_path)))
        }
    }

    pub fn write_file(file_path: &str, contents: &str, f_type: FileType) {
        let asset_path: String = FileSystem::get_path(file_path, f_type);

        let mut file = File::create(&asset_path).expect("Could not create file");
        file.write_all(contents.as_bytes()).expect("Could not write to file");
    }

    fn get_path(file_path: &str, f_type: FileType) -> String {
        let current_dir: PathBuf = env::current_dir().expect("Could not get current directory");
        let path: String;

        match f_type {
            FileType::Shader => {
                path = current_dir.join(ASSETS_PATH).join(SHADER_PATH).join(file_path).into_os_string().into_string().unwrap();
            }
            FileType::Texture => {
                path = current_dir.join(ASSETS_PATH).join(TEXTURE_PATH).join(file_path).into_os_string().into_string().unwrap();
            }
            FileType::Mesh => {
                path = current_dir.join(ASSETS_PATH).join(MESH_PATH).join(file_path).into_os_string().into_string().unwrap();
            }
            FileType::Material => {
                path = current_dir.join(ASSETS_PATH).join(MATERIAL_PATH).join(file_path).into_os_string().into_string().unwrap();
            }
        }

        path
    }
}