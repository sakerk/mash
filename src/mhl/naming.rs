use chrono::Local;
use std::path::Path;

/// Generate an MHL filename following the ASC MHL naming convention:
/// `NNNN_FOLDERNAME_YYYY-MM-DD_HHMMSS.mhl`
pub fn generate_mhl_filename(root_folder: &Path, sequence_number: u32) -> String {
    let folder_name = root_folder
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    let now = Local::now();
    let date_str = now.format("%Y-%m-%d_%H%M%S").to_string();

    format!("{:04}_{}_{}.mhl", sequence_number, folder_name, date_str)
}

/// Determine the next sequence number by scanning existing MHL files in the ascmhl directory.
pub fn next_sequence_number(ascmhl_dir: &Path) -> u32 {
    let mut max_seq = 0u32;

    if let Ok(entries) = std::fs::read_dir(ascmhl_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.ends_with(".mhl") && !name.starts_with("ascmhl_chain") {
                // Try to parse the first 4 characters as a sequence number
                if let Ok(seq) = name[..4.min(name.len())].parse::<u32>() {
                    max_seq = max_seq.max(seq);
                }
            }
        }
    }

    max_seq + 1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_next_sequence_number_empty() {
        // Non-existent directory should return 1
        let result = next_sequence_number(Path::new("/nonexistent_test_dir_12345"));
        assert_eq!(result, 1);
    }

    #[test]
    fn test_generate_mhl_filename() {
        let path = Path::new("C:/test/MyFolder");
        let name = generate_mhl_filename(path, 1);
        assert!(name.starts_with("0001_MyFolder_"));
        assert!(name.ends_with(".mhl"));
    }
}
