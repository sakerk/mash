use crate::checksum_file;
use crate::diff;
use crate::hashing::engine;
use crate::manifest_reader;
use crate::mhl;
use crate::mhl::types::*;
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "mash",
    version,
    about = "MASH - Media Asset Hash",
    long_about = "\
MASH - Media Asset Hash
ASC MHL v2.0 manifest and checksum file generator

Run with no arguments to launch the GUI.

EXAMPLES:
  mash                                    Launch GUI
  mash -d ./footage                       Hash folder (xxh64, CSV output)
  mash -d ./footage --hash md5             Use MD5 instead of xxh64
  mash -d ./footage --format tsv          TSV instead of CSV
  mash -d ./footage --no-checksum         MHL only, skip checksum file
  mash -d ./footage --no-mhl              Checksum file only, skip MHL
  mash -d ./footage --author \"Jane Doe\"   Set MHL author
  mash -d ./footage --columns checksum,algorithm,path,size
  mash -f ./clip.mov                       Hash a single file
  mash --compare manifest_a.csv manifest_b.mhl

HASH ALGORITHMS:
  xxh64    XXH64 (default, fastest)
  xxh128   XXH128 (128-bit xxhash)
  xxh3     XXH3-64
  md5      MD5
  sha1     SHA-1

CHECKSUM COLUMNS:
  checksum   Hash value
  algorithm  Algorithm name (e.g. XXH64, MD5)
  path       Relative file path
  name       File name only
  size       File size in bytes
  created    File creation date
  modified   Last modification date"
)]
struct Cli {
    /// Target folder to hash
    #[arg(short = 'd', long = "directory")]
    directory: Option<PathBuf>,

    /// Target file to hash
    #[arg(short = 'f', long = "file")]
    file: Option<PathBuf>,

    /// Compare two manifest files (CSV, TSV, or MHL)
    #[arg(long = "compare", num_args = 2, value_names = ["FILE_A", "FILE_B"])]
    compare: Option<Vec<PathBuf>>,

    /// Hash algorithm: xxh64, xxh128, xxh3, md5, sha1
    #[arg(long = "hash", default_value = "xxh64")]
    hash: String,

    /// Checksum file format: csv or tsv
    #[arg(long = "format", default_value = "csv")]
    format: String,

    /// Columns and order for checksum file (comma-separated)
    #[arg(long = "columns", default_value = "checksum,path,algorithm", value_delimiter = ',')]
    columns: Vec<String>,

    /// Skip MHL file generation
    #[arg(long = "no-mhl")]
    no_mhl: bool,

    /// Skip checksum file generation
    #[arg(long = "no-checksum")]
    no_checksum: bool,

    /// Generate a .mash sidecar file per hashed file
    #[arg(long = "per-file")]
    per_file: bool,

    /// Include header row in per-file sidecars
    #[arg(long = "per-file-header")]
    per_file_header: bool,

    /// Extension for per-file sidecar files (default: mash)
    #[arg(long = "per-file-ext", default_value = "mash")]
    per_file_ext: String,

    /// Author name for MHL creator info
    #[arg(long)]
    author: Option<String>,

    /// Author email for MHL creator info
    #[arg(long)]
    email: Option<String>,

    /// Location for MHL creator info
    #[arg(long)]
    location: Option<String>,

    /// Comment for MHL creator info
    #[arg(long)]
    comment: Option<String>,

    /// Ignore patterns (comma-separated, e.g. "*.tmp,*.log")
    #[arg(long = "ignore", value_delimiter = ',')]
    ignore: Vec<String>,

    /// Suppress progress output
    #[arg(long)]
    quiet: bool,
}

