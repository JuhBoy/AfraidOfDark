pub mod asset_server;
pub mod commons;
pub mod processors;
pub mod storage;
pub mod storage_server;

use crate::commons::{AssetHandle, Signal, State, TaskResult, ThreadWork};
use std::collections::{BTreeSet, VecDeque};
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;

type ThreadWorkerQueue = Arc<Mutex<VecDeque<Arc<Mutex<ThreadWork>>>>>;
type HandlesReadySet = Arc<RwLock<BTreeSet<AssetHandle>>>;

pub struct MonoThreadFifoExecutor {
    pub(crate) thread: Option<JoinHandle<()>>,
    pub(crate) pending_works: ThreadWorkerQueue,
    pub(crate) ready_handles: HandlesReadySet,
    pub running: bool,
    pub shutdown: Arc<Mutex<Signal>>,
}

impl MonoThreadFifoExecutor {
    pub fn new() -> Self {
        Self {
            thread: None,
            pending_works: Arc::new(Mutex::new(VecDeque::new())),
            ready_handles: Arc::new(RwLock::new(BTreeSet::new())),
            running: false,
            shutdown: Arc::new(Mutex::new(Signal::Nop)),
        }
    }

    pub fn init(&mut self) {
        if self.running {
            panic!("[assets] thread already running");
        }

        self.running = true;
        self.shutdown = Arc::new(Mutex::new(Signal::Nop));

        let shutdown = self.shutdown.clone();
        let pending_works = self.pending_works.clone();
        let completed_handles = self.ready_handles.clone();

        let main_thread_handle = thread::spawn(move || {
            loop {
                let shutdown_signal: Signal = {
                    let signal = *shutdown.lock().unwrap();
                    signal
                };
                match shutdown_signal {
                    Signal::Stop => {
                        break;
                    }
                    Signal::StopWaitAllPendingWorks => {
                        let len = pending_works.lock().unwrap().len();
                        if len == 0 {
                            break;
                        }
                    }
                    Signal::Nop => {}
                }

                match pending_works.lock().as_mut() {
                    Ok(queue) => {
                        let queue_item = queue.pop_front();
                        if queue_item.is_none() {
                            continue;
                        }

                        let worker_ptr = queue_item.unwrap();
                        let mut locked_worker = worker_ptr.lock();

                        if let Ok(worker) = locked_worker.as_mut() {
                            Self::update_state(worker, State::Running, TaskResult::Undefined);
                            let result = Self::invoke_task(worker);
                            Self::update_state(worker, State::Completed, result);

                            Self::set_completed(completed_handles.clone(), worker.asset_handle);
                        }
                    }
                    Err(_) => eprintln!("[assets] failed to acquire work queue"),
                }

                thread::sleep(Duration::from_millis(32));
            }
        });

        self.thread = Some(main_thread_handle);
    }

    pub fn push(&mut self, worker: ThreadWork) {
        let worker_ptr = Arc::from(Mutex::from(worker));
        self.pending_works
            .lock()
            .unwrap()
            .push_back(worker_ptr.clone());
    }

    pub fn wait(&mut self, asset_handle: AssetHandle) {
        let maybe_worker = {
            let pending_works = self.pending_works.lock().unwrap();
            let option = pending_works
                .iter()
                .find(|e| e.lock().unwrap().asset_handle.internal_id == asset_handle.internal_id);
            option.cloned()
        };

        match maybe_worker {
            Some(worker) => {
                let ptr = worker.clone();

                loop {
                    {
                        let completed = ptr.lock().unwrap().clone();
                        if completed.state == State::Completed {
                            break;
                        }
                    }
                    thread::sleep(Duration::from_millis(32));
                }
            }
            None => {}
        }
    }

    pub fn shutdown(&mut self, signal: Signal) {
        let wait_for_end_of_works = {
            let mut internal_signal = self
                .shutdown
                .lock()
                .expect("[assets] failed to lock shutdown signal");
            *internal_signal = signal;

            signal == Signal::StopWaitAllPendingWorks
        };

        if !wait_for_end_of_works {
            return;
        }

        if let Some(thread_hdl) = self.thread.take() {
            thread_hdl.join().unwrap();
        }
    }

    pub fn is_completed(&self, handle: &AssetHandle) -> bool {
        if let Ok(reader) = self.ready_handles.read() {
            return reader.contains(handle);
        }

        false
    }

    pub fn pop_handle(&mut self, handle: &AssetHandle) {
        if let Ok(mut writer) = self.ready_handles.write() {
            writer.remove(handle);
        }
    }

    fn set_completed(container: HandlesReadySet, handle: AssetHandle) {
        match container.write() {
            Ok(mut writer) => {
                writer.insert(handle);
            }
            Err(_) => eprintln!("[assets worker thread] failed to acquire handles set"),
        }
    }

    fn update_state(worker: &mut ThreadWork, state: State, result: TaskResult) {
        worker.state = state;
        worker.task_result = result;
    }

    fn invoke_task(worker: &mut ThreadWork) -> TaskResult {
        match worker.task.lock().as_mut() {
            Ok(task) => task.invoke(worker.asset_handle),
            Err(_) => TaskResult::Failed("[assets]: could not mutex lock on task program"),
        }
    }
}
