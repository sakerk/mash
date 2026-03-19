pub mod chain;
pub mod naming;
pub mod types;
pub mod writer;

use crate::hashing::directory_hash;
use crate::hashing::engine::HashResult;
use crate::mhl::types::*;
use crate::util;
use std::fs;

/// Generate MHL output: write the MHL file and update the chain.
pub fn generate_mhl(
    result: &HashResult,
    algorithms: &[HashAlgorithm],
    author: Option<AuthorInfo>,
    location: Option<String>,
    comment: Option<String>,
    ignore_patterns: Vec<String>,
) -> Result<String, String> {
    let root = &result.root_path;

    // Create ascmhl directory
    let ascmhl_dir = root.join("ascmhl");
    fs::create_dir_all(&ascmhl_dir)
        .map_err(|e| format!("Failed to create ascmhl directory: {}", e))?;

    // Compute directory hashes using the first algorithm
    let dir_algo = algorithms.first().copied().unwrap_or(HashAlgorithm::Xxh64);
    let directory_hashes = directory_hash::compute_directory_hashes(root, &result.entries, dir_algo);

    // Build HashList
    let host = hostname::get()
        .map(|h| h.to_string_lossy().to_string())
        .unwrap_or_else(|_| "unknown".to_string());

    let hashlist = HashList {
        creator_info: CreatorInfo {
            creation_date: util::now(),
            hostname: host,
            tool_name: "MASH".to_string(),
            tool_version: env!("CARGO_PKG_VERSION").to_string(),
            author,
            location,
            comment,
        },
        process_info: ProcessInfo {
            process: ProcessType::InPlace,
            root_hash: None,
            ignore_patterns,
        },
        hashes: result.entries.clone(),
        directory_hashes,
    };

    // Determine filename
    let seq = naming::next_sequence_number(&ascmhl_dir);
    let filename = naming::generate_mhl_filename(root, seq);
    let mhl_path = ascmhl_dir.join(&filename);

    // Write MHL XML
    let file = fs::File::create(&mhl_path)
        .map_err(|e| format!("Failed to create MHL file: {}", e))?;
    writer::write_mhl(&hashlist, file)?;

    // Update chain
    chain::add_to_chain(&ascmhl_dir, &filename, &mhl_path)?;

    Ok(mhl_path.to_string_lossy().to_string())
}
