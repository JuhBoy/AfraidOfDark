use crate::commons::{AssetFile, AssetHandle, AssetTexture, Signal, ThreadWork};
use crate::processors::files::FileStorage;
use crate::processors::textures::{TextureStorage, TextureTask};
use crate::uuid::Uuid;
use crate::{storage_server::StorageServer, MonoThreadFifoExecutor};
use std::any::TypeId;
use std::collections::HashMap;
use std::hash::Hasher;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, RwLock};

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct AssetServerConfig {
    pub asset_dir: PathBuf,
    pub texture_dir: PathBuf,
    pub shader_dir: PathBuf,
}

pub struct AssetServer {
    pub configuration: AssetServerConfig,

    storage_server: Arc<RwLock<StorageServer>>,
    executor: MonoThreadFifoExecutor,
    uuid: Uuid<AssetHandle>,
}

impl Default for AssetServer {
    fn default() -> Self {
        Self {
            storage_server: Arc::new(RwLock::new(StorageServer::default())),
            executor: MonoThreadFifoExecutor::new(),
            uuid: Uuid::<AssetHandle>::default(),

            configuration: AssetServerConfig {
                asset_dir: PathBuf::from("./assets"),
                texture_dir: PathBuf::from("./assets/textures"),
                shader_dir: PathBuf::from("./assets/shaders"),
            },
        }
    }
}

impl AssetServer {
    pub fn init(&mut self) {
        if !self.executor.running {
            self.executor.init();
        }

        let mut storage_writer = self.storage_server.write().unwrap();

        // default textures storage
        storage_writer.storages.insert(
            TypeId::of::<AssetTexture>(),
            Arc::new(RwLock::new(TextureStorage {
                data_by_handle: HashMap::new(),
            })),
        );

        // default files storage
        storage_writer.storages.insert(
            TypeId::of::<AssetFile>(),
            Arc::new(RwLock::new(FileStorage {
                storage: Vec::with_capacity(100),
                asset_to_index: Default::default(),
            })),
        );
    }

    pub fn shutdown(&mut self, signal: Signal) {
        if !self.executor.running {
            return;
        }

        self.executor.shutdown(signal);
    }

    pub fn push_texture_work(&mut self, texture_name: String) -> Option<AssetHandle> {
        if !self.is_type_handled_by_storage::<AssetTexture>() {
            eprintln!("could not find a storage to handle the type");
        }

        // @todo! think of a more optimized way to allocate those path later
        let mut file_path = PathBuf::from(&self.configuration.texture_dir);
        file_path.push(texture_name.clone());

        // spawn texture work as dyn data pointer
        let handle = AssetHandle {
            internal_id: self.uuid.new(&texture_name),
            asset_type: TypeId::of::<AssetTexture>(),
        };
        let texture_work: TextureTask =
            TextureTask::new(file_path.into(), self.storage_server.clone());
        let worker = ThreadWork::new(Arc::new(Mutex::new(texture_work)), handle);

        // push to the executor for later processing
        self.executor.push(worker);

        // returns a uniq handle for further identification
        Some(handle)
    }

    pub fn pop_asset<T>(&mut self, handle: AssetHandle) -> Result<T, &'static str>
    where
        T: Sync + Send + 'static,
    {
        if !self.is_type_handled_by_storage::<T>() {
            eprintln!("[asset_server] not storage for type T");
            return Err("handle type not found");
        }

        // first check if the data is ready
        if !self.executor.is_completed(&handle) {
            return Err("handle not completed");
        }
        self.executor.pop_handle(&handle);

        // pop data from storage server
        match self.storage_server.write() {
            Ok(mut writer) => {
                return writer.pop_data::<T>(handle);
            }
            Err(error) => {
                eprintln!("[asset_server] could not write to storage server {}", error);
            }
        }

        Err("[asset_server] storage server unexpectedly returned")
    }

    pub fn pop_texture_asset(&mut self, handle: AssetHandle) -> Result<AssetTexture, &'static str> {
        Self::pop_asset::<AssetTexture>(self, handle)
    }

    pub(crate) fn is_type_handled_by_storage<T>(&self) -> bool
    where
        T: 'static,
    {
        let request_type = TypeId::of::<T>();

        let handled = match self.storage_server.read() {
            Ok(reader) => reader.handle(&request_type),
            Err(_) => false,
        };

        handled
    }
}
