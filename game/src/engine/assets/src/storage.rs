use crate::commons::AssetHandle;
use std::any::Any;

pub trait AssetStorage {
    type InputData: Send + Sync;

    fn save(
        &mut self,
        asset_hdl: AssetHandle,
        data: Self::InputData,
    ) -> Result<usize, &'static str>;
    fn load(&mut self, asset_hdl: AssetHandle) -> Result<Self::InputData, &'static str>;
}

pub trait ErasedAssetStorage: Send + Sync {
    fn save(
        &mut self,
        asset_hdl: AssetHandle,
        data: Box<dyn Any + Send>,
    ) -> Result<usize, &'static str>;
    fn load(&mut self, asset_hdl: AssetHandle) -> Result<Box<dyn Any + Send>, &'static str>;
}

impl<T> ErasedAssetStorage for T
where
    T: AssetStorage + 'static + Send + Sync,
    T::InputData: 'static + Send + Sync,
{
    fn save(&mut self, hdl: AssetHandle, data: Box<dyn Any + Send>) -> Result<usize, &'static str> {
        match data.downcast::<T::InputData>() {
            Ok(d) => self.save(hdl, *d),
            Err(_) => Err("[erased asset server] save failed to downcast data"),
        }
    }

    fn load(&mut self, asset_hdl: AssetHandle) -> Result<Box<dyn Any + Send>, &'static str> {
        let data = self.load(asset_hdl);
        match data {
            Ok(da) => Ok(Box::new(da)),
            Err(_) => Err("[erased asset server] load failed to downcast data"),
        }
    }
}