pub fn run() {
    let cli = Cli::parse();

    // Compare mode
    if let Some(ref files) = cli.compare {
        run_compare(&files[0], &files[1]);
        return;
    }

    // Hash mode — need either -d or -f
    let target = match (&cli.directory, &cli.file) {
        (Some(d), _) => d.clone(),
        (_, Some(f)) => f.clone(),
        (None, None) => {
            eprintln!("Error: -d/--directory or -f/--file is required for hashing mode");
            eprintln!("Use --compare <FILE_A> <FILE_B> to compare manifests");
            eprintln!("Run mash --help for usage");
            std::process::exit(1);
        }
    };

    if !target.exists() {
        eprintln!("Error: Path does not exist: {}", target.display());
        std::process::exit(1);
    }
    if cli.directory.is_some() && !target.is_dir() {
        eprintln!("Error: Path is not a directory: {}", target.display());
        std::process::exit(1);
    }
    if cli.file.is_some() && !target.is_file() {
        eprintln!("Error: Path is not a file: {}", target.display());
        std::process::exit(1);
    }

    // For single file, output goes next to the file
    let output_dir = if target.is_file() {
        target.parent().unwrap_or(&target).to_path_buf()
    } else {
        target.clone()
    };

    let algorithm = HashAlgorithm::from_str_loose(&cli.hash).unwrap_or_else(|| {
        eprintln!("Error: Unknown hash algorithm: {}", cli.hash);
        eprintln!("Available: xxh64, xxh128, xxh3, md5, sha1");
        std::process::exit(1);
    });

    let separator = match cli.format.to_lowercase().as_str() {
        "csv" => ',',
        "tsv" => '\t',
        _ => {
            eprintln!("Error: Unknown format: {}. Use 'csv' or 'tsv'", cli.format);
            std::process::exit(1);
        }
    };

    let columns: Vec<ChecksumColumn> = cli
        .columns
        .iter()
        .map(|s| {
            ChecksumColumn::from_str_loose(s).unwrap_or_else(|| {
                eprintln!("Error: Unknown column: {}", s);
                eprintln!("Available: checksum, algorithm, path, name, size, created, modified");
                std::process::exit(1);
            })
        })
        .collect();

    let checksum_config = ChecksumFileConfig {
        columns,
        separator,
    };

    if !cli.quiet {
        eprintln!("MASH - Hashing: {}", target.display());
        eprintln!("Algorithm: {}", algorithm.xml_tag());
    }

    let result = match engine::run_hashing_sync(
        target.clone(),
        vec![algorithm],
        cli.ignore.clone(),
        cli.quiet,
    ) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };

    if !cli.quiet {
        eprintln!("Hashed {} files", result.entries.len());
    }

    if !cli.no_mhl {
        let author_info = cli.author.as_ref().map(|name| AuthorInfo {
            name: name.clone(),
            email: cli.email.clone(),
            phone: None,
            role: None,
        });

        match mhl::generate_mhl(
            &result,
            &[algorithm],
            author_info,
            cli.location.clone(),
            cli.comment.clone(),
            cli.ignore.clone(),
        ) {
            Ok(path) => {
                if !cli.quiet {
                    eprintln!("MHL written: {}", path);
                }
            }
            Err(e) => {
                eprintln!("Error writing MHL: {}", e);
                std::process::exit(1);
            }
        }
    }

    if !cli.no_checksum {
        let filename = checksum_file::checksum_filename(&output_dir, separator);
        let output_path = output_dir.join(&filename);

        match checksum_file::write_checksum_file(&result, &checksum_config, &output_path) {
            Ok(()) => {
                if !cli.quiet {
                    eprintln!("Checksum file written: {}", output_path.display());
                }
            }
            Err(e) => {
                eprintln!("Error writing checksum file: {}", e);
                std::process::exit(1);
            }
        }
    }

    if cli.per_file {
        match checksum_file::write_per_file_checksums(&result, &checksum_config, cli.per_file_header, &cli.per_file_ext) {
            Ok(count) => {
                if !cli.quiet {
                    eprintln!("Per-file: {} .mash sidecar files written", count);
                }
            }
            Err(e) => {
                eprintln!("Error writing per-file checksums: {}", e);
                std::process::exit(1);
            }
        }
    }
}

fn run_compare(file_a: &PathBuf, file_b: &PathBuf) {
    eprintln!("MASH - Comparing manifests");
    eprintln!("  A: {}", file_a.display());
    eprintln!("  B: {}", file_b.display());
    eprintln!();

    let manifest_a = match manifest_reader::read_manifest(file_a) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("Error reading {}: {}", file_a.display(), e);
            std::process::exit(1);
        }
    };

    let manifest_b = match manifest_reader::read_manifest(file_b) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("Error reading {}: {}", file_b.display(), e);
            std::process::exit(1);
        }
    };

    let result = diff::compare_manifests(&manifest_a, &manifest_b);

    // Summary
    println!(
        "Summary: {} added, {} removed, {} modified, {} unchanged",
        result.added.len(),
        result.removed.len(),
        result.modified.len(),
        result.unchanged.len()
    );
    println!();

    if !result.added.is_empty() {
        println!("ADDED ({} files - in B but not A):", result.added.len());
        for entry in &result.added {
            let hash_str = entry
                .hashes
                .first()
                .map(|(a, v)| format!(" [{}:{}]", a.xml_tag(), &v[..v.len().min(16)]))
                .unwrap_or_default();
            println!("  + {}{}", entry.path, hash_str);
        }
        println!();
    }

    if !result.removed.is_empty() {
        println!("REMOVED ({} files - in A but not B):", result.removed.len());
        for entry in &result.removed {
            let hash_str = entry
                .hashes
                .first()
                .map(|(a, v)| format!(" [{}:{}]", a.xml_tag(), &v[..v.len().min(16)]))
                .unwrap_or_default();
            println!("  - {}{}", entry.path, hash_str);
        }
        println!();
    }

    if !result.modified.is_empty() {
        println!(
            "MODIFIED ({} files - hash mismatch):",
            result.modified.len()
        );
        for entry in &result.modified {
            println!("  ~ {}", entry.path);
            for (algo, val) in &entry.hashes_a {
                let val_b = entry
                    .hashes_b
                    .iter()
                    .find(|(a, _)| a == algo)
                    .map(|(_, v)| v.as_str())
                    .unwrap_or("n/a");
                if !val.eq_ignore_ascii_case(val_b) {
                    println!("      {} A: {}", algo.xml_tag(), val);
                    println!("      {} B: {}", algo.xml_tag(), val_b);
                }
            }
        }
        println!();
    }

    if result.added.is_empty() && result.removed.is_empty() && result.modified.is_empty() {
        println!("All {} files match.", result.unchanged.len());
    }
}
