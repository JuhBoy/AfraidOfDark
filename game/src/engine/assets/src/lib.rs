pub mod asset_server;
pub mod commons;
pub mod processors;
pub mod storage;
pub mod storage_server;
pub mod uuid;

use crate::commons::{AssetHandle, Signal, State, TaskResult, ThreadWork};
use std::collections::{BTreeSet, VecDeque};
use std::sync::{Arc, Condvar, Mutex, RwLock};
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;

type ThreadWorkerQueue = Arc<Mutex<VecDeque<Arc<Mutex<ThreadWork>>>>>;
type HandlesReadySet = Arc<RwLock<BTreeSet<AssetHandle>>>;

pub struct MonoThreadFifoExecutor {
    pub(crate) thread: Option<JoinHandle<()>>,
    pub(crate) works_queue: ThreadWorkerQueue,
    pub(crate) ready_handles: HandlesReadySet,
    pub(crate) awaiter: Arc<Condvar>,
    pub running: bool,
    pub shutdown: Arc<Mutex<Signal>>,
}

impl MonoThreadFifoExecutor {
    pub fn new() -> Self {
        Self {
            thread: None,
            works_queue: Arc::new(Mutex::new(VecDeque::new())),
            ready_handles: Arc::new(RwLock::new(BTreeSet::new())),
            awaiter: Arc::new(Condvar::new()),
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
        let works_queue = self.works_queue.clone();
        let completed_handles = self.ready_handles.clone();
        let awaiter = self.awaiter.clone();

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
                        let len = works_queue.lock().unwrap().len();
                        if len == 0 {
                            break;
                        }
                    }
                    Signal::Nop => {}
                }

                // get worker pointer from pending queue and release the lock as soon as possible
                let worker = match works_queue.lock() {
                    Ok(queue) => queue.front().cloned(),
                    Err(_) => None,
                };

                // once the worker is acquired let's work! - only the worker is locked now
                if let Some(worker_ptr) = worker {
                    if let Ok(worker) = worker_ptr.lock().as_mut() {
                        Self::update_state(worker, State::Running, TaskResult::Undefined);
                        let task_result: TaskResult = Self::invoke_task(worker);
                        Self::update_state(worker, State::Completed, task_result);

                        Self::set_completed(completed_handles.clone(), worker.asset_handle);
                    }

                    // cleans the queue from the worker once completed
                    if let Ok(mut writer) = works_queue.lock() {
                        writer.pop_front();
                    }
                }

                if let Ok(mut queue_guard) = works_queue.lock() {
                    while queue_guard.is_empty() {
                        // ensure no signal has been sent before blocking the current thread
                        let signal = *shutdown.lock().unwrap();
                        if signal != Signal::Nop {
                            break;
                        }

                        queue_guard = awaiter.wait(queue_guard).unwrap();
                    }
                }
            }
        });

        self.thread = Some(main_thread_handle);
    }

    pub fn push(&mut self, worker: ThreadWork) {
        let worker_ptr = Arc::from(Mutex::from(worker));

        match self.works_queue.lock() {
            Ok(mut works_queue) => {
                works_queue.push_back(worker_ptr);
                self.awaiter.notify_all();
            }
            Err(_) => {
                eprintln!("[asset_server] failed to push worker to pending works queue");
            }
        }
    }

    /// block the current thread until the work associated with the asset Handle is done
    /// if the worker can't be acquired then the task is supposed to be completed (either because it's done or not available)
    pub fn wait(&mut self, asset_handle: AssetHandle) {
        let maybe_worker = {
            let works_queue = self.works_queue.lock().unwrap();
            let handle_worker = works_queue
                .iter()
                .find(|e| e.lock().unwrap().asset_handle == asset_handle);
            handle_worker.cloned()
        };

        match maybe_worker {
            Some(worker) => {
                let worker_ptr = worker.clone();

                loop {
                    let is_completed: bool = {
                        let value = worker_ptr.lock().unwrap();
                        value.state == State::Completed
                    };

                    if is_completed {
                        break;
                    }
                    thread::sleep(Duration::from_millis(1));
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
        
        // notify the thread so it awake to terminate if no work pending
        self.awaiter.notify_all();

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
