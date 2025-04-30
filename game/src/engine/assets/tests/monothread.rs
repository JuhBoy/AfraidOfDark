use assets::MonoThreadFifoExecutor;
use assets::asset_server::AssetServer;
use assets::commons::{AssetFile, AssetHandle, AssetTexture, Signal, ThreadWork};
use assets::processors::files::{FileLoadTask, FileStorage};
use assets::processors::textures::{TextureStorage, TextureTask};
use assets::storage::ErasedAssetStorage;
use assets::storage_server::StorageServer;
use std::any::TypeId;
use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};

// this is the types used to filter the server requests and assotiate storages
struct TestAssetPngType;
struct TestAssetTextureType;

fn init_executor() -> (MonoThreadFifoExecutor, Arc<RwLock<StorageServer>>) {
    let exe = MonoThreadFifoExecutor::new();
    let file_storage: Arc<RwLock<dyn ErasedAssetStorage>> = Arc::new(RwLock::new(FileStorage {
        asset_to_index: HashMap::new(),
        storage: Vec::new(),
    }));
    let texture_storage: Arc<RwLock<dyn ErasedAssetStorage>> =
        Arc::new(RwLock::new(TextureStorage {
            data_by_handle: HashMap::new(),
        }));
    let server = Arc::new(RwLock::new(StorageServer {
        storages: HashMap::new(),
    }));
    server
        .write()
        .unwrap()
        .storages
        .insert(TypeId::of::<TestAssetPngType>(), file_storage.clone());
    server.write().unwrap().storages.insert(
        TypeId::of::<TestAssetTextureType>(),
        texture_storage.clone(),
    );

    (exe, server)
}

#[test]
fn mono_thread_should_complete_work_with_one_request_test() {
    let (mut exe, storage_server) = init_executor();
    let task = FileLoadTask::new(
        "./tests/fixtures/fixtures_abstract.json",
        storage_server.clone(),
    );

    let asset_hdl = AssetHandle {
        internal_id: 1,
        asset_type: TypeId::of::<TestAssetPngType>(),
    };

    exe.push(ThreadWork::new(task, asset_hdl));
    exe.init();
    exe.wait(asset_hdl); // thread blocking

    let mut storage_writer = storage_server.write().unwrap();
    let data: Result<AssetFile, &str> = storage_writer.pop_data::<AssetFile>(asset_hdl);

    exe.shutdown(Signal::StopWaitAllPendingWorks);

    assert!(data.is_ok());
    assert!(!data.unwrap().data.is_empty());
}

#[test]
fn mono_thread_should_complete_work_with_multiple_request_test() {
    let (mut exe, storage_server) = init_executor();
    let task = FileLoadTask::new(
        "./tests/fixtures/fixtures_abstract.json",
        storage_server.clone(),
    );

    // launch the executor internal thread and put it in waiting
    exe.init();

    let mut handles: Vec<AssetHandle> = Vec::new();
    for i in 0..50 {
        let handle = AssetHandle {
            internal_id: i,
            asset_type: TypeId::of::<TestAssetPngType>(),
        };
        handles.push(handle);
        exe.push(ThreadWork::new(task.clone(), handle));
    }

    // shutdown the executor internal thread, block thread until all works are completed
    exe.shutdown(Signal::StopWaitAllPendingWorks);

    for handle in handles {
        let mut storage_reader = storage_server.write().unwrap();
        let data: Result<AssetFile, &str> = storage_reader.pop_data::<AssetFile>(handle);
        assert!(data.is_ok());

        let string_data = String::from_utf8(data.unwrap().data);
        assert!(string_data.is_ok());
        assert!(string_data.iter().len() > 0);
    }
}

#[test]
fn mono_thread_should_load_single_texture() {
    let file_to_load = String::from("./tests/fixtures/texture_11.png");
    let (mut exe, storage_server) = init_executor();
    let thread_task = TextureTask {
        identifier: Cow::Owned(file_to_load),
        storage: storage_server.clone(),
    };
    let handle: AssetHandle = AssetHandle {
        internal_id: 1,
        asset_type: TypeId::of::<TestAssetTextureType>(),
    };
    let task_ref = Arc::new(Mutex::new(thread_task));
    let worker = ThreadWork::new(task_ref.clone(), handle);

    exe.push(worker);
    exe.init();
    exe.shutdown(Signal::StopWaitAllPendingWorks);

    let mut storage_writer = storage_server.write().unwrap();
    let data = storage_writer.pop_data::<AssetTexture>(handle);

    if let Err(err) = data {
        println!("error: {}", err);
    }

    assert!(data.is_ok());

    let texture: AssetTexture = data.unwrap();
    assert!(texture.width > 0);
    assert!(texture.height > 0);
    assert!(!texture.data.is_empty());
}

// fn should_process_texture_load_when_asset_server_got_request() {
//     let mut asset_server = AssetServer::default();
//     asset_server.init();
//
//     let handle: Option<AssetHandle> = asset_server
//         .push_texture_work("./tests/fixtures/texture_11.png".to_string());
//
// }
