use crate::commons::{AssetFile, AssetHandle, TaskResult, ThreadTask};
use crate::storage::AssetStorage;
use crate::storage_server::StorageServer;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::sync::{Arc, Mutex, RwLock};

pub struct FileStorage {
    pub storage: Vec<AssetFile>,
    pub asset_to_index: HashMap<AssetHandle, usize>,
}

pub struct FileLoadTask<'a> {
    file_path: &'a str,
    storage: Arc<RwLock<StorageServer>>,
}

impl AssetStorage for FileStorage {
    type InputData = AssetFile;

    fn save(
        &mut self,
        asset_hdl: AssetHandle,
        data: Self::InputData,
    ) -> Result<usize, &'static str> {
        self.storage.push(data);
        self.asset_to_index
            .insert(asset_hdl, self.storage.len() - 1);

        Ok(self.storage.len() - 1)
    }

    fn load(&mut self, asset_hdl: AssetHandle) -> Result<Self::InputData, &'static str> {
        let hash_value = self.asset_to_index.remove(&asset_hdl);

        let data = match hash_value {
            Some(idx) => Ok(self.storage.get(idx).unwrap().clone()),
            None => Err("[file server] couldn't retrieve the asset handle requested"),
        };

        data
    }
}

impl FileLoadTask<'_> {
    pub fn new(file_path: &str, storage: Arc<RwLock<StorageServer>>) -> Arc<Mutex<FileLoadTask>> {
        Arc::from(Mutex::from(FileLoadTask { file_path, storage }))
    }
}

impl ThreadTask for FileLoadTask<'_> {
    fn invoke(&mut self, asset_handle: AssetHandle) -> TaskResult {
        let read_result: Result<Vec<u8>, String> = match File::open(self.file_path).as_mut() {
            Ok(file) => {
                let mut file_content: Vec<u8> = Vec::new();
                file.read_to_end(&mut file_content)
                    .expect("failed to read file");
                Ok(file_content)
            }
            Err(_) => Err(format!("could not open file {}", self.file_path)),
        };

        if let Ok(buffer) = read_result {
            let store_result = self
                .storage
                .write()
                .unwrap()
                .push_data::<AssetFile>(asset_handle, AssetFile { data: buffer });

            return match store_result {
                Ok(_store_index) => TaskResult::Success,
                Err(_) => TaskResult::Failed("[storage container] failed to store data"),
            };
        }

        TaskResult::Failed("failed to read file")
    }
}
