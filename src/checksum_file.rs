use crate::hashing::engine::HashResult;
use crate::mhl::types::{ChecksumColumn, ChecksumFileConfig};
use crate::util;
use std::fs::File;
use std::io::Write;
use std::path::Path;

/// Write a checksum file (CSV or TSV) from hash results.
pub fn write_checksum_file(
    result: &HashResult,
    config: &ChecksumFileConfig,
    output_path: &Path,
) -> Result<(), String> {
    let mut file =
        File::create(output_path).map_err(|e| format!("Failed to create checksum file: {}", e))?;

    let sep = config.separator;

    // Write header
    let headers: Vec<&str> = config.columns.iter().map(|c| c.header()).collect();
    writeln!(file, "{}", headers.join(&sep.to_string()))
        .map_err(|e| format!("Write error: {}", e))?;

    // Write entries
    for entry in &result.entries {
        let mut fields: Vec<String> = Vec::new();

        for col in &config.columns {
            let value = match col {
                ChecksumColumn::Checksum => entry
                    .hashes
                    .first()
                    .map(|h| h.value.clone())
                    .unwrap_or_default(),
                ChecksumColumn::Algorithm => entry
                    .hashes
                    .first()
                    .map(|h| h.algorithm.xml_tag().to_uppercase())
                    .unwrap_or_default(),
                ChecksumColumn::FilePath => entry.path.clone(),
                ChecksumColumn::FileName => {
                    Path::new(&entry.path)
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_default()
                }
                ChecksumColumn::FileSize => entry.size.to_string(),
                ChecksumColumn::CreationDate => entry
                    .creation_date
                    .as_ref()
                    .map(|dt| util::format_datetime(dt))
                    .unwrap_or_default(),
                ChecksumColumn::LastModificationDate => entry
                    .last_modification_date
                    .as_ref()
                    .map(|dt| util::format_datetime(dt))
                    .unwrap_or_default(),
            };

            // Escape CSV fields containing separator or quotes
            if sep == ',' && (value.contains(',') || value.contains('"') || value.contains('\n')) {
                fields.push(format!("\"{}\"", value.replace('"', "\"\"")));
            } else {
                fields.push(value);
            }
        }

        writeln!(file, "{}", fields.join(&sep.to_string()))
            .map_err(|e| format!("Write error: {}", e))?;
    }

    Ok(())
}

/// Write a sidecar file per hashed file.
pub fn write_per_file_checksums(
    result: &HashResult,
    config: &ChecksumFileConfig,
    include_header: bool,
    extension: &str,
) -> Result<u64, String> {
    let sep = config.separator;
    let ext = extension.trim_start_matches('.');
    let mut count = 0u64;

    for entry in &result.entries {
        let full_path = result.root_path.join(&entry.path.replace('/', "\\"));
        let sidecar_path = full_path.with_extension(format!(
            "{}.{}",
            full_path
                .extension()
                .map(|e| e.to_string_lossy().to_string())
                .unwrap_or_default(),
            ext
        ));

        let mut file = File::create(&sidecar_path)
            .map_err(|e| format!("Failed to create {}: {}", sidecar_path.display(), e))?;

        if include_header {
            let headers: Vec<&str> = config.columns.iter().map(|c| c.header()).collect();
            writeln!(file, "{}", headers.join(&sep.to_string()))
                .map_err(|e| format!("Write error: {}", e))?;
        }

        let mut fields: Vec<String> = Vec::new();

        for col in &config.columns {
            let value = match col {
                ChecksumColumn::Checksum => entry
                    .hashes
                    .first()
                    .map(|h| h.value.clone())
                    .unwrap_or_default(),
                ChecksumColumn::Algorithm => entry
                    .hashes
                    .first()
                    .map(|h| h.algorithm.xml_tag().to_uppercase())
                    .unwrap_or_default(),
                ChecksumColumn::FilePath => entry.path.clone(),
                ChecksumColumn::FileName => Path::new(&entry.path)
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default(),
                ChecksumColumn::FileSize => entry.size.to_string(),
                ChecksumColumn::CreationDate => entry
                    .creation_date
                    .as_ref()
                    .map(|dt| util::format_datetime(dt))
                    .unwrap_or_default(),
                ChecksumColumn::LastModificationDate => entry
                    .last_modification_date
                    .as_ref()
                    .map(|dt| util::format_datetime(dt))
                    .unwrap_or_default(),
            };

            if sep == ',' && (value.contains(',') || value.contains('"') || value.contains('\n')) {
                fields.push(format!("\"{}\"", value.replace('"', "\"\"")));
            } else {
                fields.push(value);
            }
        }

        write!(file, "{}", fields.join(&sep.to_string()))
            .map_err(|e| format!("Write error: {}", e))?;

        count += 1;
    }

    Ok(count)
}

/// Generate the default checksum filename based on folder name and format.
pub fn checksum_filename(root: &Path, separator: char) -> String {
    let folder_name = root
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "checksums".to_string());

    let ext = if separator == '\t' { "tsv" } else { "csv" };
    format!("{}_checksums.{}", folder_name, ext)
}
