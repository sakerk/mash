use crate::mhl::types::*;
use quick_xml::events::Event;
use std::io::BufRead;
use std::path::Path;

/// Read a manifest file (CSV, TSV, or MHL) into a common Manifest struct.
pub fn read_manifest(path: &Path) -> Result<Manifest, String> {
    let ext = path
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default();

    match ext.as_str() {
        "csv" => read_csv_tsv(path, ',', ManifestFormat::Csv),
        "tsv" => read_csv_tsv(path, '\t', ManifestFormat::Tsv),
        "mhl" => read_mhl(path),
        _ => Err(format!(
            "Unsupported file format '{}'. Use .csv, .tsv, or .mhl",
            ext
        )),
    }
}

/// Normalize a file path for comparison.
fn normalize_path(p: &str) -> String {
    let mut s = p.replace('\\', "/");
    if s.starts_with("./") {
        s = s[2..].to_string();
    }
    while s.starts_with('/') {
        s = s[1..].to_string();
    }
    s
}

// --- CSV / TSV reader ---

fn read_csv_tsv(path: &Path, separator: char, format: ManifestFormat) -> Result<Manifest, String> {
    let file =
        std::fs::File::open(path).map_err(|e| format!("Failed to open {}: {}", path.display(), e))?;
    let reader = std::io::BufReader::new(file);
    let mut lines = reader.lines();

    // Parse header
    let header_line = lines
        .next()
        .ok_or("File is empty")?
        .map_err(|e| format!("Read error: {}", e))?;
    let headers: Vec<&str> = header_line.split(separator).collect();

    // Find column indices
    let mut checksum_idx: Option<usize> = None;
    let mut algorithm_idx: Option<usize> = None;
    let mut path_idx: Option<usize> = None;
    let mut name_idx: Option<usize> = None;
    let mut size_idx: Option<usize> = None;

    for (i, h) in headers.iter().enumerate() {
        let h_trimmed = h.trim().trim_matches('"');
        match h_trimmed {
            "Checksum" => checksum_idx = Some(i),
            "Algorithm" => algorithm_idx = Some(i),
            "File Path" => path_idx = Some(i),
            "File Name" => name_idx = Some(i),
            "File Size" => size_idx = Some(i),
            _ => {}
        }
    }

    // Need at least a path column
    if path_idx.is_none() && name_idx.is_none() {
        return Err("CSV/TSV must contain a 'File Path' or 'File Name' column".to_string());
    }

    let mut entries = Vec::new();

    for line_result in lines {
        let line = line_result.map_err(|e| format!("Read error: {}", e))?;
        if line.trim().is_empty() {
            continue;
        }

        let fields = parse_csv_line(&line, separator);

        let file_path = path_idx
            .and_then(|i| fields.get(i))
            .map(|s| normalize_path(s))
            .or_else(|| name_idx.and_then(|i| fields.get(i)).map(|s| s.clone()))
            .unwrap_or_default();

        if file_path.is_empty() {
            continue;
        }

        let mut hashes = Vec::new();
        if let Some(idx) = checksum_idx {
            if let Some(hash_val) = fields.get(idx) {
                if !hash_val.is_empty() {
                    let algo = algorithm_idx
                        .and_then(|ai| fields.get(ai))
                        .and_then(|a| HashAlgorithm::from_str_loose(a))
                        .unwrap_or(HashAlgorithm::Xxh64);
                    hashes.push((algo, hash_val.clone()));
                }
            }
        }

        let size = size_idx
            .and_then(|i| fields.get(i))
            .and_then(|s| s.parse::<u64>().ok());

        entries.push(ManifestEntry {
            path: file_path,
            hashes,
            size,
        });
    }

    Ok(Manifest {
        source_path: path.to_string_lossy().to_string(),
        format,
        entries,
    })
}

/// Parse a CSV line respecting quoted fields.
fn parse_csv_line(line: &str, separator: char) -> Vec<String> {
    let mut fields = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut chars = line.chars().peekable();

    while let Some(c) = chars.next() {
        if in_quotes {
            if c == '"' {
                if chars.peek() == Some(&'"') {
                    current.push('"');
                    chars.next();
                } else {
                    in_quotes = false;
                }
            } else {
                current.push(c);
            }
        } else if c == '"' {
            in_quotes = true;
        } else if c == separator {
            fields.push(current.clone());
            current.clear();
        } else {
            current.push(c);
        }
    }
    fields.push(current);
    fields
}

// --- MHL XML reader ---

fn read_mhl(path: &Path) -> Result<Manifest, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;

    let mut reader = quick_xml::Reader::from_str(&content);
    let mut entries = Vec::new();

    // State tracking
    let mut in_hash = false;
    let mut in_directoryhash = false;
    let mut current_path = String::new();
    let mut current_size: Option<u64> = None;
    let mut current_hashes: Vec<(HashAlgorithm, String)> = Vec::new();
    let mut current_tag = String::new();
    let mut current_algo: Option<HashAlgorithm> = None;

    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();

                if tag == "hash" && !in_directoryhash {
                    in_hash = true;
                    current_path.clear();
                    current_size = None;
                    current_hashes.clear();
                } else if tag == "directoryhash" {
                    in_directoryhash = true;
                } else if in_hash && tag == "path" {
                    // Extract size attribute
                    for attr in e.attributes().flatten() {
                        let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                        if key == "size" {
                            let val = String::from_utf8_lossy(&attr.value).to_string();
                            current_size = val.parse().ok();
                        }
                    }
                } else if in_hash {
                    // Check if this tag is a hash algorithm
                    current_algo = HashAlgorithm::from_str_loose(&tag);
                }

                current_tag = tag;
            }
            Ok(Event::Text(ref e)) => {
                let text = e.unescape().unwrap_or_default().to_string();
                if in_hash && current_tag == "path" {
                    current_path = normalize_path(&text);
                } else if in_hash {
                    if let Some(algo) = current_algo {
                        if !text.trim().is_empty() {
                            current_hashes.push((algo, text.trim().to_string()));
                        }
                    }
                }
            }
            Ok(Event::End(ref e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if tag == "hash" && in_hash {
                    if !current_path.is_empty() {
                        entries.push(ManifestEntry {
                            path: current_path.clone(),
                            hashes: current_hashes.clone(),
                            size: current_size,
                        });
                    }
                    in_hash = false;
                } else if tag == "directoryhash" {
                    in_directoryhash = false;
                }
                current_algo = None;
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(format!("XML parse error: {}", e)),
            _ => {}
        }
    }

    Ok(Manifest {
        source_path: path.to_string_lossy().to_string(),
        format: ManifestFormat::Mhl,
        entries,
    })
}
