use crate::hashing::algorithms::{create_hasher, StreamHasher};
use crate::mhl::types::{HashAction, HashAlgorithm, HashEntry, HashValue};
use crate::util;
use rayon::prelude::*;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use walkdir::WalkDir;

const CHUNK_SIZE: usize = 256 * 1024; // 256 KB

#[derive(Debug, Clone)]
pub struct HashResult {
    pub entries: Vec<HashEntry>,
    pub root_path: PathBuf,
}

pub struct HashProgress {
    pub total_bytes: AtomicU64,
    pub processed_bytes: AtomicU64,
    pub total_files: AtomicUsize,
    pub processed_files: AtomicUsize,
    pub current_file: Mutex<String>,
    pub result: Mutex<Option<Result<HashResult, String>>>,
}

impl HashProgress {
    pub fn new() -> Self {
        Self {
            total_bytes: AtomicU64::new(0),
            processed_bytes: AtomicU64::new(0),
            total_files: AtomicUsize::new(0),
            processed_files: AtomicUsize::new(0),
            current_file: Mutex::new(String::new()),
            result: Mutex::new(None),
        }
    }
}

pub struct HashJob {
    pub cancel: Arc<AtomicBool>,
    pub progress: Arc<HashProgress>,
}

impl HashJob {
    pub fn new() -> Self {
        Self {
            cancel: Arc::new(AtomicBool::new(false)),
            progress: Arc::new(HashProgress::new()),
        }
    }
}

/// Discover all files in the directory, skipping `ascmhl/` and matching ignore patterns.
fn discover_files(root: &Path, ignore_patterns: &[String]) -> Vec<PathBuf> {
    let mut files = Vec::new();

    for entry in WalkDir::new(root).into_iter().filter_entry(|e| {
        let name = e.file_name().to_string_lossy();
        // Skip ascmhl directory
        if e.file_type().is_dir() && name == "ascmhl" {
            return false;
        }
        true
    }) {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        if !entry.file_type().is_file() {
            continue;
        }

        let path = entry.path().to_path_buf();
        let rel = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .to_string_lossy()
            .to_string();

        // Check ignore patterns (simple glob matching)
        let should_ignore = ignore_patterns.iter().any(|pattern| {
            let pattern = pattern.trim();
            if pattern.starts_with("*.") {
                let ext = &pattern[1..]; // e.g., ".tmp"
                rel.ends_with(ext)
            } else {
                rel.contains(pattern)
            }
        });

        if !should_ignore {
            files.push(path);
        }
    }

    files
}

/// Hash a single file with multiple algorithms.
fn hash_file(
    path: &Path,
    root: &Path,
    algorithms: &[HashAlgorithm],
    cancel: &AtomicBool,
    progress: &HashProgress,
) -> Result<Option<HashEntry>, String> {
    let rel_path = path
        .strip_prefix(root)
        .unwrap_or(path);
    let rel_str = util::to_forward_slashes(rel_path);

    // Update current file
    if let Ok(mut current) = progress.current_file.lock() {
        *current = rel_str.clone();
    }

    let meta = util::get_file_metadata(path).map_err(|e| format!("{}: {}", rel_str, e))?;

    let mut file = File::open(path).map_err(|e| format!("{}: {}", rel_str, e))?;

    // Create hashers for each algorithm
    let mut hashers: Vec<Box<dyn StreamHasher>> =
        algorithms.iter().map(|a| create_hasher(*a)).collect();

    let mut buf = vec![0u8; CHUNK_SIZE];
    loop {
        if cancel.load(Ordering::Relaxed) {
            return Ok(None);
        }

        let n = file.read(&mut buf).map_err(|e| format!("{}: {}", rel_str, e))?;
        if n == 0 {
            break;
        }

        for h in &mut hashers {
            h.update(&buf[..n]);
        }

        progress
            .processed_bytes
            .fetch_add(n as u64, Ordering::Relaxed);
    }

    let hash_date = util::now();
    let hash_values: Vec<HashValue> = hashers
        .into_iter()
        .map(|h| {
            let algo = h.algorithm();
            let value = h.finalize();
            HashValue {
                algorithm: algo,
                action: HashAction::Original,
                hash_date,
                value,
            }
        })
        .collect();

    progress.processed_files.fetch_add(1, Ordering::Relaxed);

    Ok(Some(HashEntry {
        path: rel_str,
        size: meta.size,
        creation_date: meta.created,
        last_modification_date: meta.modified,
        hashes: hash_values,
    }))
}

