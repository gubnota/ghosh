use crate::contracts::Result;
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use std::{
    path::Path,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::{RecvTimeoutError, sync_channel},
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

/// Bounded ingress and fixed debounce window: storms cannot grow memory or defer
/// notifications indefinitely. Drop cancels and joins the worker.
pub struct WorkspaceWatch {
    _watcher: RecommendedWatcher,
    stop: Arc<AtomicBool>,
    worker: Option<JoinHandle<()>>,
}
impl WorkspaceWatch {
    pub fn start(path: &Path, changed: impl Fn() + Send + 'static) -> Result<Self> {
        let dir = cap_std::fs::Dir::open_ambient_dir(path, cap_std::ambient_authority())?;
        let mut previous = super::storage::current_revision(&dir).map_err(|e| e.code);
        let (tx, rx) = sync_channel(1);
        let mut watcher = notify::recommended_watcher(move |_: notify::Result<notify::Event>| {
            let _ = tx.try_send(());
        })
        .map_err(|_| crate::contracts::AppError::new(crate::contracts::ErrorCode::Io))?;
        watcher
            .watch(path, RecursiveMode::Recursive)
            .map_err(|_| crate::contracts::AppError::new(crate::contracts::ErrorCode::Io))?;
        let stop = Arc::new(AtomicBool::new(false));
        let cancelled = stop.clone();
        let worker = thread::spawn(move || {
            let probe_interval = Duration::from_secs(2);
            let mut next_probe = Instant::now() + probe_interval;
            while !cancelled.load(Ordering::Acquire) {
                match rx.recv_timeout(Duration::from_millis(100)) {
                    Ok(()) => {
                        let until = Instant::now() + Duration::from_millis(250);
                        while Instant::now() < until && !cancelled.load(Ordering::Acquire) {
                            if let Err(RecvTimeoutError::Disconnected) = rx.recv_timeout(
                                until
                                    .saturating_duration_since(Instant::now())
                                    .min(Duration::from_millis(50)),
                            ) {
                                return;
                            }
                        }
                        if !cancelled.load(Ordering::Acquire) {
                            previous = super::storage::current_revision(&dir).map_err(|e| e.code);
                            next_probe = Instant::now() + probe_interval;
                            changed();
                        }
                    }
                    Err(RecvTimeoutError::Timeout) => (),
                    Err(RecvTimeoutError::Disconnected) => break,
                }
                // Native streams can lose startup/coalesced events. Revisions read
                // only bounded configuration files, never scripts, data or secrets.
                if Instant::now() >= next_probe && !cancelled.load(Ordering::Acquire) {
                    let revision = super::storage::current_revision(&dir).map_err(|e| e.code);
                    if revision != previous {
                        previous = revision;
                        changed();
                    }
                    next_probe = Instant::now() + probe_interval;
                }
            }
        });
        Ok(Self {
            _watcher: watcher,
            stop,
            worker: Some(worker),
        })
    }
}
impl Drop for WorkspaceWatch {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}
