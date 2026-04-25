//! When "manual watch" is enabled, debounced filesystem events trigger re-runs of the
//! currently **checked** tests only.

use anyhow::{Context, Result};
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::{Duration, Instant};

use super::config::RunConfig;

/// Re-starts the watcher when settings change.
pub(crate) fn apply_manual_watch_config(
    root: &Path,
    run_config: &RunConfig,
    handle: &mut Option<ManualWatchHandle>,
) {
    if let Some(h) = handle.take() {
        h.stop();
    }
    if !run_config.manual_watch_enabled {
        return;
    }
    let delay = Duration::from_millis(run_config.manual_watch_delay_ms as u64);
    match start_manual_watch(root.to_path_buf(), delay) {
        Ok(h) => *handle = Some(h),
        Err(e) => {
            eprintln!("Could not start manual watch: {e}");
        }
    }
}

const POLL_TICK_MS: u64 = 150;

/// Returns true for `.cs` under the tree, excluding `bin`, `obj`, `.git`, `target`, etc.
fn is_watched_code_file(path: &Path, root: &Path) -> bool {
    if path
        .components()
        .any(|c| {
            let s = c.as_os_str().to_string_lossy();
            matches!(s.as_ref(), "bin" | "obj" | "target" | "node_modules" | "packages")
                || s.starts_with('.') && s != "."
    }) {
        return false;
    }
    if path.extension().and_then(|e| e.to_str()).map(|e| e.eq_ignore_ascii_case("cs")) != Some(true) {
        return false;
    }
    path.starts_with(root)
}

pub struct ManualWatchHandle {
    pub rx: mpsc::Receiver<()>,
    stop_tx: mpsc::Sender<()>,
}

impl ManualWatchHandle {
    /// Stop the background watcher thread.
    pub fn stop(&self) {
        let _ = self.stop_tx.send(());
    }
}

/// Watches `root` recursively. After `debounce` elapses with no new relevant `.cs` events, sends `()` on `rx` (may coalesce many saves into one tick).
pub fn start_manual_watch(root: PathBuf, debounce: Duration) -> Result<ManualWatchHandle> {
    let (event_tx, event_rx) = mpsc::channel();
    let (stop_tx, stop_rx) = mpsc::channel();

    let root_c = root.clone();
    std::thread::Builder::new()
        .name("dotest-manual-watch".to_string())
        .spawn(move || {
            if let Err(e) = run_watcher_thread(root_c, debounce, event_tx, stop_rx) {
                eprintln!("manual watch: {}", e);
            }
        })
        .context("spawn manual watch thread")?;

    Ok(ManualWatchHandle {
        rx: event_rx,
        stop_tx,
    })
}

fn run_watcher_thread(
    root: PathBuf,
    debounce: Duration,
    out_tx: mpsc::Sender<()>,
    stop_rx: mpsc::Receiver<()>,
) -> Result<()> {
    let (raw_tx, raw_rx) = mpsc::channel();
    let watch_root = root.clone();
    let mut watcher = RecommendedWatcher::new(
        move |res: Result<Event, notify::Error>| {
            if let Ok(ev) = res {
                for path in ev.paths {
                    if is_watched_code_file(&path, &watch_root) {
                        let _ = raw_tx.send(());
                    }
                }
            }
        },
        Config::default(),
    )?;

    watcher
        .watch(&root, RecursiveMode::Recursive)
        .with_context(|| format!("watch {}", root.display()))?;

    // `Some(t)` = fire a debounced re-run at instant `t`.
    let mut fire_at: Option<Instant> = None;

    loop {
        if stop_rx.try_recv().is_ok() {
            break;
        }

        match raw_rx.recv_timeout(Duration::from_millis(POLL_TICK_MS)) {
            Ok(()) => {
                fire_at = Some(Instant::now() + debounce);
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                if fire_at.is_none() {
                    break;
                }
            }
        }
        while let Ok(()) = raw_rx.try_recv() {
            fire_at = Some(Instant::now() + debounce);
        }

        if let Some(deadline) = fire_at {
            if Instant::now() >= deadline {
                fire_at = None;
                if out_tx.send(()).is_err() {
                    break;
                }
            }
        }
    }
    Ok(())
}