/// Run the hashing engine. This is meant to be called from a background thread.
pub fn run_hashing(
    root: PathBuf,
    algorithms: Vec<HashAlgorithm>,
    ignore_patterns: Vec<String>,
    job: Arc<HashJob>,
) {
    let cancel = &job.cancel;
    let progress = &job.progress;

    // Discovery phase — handle single file or directory
    let (files, effective_root) = if root.is_file() {
        (vec![root.clone()], root.parent().unwrap_or(&root).to_path_buf())
    } else {
        (discover_files(&root, &ignore_patterns), root.clone())
    };
    let root = effective_root;

    // Calculate total bytes
    let total_bytes: u64 = files
        .iter()
        .filter_map(|f| std::fs::metadata(f).ok())
        .map(|m| m.len())
        .sum();

    progress
        .total_files
        .store(files.len(), Ordering::Relaxed);
    progress.total_bytes.store(total_bytes, Ordering::Relaxed);

    if cancel.load(Ordering::Relaxed) {
        let mut result = progress.result.lock().unwrap();
        *result = Some(Err("Cancelled".to_string()));
        return;
    }

    // Parallel hashing
    let entries: Vec<Result<Option<HashEntry>, String>> = files
        .par_iter()
        .map(|path| hash_file(path, &root, &algorithms, cancel, progress))
        .collect();

    if cancel.load(Ordering::Relaxed) {
        let mut result = progress.result.lock().unwrap();
        *result = Some(Err("Cancelled".to_string()));
        return;
    }

    // Collect results
    let mut hash_entries = Vec::new();
    for entry_result in entries {
        match entry_result {
            Ok(Some(entry)) => hash_entries.push(entry),
            Ok(None) => {
                // Cancelled
                let mut result = progress.result.lock().unwrap();
                *result = Some(Err("Cancelled".to_string()));
                return;
            }
            Err(e) => {
                let mut result = progress.result.lock().unwrap();
                *result = Some(Err(e));
                return;
            }
        }
    }

    // Sort entries by path for deterministic output
    hash_entries.sort_by(|a, b| a.path.cmp(&b.path));

    let mut result = progress.result.lock().unwrap();
    *result = Some(Ok(HashResult {
        entries: hash_entries,
        root_path: root,
    }));
}

/// Synchronous version for CLI use.
pub fn run_hashing_sync(
    root: PathBuf,
    algorithms: Vec<HashAlgorithm>,
    ignore_patterns: Vec<String>,
    quiet: bool,
) -> Result<HashResult, String> {
    let job = Arc::new(HashJob::new());
    let progress = job.progress.clone();

    let job_clone = job.clone();
    let handle = std::thread::spawn(move || {
        run_hashing(root, algorithms, ignore_patterns, job_clone);
    });

    if !quiet {
        // Print progress to stderr
        loop {
            std::thread::sleep(std::time::Duration::from_millis(250));

            let total = progress.total_files.load(Ordering::Relaxed);
            let done = progress.processed_files.load(Ordering::Relaxed);
            let total_bytes = progress.total_bytes.load(Ordering::Relaxed);
            let done_bytes = progress.processed_bytes.load(Ordering::Relaxed);

            if let Ok(result) = progress.result.lock() {
                if result.is_some() {
                    break;
                }
            }

            let current = progress
                .current_file
                .lock()
                .map(|s| s.clone())
                .unwrap_or_default();

            eprint!(
                "\r{} / {} files  |  {} / {}  |  {}",
                done,
                total,
                util::format_bytes(done_bytes),
                util::format_bytes(total_bytes),
                current
            );
            // Clear rest of line
            eprint!("\x1b[K");
        }
        eprintln!();
    }

    handle.join().map_err(|_| "Hashing thread panicked".to_string())?;

    let result = progress
        .result
        .lock()
        .unwrap()
        .take()
        .unwrap_or_else(|| Err("No result produced".to_string()));

    result
}
