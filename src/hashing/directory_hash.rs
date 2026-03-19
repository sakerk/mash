use crate::hashing::algorithms::create_hasher;
use crate::mhl::types::{
    DirectoryHashEntry, HashAction, HashAlgorithm, HashEntry, HashValue,
};
use crate::util;
use std::collections::BTreeMap;
use std::path::Path;

/// Compute directory hashes (content hash and structure hash) bottom-up.
///
/// Content hash: hash of sorted child hash values (concatenated hex strings).
/// Structure hash: hash of sorted child names (file names only).
pub fn compute_directory_hashes(
    root: &Path,
    entries: &[HashEntry],
    algorithm: HashAlgorithm,
) -> Vec<DirectoryHashEntry> {
    // Group files by their parent directory (relative paths)
    let mut dir_children: BTreeMap<String, Vec<&HashEntry>> = BTreeMap::new();

    for entry in entries {
        let parent = Path::new(&entry.path)
            .parent()
            .map(|p| util::to_forward_slashes(p))
            .unwrap_or_default();

        dir_children.entry(parent).or_default().push(entry);
    }

    // Collect all directory paths, including intermediate ones
    let mut all_dirs: Vec<String> = Vec::new();
    for dir in dir_children.keys() {
        let mut current = dir.clone();
        loop {
            if !all_dirs.contains(&current) {
                all_dirs.push(current.clone());
            }
            match current.rfind('/') {
                Some(pos) => current = current[..pos].to_string(),
                None => {
                    if !current.is_empty() && !all_dirs.contains(&String::new()) {
                        all_dirs.push(String::new());
                    }
                    break;
                }
            }
        }
    }

    // Sort dirs so deepest come first (bottom-up)
    all_dirs.sort_by(|a, b| {
        let depth_a = if a.is_empty() { 0 } else { a.matches('/').count() + 1 };
        let depth_b = if b.is_empty() { 0 } else { b.matches('/').count() + 1 };
        depth_b.cmp(&depth_a).then(a.cmp(b))
    });

    // Store computed hashes for directories so parent dirs can reference them
    let mut dir_hash_map: BTreeMap<String, (String, String)> = BTreeMap::new();
    let mut result = Vec::new();

    for dir in &all_dirs {
        // Gather content hash inputs: file hashes + subdirectory content hashes
        let mut content_parts: Vec<String> = Vec::new();
        let mut structure_parts: Vec<String> = Vec::new();

        // Add file entries in this directory
        if let Some(children) = dir_children.get(dir) {
            let mut sorted_children: Vec<&&HashEntry> = children.iter().collect();
            sorted_children.sort_by(|a, b| a.path.cmp(&b.path));

            for child in sorted_children {
                // Use the first hash value matching the algorithm, or any first hash
                let hash_val = child
                    .hashes
                    .iter()
                    .find(|h| h.algorithm == algorithm)
                    .or_else(|| child.hashes.first());

                if let Some(hv) = hash_val {
                    content_parts.push(hv.value.clone());
                }

                // Structure hash uses just the file name
                let name = Path::new(&child.path)
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();
                structure_parts.push(name);
            }
        }

        // Add subdirectory hashes
        let prefix = if dir.is_empty() {
            String::new()
        } else {
            format!("{}/", dir)
        };
        for (subdir, (content_h, _structure_h)) in &dir_hash_map {
            let is_direct_child = if prefix.is_empty() {
                !subdir.contains('/')
            } else {
                subdir.starts_with(&prefix) && !subdir[prefix.len()..].contains('/')
            };
            if is_direct_child {
                content_parts.push(content_h.clone());
                let subdir_name = Path::new(subdir)
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();
                structure_parts.push(subdir_name);
            }
        }

        content_parts.sort();
        structure_parts.sort();

        // Compute content hash
        let mut content_hasher = create_hasher(algorithm);
        for part in &content_parts {
            content_hasher.update(part.as_bytes());
        }
        let content_hash_value = content_hasher.finalize();

        // Compute structure hash
        let mut structure_hasher = create_hasher(algorithm);
        for part in &structure_parts {
            structure_hasher.update(part.as_bytes());
        }
        let structure_hash_value = structure_hasher.finalize();

        dir_hash_map.insert(dir.clone(), (content_hash_value.clone(), structure_hash_value.clone()));

        // Don't include root directory as "" in output
        let dir_path = if dir.is_empty() {
            ".".to_string()
        } else {
            dir.clone()
        };

        let hash_date = util::now();
        let dir_meta = root.join(dir.replace('/', "\\"));
        let meta = util::get_file_metadata(&dir_meta).ok();

        result.push(DirectoryHashEntry {
            path: dir_path,
            creation_date: meta.as_ref().and_then(|m| m.created),
            last_modification_date: meta.as_ref().and_then(|m| m.modified),
            content_hash: HashValue {
                algorithm,
                action: HashAction::Original,
                hash_date,
                value: content_hash_value,
            },
            structure_hash: HashValue {
                algorithm,
                action: HashAction::Original,
                hash_date,
                value: structure_hash_value,
            },
        });
    }

    // Reverse so root comes first
    result.reverse();
    result
}
