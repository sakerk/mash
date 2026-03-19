use crate::mhl::types::*;
use std::collections::HashMap;

/// Compare two manifests and produce a diff result.
pub fn compare_manifests(a: &Manifest, b: &Manifest) -> DiffResult {
    let mut a_map: HashMap<String, &ManifestEntry> = HashMap::new();
    for entry in &a.entries {
        a_map.insert(entry.path.clone(), entry);
    }

    let mut added = Vec::new();
    let mut modified = Vec::new();
    let mut unchanged = Vec::new();

    for entry_b in &b.entries {
        match a_map.remove(&entry_b.path) {
            None => {
                added.push(DiffEntry {
                    path: entry_b.path.clone(),
                    hashes: entry_b.hashes.clone(),
                    size: entry_b.size,
                });
            }
            Some(entry_a) => {
                if hashes_differ(entry_a, entry_b) {
                    modified.push(DiffModified {
                        path: entry_b.path.clone(),
                        hashes_a: entry_a.hashes.clone(),
                        hashes_b: entry_b.hashes.clone(),
                        size_a: entry_a.size,
                        size_b: entry_b.size,
                    });
                } else {
                    unchanged.push(DiffEntry {
                        path: entry_b.path.clone(),
                        hashes: entry_b.hashes.clone(),
                        size: entry_b.size,
                    });
                }
            }
        }
    }

    // Remaining in a_map are removed
    let mut removed: Vec<DiffEntry> = a_map
        .into_values()
        .map(|e| DiffEntry {
            path: e.path.clone(),
            hashes: e.hashes.clone(),
            size: e.size,
        })
        .collect();

    // Sort all categories by path
    added.sort_by(|a, b| a.path.cmp(&b.path));
    removed.sort_by(|a, b| a.path.cmp(&b.path));
    modified.sort_by(|a, b| a.path.cmp(&b.path));
    unchanged.sort_by(|a, b| a.path.cmp(&b.path));

    DiffResult {
        manifest_a: a.source_path.clone(),
        manifest_b: b.source_path.clone(),
        added,
        removed,
        modified,
        unchanged,
    }
}

/// Check if two entries have differing hashes for any common algorithm.
fn hashes_differ(a: &ManifestEntry, b: &ManifestEntry) -> bool {
    let a_map: HashMap<HashAlgorithm, &str> =
        a.hashes.iter().map(|(algo, val)| (*algo, val.as_str())).collect();

    for (algo, val_b) in &b.hashes {
        if let Some(val_a) = a_map.get(algo) {
            if !val_a.eq_ignore_ascii_case(val_b) {
                return true;
            }
        }
    }
    false
}
