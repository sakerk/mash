use chrono::{DateTime, FixedOffset, Local};
use std::fs;
use std::path::Path;

/// Get the current time as a DateTime<FixedOffset>.
pub fn now() -> DateTime<FixedOffset> {
    Local::now().fixed_offset()
}

/// Format a DateTime for MHL XML output (ISO 8601).
pub fn format_datetime(dt: &DateTime<FixedOffset>) -> String {
    dt.to_rfc3339()
}

/// Format bytes as a human-readable string (e.g., "12.5 GB").
pub fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * KB;
    const GB: u64 = 1024 * MB;
    const TB: u64 = 1024 * GB;

    if bytes >= TB {
        format!("{:.1} TB", bytes as f64 / TB as f64)
    } else if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// Format a number with comma separators (e.g., 1234567 → "1,234,567").
pub fn format_number(n: u64) -> String {
    let s = n.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result.chars().rev().collect()
}

/// Get file metadata: size, creation date, modification date.
pub struct FileMetadata {
    pub size: u64,
    pub created: Option<DateTime<FixedOffset>>,
    pub modified: Option<DateTime<FixedOffset>>,
}

pub fn get_file_metadata(path: &Path) -> std::io::Result<FileMetadata> {
    let meta = fs::metadata(path)?;
    let size = meta.len();

    let created = meta
        .created()
        .ok()
        .map(|t| DateTime::<Local>::from(t).fixed_offset());

    let modified = meta
        .modified()
        .ok()
        .map(|t| DateTime::<Local>::from(t).fixed_offset());

    Ok(FileMetadata {
        size,
        created,
        modified,
    })
}

/// Convert a path to use forward slashes (for MHL compatibility).
pub fn to_forward_slashes(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(512), "512 B");
        assert_eq!(format_bytes(1024), "1.0 KB");
        assert_eq!(format_bytes(1024 * 1024 * 5), "5.0 MB");
    }

    #[test]
    fn test_format_number() {
        assert_eq!(format_number(0), "0");
        assert_eq!(format_number(999), "999");
        assert_eq!(format_number(1000), "1,000");
        assert_eq!(format_number(1234567), "1,234,567");
    }
}
