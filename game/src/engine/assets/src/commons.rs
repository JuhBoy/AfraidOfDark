use std::any::TypeId;
use std::sync::{Arc, Mutex};

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
#[derive(Ord, PartialOrd)]
pub struct AssetHandle {
    pub internal_id: u64,
    pub asset_type: TypeId,
}

#[derive(Eq, PartialEq, Hash, Clone, Copy)]
pub enum State {
    Init,
    Running,
    Completed,
}

#[derive(Eq, PartialEq, Hash, Clone, Copy)]
pub enum Signal {
    Nop,
    Stop,
    StopWaitAllPendingWorks,
}

#[derive(Eq, PartialEq, Hash, Clone, Copy)]
pub enum TaskResult {
    Undefined,
    Success,
    Failed(&'static str),
}

#[derive(Clone)]
pub struct AssetTexture {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub channels: u32,
}

#[derive(Clone)]
pub struct AssetFile {
    pub data: Vec<u8>,
}

#[derive(Clone)]
pub struct BytesFile {
    pub len: u32,
    pub data: Vec<u8>,
}

pub trait ThreadTask: Send + Sync {
    fn invoke(&mut self, asset_handle: AssetHandle) -> TaskResult;
}

#[derive(Clone)]
pub struct ThreadWork {
    pub task: Arc<Mutex<dyn ThreadTask>>,
    pub asset_handle: AssetHandle,
    pub state: State,
    pub task_result: TaskResult,
}

impl ThreadWork {
    pub fn new(task: Arc<Mutex<dyn ThreadTask>>, hdl: AssetHandle) -> Self {
        ThreadWork {
            task,
            asset_handle: hdl,
            state: State::Init,
            task_result: TaskResult::Undefined,
        }
    }
}
