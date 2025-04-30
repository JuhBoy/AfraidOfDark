use crate::commons::AssetHandle;
use crate::storage::ErasedAssetStorage;
use std::any::TypeId;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[derive(Default)]
pub struct StorageServer {
    pub storages: HashMap<TypeId, Arc<RwLock<dyn ErasedAssetStorage>>>,
}

impl StorageServer {
    pub fn get_storage(&self, type_id: &TypeId) -> Option<Arc<RwLock<dyn ErasedAssetStorage>>> {
        self.storages.get(type_id).cloned()
    }

    pub fn push_data<T>(&self, hdl: AssetHandle, data: T) -> Result<usize, &'static str>
    where
        T: Send + 'static,
    {
        if let Some(storage) = self.storages.get(&hdl.asset_type) {
            let mut storage_writer = storage.write().unwrap();
            let save_result = storage_writer.save(hdl, Box::new(data));
            return save_result;
        }

        Err("[server container] failed to push data")
    }

    pub fn pop_data<T>(&mut self, asset_handle: AssetHandle) -> Result<T, &'static str>
    where
        T: Send + 'static,
    {
        if let Some(server) = self.storages.get(&asset_handle.asset_type) {
            let mut server_writer = server.write().unwrap();
            let data = server_writer.load(asset_handle)?;

            return match data.downcast::<T>() {
                Ok(d) => Ok(*d),
                Err(_) => Err("[server container] failed to downcast data"),
            };
        }

        Err("[server container] failed to get server")
    }

    pub fn handle(&self, type_id: &TypeId) -> bool {
        if !self.storages.contains_key(type_id) {
           return false;
        }

        true
    }
}
