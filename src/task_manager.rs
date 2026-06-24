use std::sync::Mutex;
use std::thread::JoinHandle;

/// Manages background threads spawned by the TUI event loop.
///
/// Tracks all active `JoinHandle`s so they can be joined on shutdown,
/// preventing dangling threads from writing to closed channels.
pub struct TaskManager {
    handles: Mutex<Vec<JoinHandle<()>>>,
}

impl TaskManager {
    pub fn new() -> Self {
        Self {
            handles: Mutex::new(Vec::new()),
        }
    }

    /// Spawn a new background task and track its handle.
    pub fn spawn<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let handle = std::thread::Builder::new()
            .name("gitnapse-worker".into())
            .spawn(f)
            .expect("failed to spawn background thread");
        self.handles.lock().unwrap().push(handle);
    }

    /// Remove handles for threads that have already completed.
    pub fn cleanup(&self) {
        let mut handles = self.handles.lock().unwrap();
        handles.retain(|h| !h.is_finished());
    }

    /// Number of currently tracked (running) threads.
    pub fn active_count(&self) -> usize {
        let handles = self.handles.lock().unwrap();
        handles.len()
    }

    /// Join (wait for) all tracked threads to finish.
    /// Called during shutdown to ensure clean teardown.
    pub fn join_all(&self) {
        let mut handles = std::mem::take(&mut *self.handles.lock().unwrap());
        for h in handles.drain(..) {
            let _ = h.join();
        }
    }
}

impl Default for TaskManager {
    fn default() -> Self {
        Self::new()
    }
}
