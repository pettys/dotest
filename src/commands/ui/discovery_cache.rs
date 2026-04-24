//! Fingerprint-based cache for `dotnet test -t` discovery. Skips discovery on startup when the
//! workspace looks unchanged since the last run.
//!
//! - **Git repo:** SHA-256 of `git rev-parse HEAD`, `git diff HEAD`, and `git status --porcelain`
//!   (so staged, unstaged, and untracked changes all invalidate the cache).
//! - **Otherwise:** SHA-256 over `.cs` / `.csproj` paths, sizes, and mtimes (shallow walk, skips `bin`/`obj`/etc.).

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::io;
use std::path::Path;
use std::process::Command;
use std::time::UNIX_EPOCH;

pub(crate) const CACHE_PATH: &str = ".dotest_cache.json";

#[derive(Serialize, Deserialize)]
struct DiscoveryCacheFile {
    fingerprint: String,
    tests: Vec<(String, String, usize)>,
}

/// `git rev-parse HEAD` + `git diff HEAD` + `git status --porcelain`, or `None` if not a usable git worktree.
fn try_git_fingerprint() -> Option<String> {
    let root = std::env::current_dir().ok()?;
    let inside = Command::new("git")
        .args(["rev-parse", "--is-inside-work-tree"])
        .current_dir(&root)
        .output()
        .ok()?;
    if !inside.status.success() {
        return None;
    }
    if !String::from_utf8_lossy(&inside.stdout).trim().eq_ignore_ascii_case("true") {
        return None;
    }

    let head = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(&root)
        .output()
        .ok()?;
    if !head.status.success() {
        return None;
    }

    let diff = Command::new("git")
        .args(["diff", "HEAD"])
        .current_dir(&root)
        .output()
        .ok()?;
    if !diff.status.success() {
        return None;
    }

    let porcelain = Command::new("git")
        .args(["status", "--porcelain=v1", "-u"])
        .current_dir(&root)
        .output()
        .ok()?;
    if !porcelain.status.success() {
        return None;
    }

    let mut h = Sha256::new();
    h.update(b"git-v1\0");
    h.update(&head.stdout);
    h.update(&diff.stdout);
    h.update(&porcelain.stdout);
    Some(format!("{:x}", h.finalize()))
}

fn hash_dir(path: &Path, depth: usize, max_depth: usize, h: &mut Sha256) -> io::Result<()> {
    if depth > max_depth {
        return Ok(());
    }
    let rd = match fs::read_dir(path) {
        Ok(r) => r,
        Err(_) => return Ok(()),
    };
    let mut entries: Vec<_> = rd.flatten().collect();
    entries.sort_by_key(|e| e.path());
    for e in entries {
        let name = e.file_name();
        let name_s = name.to_string_lossy();
        if name_s == "." || name_s == ".." {
            continue;
        }
        if name_s.starts_with('.') {
            continue;
        }
        if matches!(
            name_s.as_ref(),
            "bin" | "obj" | "target" | "node_modules" | "packages"
        ) {
            continue;
        }
        let p = e.path();
        if p.is_dir() {
            hash_dir(&p, depth + 1, max_depth, h)?;
        } else if let Some(ext) = p.extension().and_then(|x| x.to_str()) {
            let ext = ext.to_ascii_lowercase();
            if ext == "cs" || ext == "csproj" {
                h.update(p.to_string_lossy().as_bytes());
                h.update(&[0]);
                if let Ok(meta) = fs::metadata(&p) {
                    h.update(meta.len().to_le_bytes());
                    if let Ok(m) = meta.modified() {
                        if let Ok(d) = m.duration_since(UNIX_EPOCH) {
                            h.update(d.as_secs().to_le_bytes());
                            h.update(d.subsec_nanos().to_le_bytes());
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

fn filesystem_fingerprint() -> String {
    let mut h = Sha256::new();
    h.update(b"fs-v1\0");
    let _ = hash_dir(Path::new("."), 0, 10, &mut h);
    format!("{:x}", h.finalize())
}

pub(crate) fn compute_source_fingerprint() -> String {
    try_git_fingerprint().unwrap_or_else(filesystem_fingerprint)
}

pub(crate) fn try_load_cached_tests() -> Option<Vec<(String, String, usize)>> {
    let fp = compute_source_fingerprint();
    let s = fs::read_to_string(CACHE_PATH).ok()?;
    let file: DiscoveryCacheFile = serde_json::from_str(&s).ok()?;
    if file.fingerprint == fp {
        Some(file.tests)
    } else {
        None
    }
}

pub(crate) fn save_discovery_cache(tests: &[(String, String, usize)]) -> Result<()> {
    let fp = compute_source_fingerprint();
    let file = DiscoveryCacheFile {
        fingerprint: fp,
        tests: tests.to_vec(),
    };
    let s = serde_json::to_string(&file).context("serialize discovery cache")?;
    fs::write(CACHE_PATH, s).context("write discovery cache")?;
    Ok(())
}
