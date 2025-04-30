use crate::commons::{AssetFile, AssetHandle, AssetTexture, Signal, ThreadWork};
use crate::processors::files::FileStorage;
use crate::processors::textures::{TextureStorage, TextureTask};
use crate::{MonoThreadFifoExecutor, storage_server::StorageServer};
use std::any::TypeId;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};

pub struct AssetServer {
    storage_server: Arc<RwLock<StorageServer>>,
    executor: MonoThreadFifoExecutor,

    asset_handles_index: u32,
}

impl Default for AssetServer {
    fn default() -> Self {
        Self {
            storage_server: Arc::new(RwLock::new(StorageServer::default())),
            executor: MonoThreadFifoExecutor::new(),
            asset_handles_index: 0,
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

    pub fn shutdown(&mut self) {
        // stop the threaded executor, this will not block the current thread
        if self.executor.running {
            self.executor.shutdown(Signal::Stop);
        }
    }

    pub fn request_texture(&mut self, texture_name: String) -> Option<AssetHandle> {
        let request_type: TypeId = TypeId::of::<AssetTexture>();
        let type_has_storage: bool = { self.storage_server.read().unwrap().handle(&request_type) };
        if !type_has_storage {
            eprintln!("could not find a storage to handle the type");
            return None;
        }

        debug_assert!(self.asset_handles_index < u32::MAX);

        // spawn texture work as dyn data pointer
        self.asset_handles_index += 1;
        let handle = AssetHandle {
            internal_id: self.asset_handles_index,
            asset_type: request_type,
        };
        let texture_work: TextureTask = TextureTask::new(texture_name, self.storage_server.clone());
        let worker = ThreadWork::new(Arc::new(Mutex::new(texture_work)), handle);

        // push to the executor for later processing
        self.executor.push(worker);

        // returns a uniq handle for further identification
        Some(handle)
    }

    pub fn pop_asset<T>(&mut self, handle: AssetHandle) -> Option<T>
    where
        T: Sync + Send + 'static,
    {
        // @todo! this should be done only in debug mode as it is a develop mistake to not
        // subscribe a storage handler for a type id
        let type_has_storage: bool = {
            let storage_reader = self.storage_server.read().unwrap();
            storage_reader.handle(&TypeId::of::<T>())
        };
        if !type_has_storage {
            eprintln!("[asset_server] not storage for type T");
            return None;
        }

        // first check if the data is ready
        let handle_is_ready = self.executor.is_completed(&handle);
        if !handle_is_ready {
            return None;
        }
        self.executor.pop_handle(&handle);

        // finally fetch data from the storage, if the workers failed the data will be None
        let mut storage_writer = self.storage_server.write().unwrap();
        if let Ok(data) = storage_writer.pop_data::<T>(handle) {
            return Some(data);
        } else {
            eprintln!(
                "[asset_server] failed to load data for asset handle {:?}",
                handle.internal_id
            );
        }

        None
    }
}
