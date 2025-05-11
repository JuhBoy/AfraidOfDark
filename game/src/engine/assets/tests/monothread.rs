use assets::MonoThreadFifoExecutor;
use assets::asset_server::AssetServer;
use assets::commons::{AssetFile, AssetHandle, AssetTexture, Signal, ThreadWork};
use assets::processors::files::{FileLoadTask, FileStorage};
use assets::processors::textures::{TextureStorage, TextureTask};
use assets::storage::ErasedAssetStorage;
use assets::storage_server::StorageServer;
use assets::uuid::Uuid;
use std::any::TypeId;
use std::borrow::Cow;
use std::collections::HashMap;
use std::ffi::OsString;
use std::path::PathBuf;
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
    let file_to_load = OsString::from("./tests/fixtures/texture_11.png");
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

#[test]
fn should_process_texture_loading_when_asset_server_got_request() {
    let mut asset_server = AssetServer::default();
    asset_server.configuration.texture_dir = PathBuf::from("./tests/fixtures/");

    let texture_name = String::from("texture_11.png");
    asset_server.init();

    let handle: Option<AssetHandle> = asset_server.push_texture_work(texture_name);
    assert!(handle.is_some());

    asset_server.shutdown(Signal::StopWaitAllPendingWorks);

    let result_data = asset_server.pop_texture_asset(handle.unwrap());
    assert!(result_data.is_ok());

    let texture = result_data.unwrap();
    assert!(!texture.data.is_empty());
}

#[test]
fn should_process_textures_loading_when_asset_server_got_requests() {
    let mut asset_server = AssetServer::default();
    asset_server.configuration.texture_dir = PathBuf::from("./tests/fixtures/");

    let texture_name = String::from("texture_11.png");
    asset_server.init();

    let mut handles: [AssetHandle; 50] = [AssetHandle {
        internal_id: 0,
        asset_type: TypeId::of::<AssetTexture>(),
    }; 50];

    for handle in handles.iter_mut() {
        let result = asset_server.push_texture_work(texture_name.clone());
        assert!(result.is_some());

        let r_handle = result.unwrap();
        *handle = r_handle;
    }

    asset_server.shutdown(Signal::StopWaitAllPendingWorks);

    for handle in handles.iter() {
        let data = asset_server.pop_asset::<AssetTexture>(*handle);
        assert!(data.is_ok());
        assert!(!data.unwrap().data.is_empty());
    }
}

#[test]
fn should_not_pop_asset_with_same_handle() {
    let mut asset_server = AssetServer::default();
    asset_server.configuration.texture_dir = PathBuf::from("./tests/fixtures/");

    let texture_name = String::from("texture_11.png");
    asset_server.init();

    let handle = asset_server.push_texture_work(texture_name);
    assert!(handle.is_some());

    let maybe_data = asset_server.pop_texture_asset(handle.unwrap());
    assert!(maybe_data.is_err());

    let data: Option<AssetTexture>;

    loop {
        match asset_server.pop_texture_asset(handle.unwrap()) {
            Ok(texture) => {
                data = Option::from(texture);
                break;
            }
            Err(err) => println!("error: {}", err),
        }
    }

    assert!(data.is_some());
    assert!(!data.unwrap().data.is_empty());

    let result = asset_server.pop_texture_asset(handle.unwrap());
    assert!(result.is_err());
}

#[test]
fn should_create_same_hash_handle_when_same_string_is_used() {
    let uuid: Uuid<usize> = Uuid::default();
    let identifier: String = String::from("abcdefgh.txt");

    let handle_a = AssetHandle {
        internal_id: uuid.new(&identifier),
        asset_type: TypeId::of::<AssetTexture>(),
    };
    let handle_b = AssetHandle {
        internal_id: uuid.new(&identifier),
        asset_type: TypeId::of::<AssetTexture>(),
    };

    assert!(handle_a == handle_b);
}
