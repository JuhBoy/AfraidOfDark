use crate::{
    commons::{AssetHandle, AssetTexture, TaskResult, ThreadTask},
    storage::AssetStorage,
    storage_server::StorageServer,
};
use image::ImageReader;
use std::{
    borrow::Cow,
    collections::HashMap,
    sync::{Arc, RwLock},
};

pub struct TextureStorage {
    pub data_by_handle: HashMap<AssetHandle, AssetTexture>,
}

pub struct TextureTask<'a> {
    pub identifier: Cow<'a, str>,
    pub storage: Arc<RwLock<StorageServer>>,
}

impl AssetStorage for TextureStorage {
    type InputData = AssetTexture;

    fn save(&mut self, hdl: AssetHandle, data: AssetTexture) -> Result<usize, &'static str> {
        let inserted = self.data_by_handle.insert(hdl, data);

        match inserted {
            Some(texture) => Ok(texture.data.len()),
            None => Err("failed to insert texture in data storage"),
        }
    }

    fn load(&mut self, hdl: AssetHandle) -> Result<Self::InputData, &'static str> {
        if !self.data_by_handle.contains_key(&hdl) {
            return Err("couldn't find data for asset handle");
        }

        let data = self.data_by_handle.remove(&hdl);

        if let Some(data) = data {
            Ok(data)
        } else {
            Err("[texture storage] failed to extracts data from Texture Storage")
        }
    }
}

impl ThreadTask for TextureTask<'_> {
    fn invoke(&mut self, asset_handle: AssetHandle) -> TaskResult {
        let _hdl = asset_handle;

        let path: &str = &self.identifier;
        let img_reader = ImageReader::open(path);
        if img_reader.is_err() {
            return TaskResult::Failed("failed to open texture file");
        }

        match img_reader.unwrap().decode() {
            Ok(img) => {
                let width = img.width();
                let height = img.height();
                let channels = if img.color().has_alpha() { 4 } else { 3 }; // todo! handle 1-2-3 and 4 channels later

                let img = img.flipv();
                let data: Vec<u8> = img.into_bytes();

                let texture_file = AssetTexture {
                    data,
                    width,
                    height,
                    channels,
                };

                if let Ok(writer) = self.storage.write() {
                    let _ = writer.push_data(asset_handle, texture_file);
                }

                return TaskResult::Success;
            }
            Err(_) => TaskResult::Failed("failed to decode texture file"),
        };

        TaskResult::Success
    }
}

impl TextureTask<'_> {
    pub(crate) fn new(identifier: String, storage: Arc<RwLock<StorageServer>>) -> Self {
        Self {
            identifier: Cow::Owned::<'_>(identifier),
            storage,
        }
    }
}
