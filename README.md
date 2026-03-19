# MASH - Media Asset Hash

A fast, cross-platform tool for hashing media files and generating [ASC MHL v2.0](https://github.com/ascmhl/mhl-specification) manifests and checksum files. Built for media production workflows where file integrity matters.

Single binary — runs as a GUI with no arguments, or as a full CLI with flags.

**Author:** Saker Klippsten

## Features

- **Hash algorithms:** XXH64 (default), XXH128, XXH3, MD5, SHA-1
- **ASC MHL v2.0 manifests** with chain file support (C4/SHA-512 hash chain)
- **Checksum files** in CSV or TSV with configurable, reorderable columns
- **Per-file sidecar files** (e.g., `clip.mov.mash`) with configurable extension, columns, and optional header
- **Manifest comparison/diff** — load two CSV, TSV, or MHL files and see what was added, removed, or modified
- **Single file or folder hashing** — point at a directory tree or a single file
- **Parallel hashing** via rayon for multi-core performance
- **Streaming** — 256KB chunked reads, constant memory regardless of file size
- **Cancellable** — cancel in-progress jobs from GUI or Ctrl+C in CLI
- **Dark themed GUI** with progress bar, clickable output paths, and inline help

## Screenshots

### GUI Mode
```
+------------------------------------------------------+
| MASH                    Hash  |  Compare        v0.1 |
+------------------------------------------------------+
| Target                                               |
|  [path/to/folder___________] [Folder] [File]         |
|  1,234 files | 12.5 GB                               |
+------------------------------------------------------+
| Algorithm                                            |
|  (x) XXH64  ( ) XXH128  ( ) XXH3  ( ) MD5  ( ) SHA1 |
+------------------------------------------------------+
| Output                                               |
|  [x] MHL manifest  [x] Checksum file  [ ] Per-file  |
+------------------------------------------------------+
| Creator Info (collapsible, MHL only)                  |
| Checksum File Settings (columns, format)              |
+------------------------------------------------------+
| [============ 45% ================              ]     |
|              [ Start Hashing ]                        |
+------------------------------------------------------+
```

## Installation

### Prerequisites

- [Rust](https://rustup.rs/) (1.70+ recommended)
- On Windows: MSVC build tools (Visual Studio Build Tools or Visual Studio with C++ workload)

### Build from source

```bash
git clone https://github.com/YOUR_USERNAME/mash.git
cd mash
cargo build --release
```

The binary will be at `target/release/mash.exe` (Windows) or `target/release/mash` (Linux/macOS).

### Install to PATH

```bash
cargo install --path .
```

Then run `mash` from anywhere.

## Usage

### GUI Mode

Run with no arguments to launch the graphical interface:

```bash
mash
```

### CLI Mode

```bash
# Hash a folder (default: XXH64, CSV output)
mash -d ./footage

# Hash a single file
mash -f ./clip.mov

# Use a different algorithm
mash -d ./footage --hash md5

# TSV instead of CSV
mash -d ./footage --format tsv

# MHL only, skip checksum file
mash -d ./footage --no-checksum

# Checksum file only, skip MHL
mash -d ./footage --no-mhl

# Generate per-file sidecar files
mash -d ./footage --per-file

# Per-file with custom extension and header
mash -d ./footage --per-file --per-file-ext md5 --per-file-header

# Set MHL creator info
mash -d ./footage --author "Jane Doe" --email "jane@example.com" --location "Stage 5"

# Configure checksum columns and order
mash -d ./footage --columns checksum,path,algorithm

# Compare two manifests
mash --compare manifest_a.csv manifest_b.mhl

# Show all options
mash --help
```

### CLI Flags

| Flag | Default | Description |
|------|---------|-------------|
| `-d, --directory <PATH>` | | Target folder to hash |
| `-f, --file <PATH>` | | Target file to hash |
| `--hash <ALGO>` | `xxh64` | Hash algorithm: `xxh64`, `xxh128`, `xxh3`, `md5`, `sha1` |
| `--format <FMT>` | `csv` | Checksum file format: `csv` or `tsv` |
| `--columns <COLS>` | `checksum,path,algorithm` | Columns and order (comma-separated) |
| `--no-mhl` | | Skip MHL manifest generation |
| `--no-checksum` | | Skip checksum file generation |
| `--per-file` | | Generate per-file sidecar files |
| `--per-file-header` | | Include header row in sidecar files |
| `--per-file-ext <EXT>` | `mash` | Sidecar file extension |
| `--author <NAME>` | | Author name for MHL |
| `--email <EMAIL>` | | Author email for MHL |
| `--location <LOC>` | | Location for MHL |
| `--comment <TEXT>` | | Comment for MHL |
| `--ignore <PATTERNS>` | | Ignore patterns (comma-separated, e.g. `*.tmp,*.log`) |
| `--compare <A> <B>` | | Compare two manifest files |
| `--quiet` | | Suppress progress output |

### Available Columns

| Column | Description |
|--------|-------------|
| `checksum` | Hash value |
| `algorithm` | Algorithm name (e.g. XXH64, MD5) |
| `path` | Relative file path (includes file name) |
| `name` | File name only |
| `size` | File size in bytes |
| `created` | File creation date |
| `modified` | Last modification date |

## Output Files

### ASC MHL v2.0 Manifest

Written to an `ascmhl/` subdirectory following the ASC MHL naming convention:

```
footage/
  ascmhl/
    0001_footage_2026-03-19_143022.mhl
    ascmhl_chain.xml
```

The MHL file is a valid ASC MHL v2.0 XML document. The chain file tracks C4 hashes (SHA-512 + base58) of each MHL generation for tamper detection.

### Checksum File

Written to the target folder:

```
footage/
  footage_checksums.csv
```

### Per-file Sidecars

Written next to each source file:

```
footage/
  clip_001.mov
  clip_001.mov.mash
  clip_002.mov
  clip_002.mov.mash
```

## Built With

- **[Rust](https://www.rust-lang.org/)** — systems programming language
- **[egui](https://github.com/emilk/egui) / [eframe](https://github.com/emilk/egui/tree/master/crates/eframe)** — immediate mode GUI framework
- **[clap](https://github.com/clap-rs/clap)** — CLI argument parsing
- **[xxhash-rust](https://github.com/DoumanAsh/xxhash-rust)** — XXH64, XXH128, XXH3 hash implementations
- **[rayon](https://github.com/rayon-rs/rayon)** — parallel iteration for multi-core hashing
- **[quick-xml](https://github.com/tafia/quick-xml)** — XML reading/writing for MHL format
- **[walkdir](https://github.com/BurntSushi/walkdir)** — recursive directory traversal
- **[rfd](https://github.com/PolyMeilex/rfd)** — native file dialogs
- **[digest](https://github.com/RustCrypto/traits)** / **[md-5](https://github.com/RustCrypto/hashes)** / **[sha1](https://github.com/RustCrypto/hashes)** / **[sha2](https://github.com/RustCrypto/hashes)** — cryptographic hash implementations
- **[bs58](https://github.com/Nullus157/bs58-rs)** — Base58 encoding for C4 hashes
- **[chrono](https://github.com/chronotope/chrono)** — date/time handling

## Project Structure

```
src/
  main.rs                 # Entry point: CLI args -> CLI mode, else -> GUI
  cli.rs                  # CLI argument parsing and headless execution
  app.rs                  # eframe::App, GUI state machine, theming
  gui/
    folder_panel.rs       # Folder/file browser
    options_panel.rs      # Algorithm, output, column config panels
    creator_info.rs       # MHL author/location/comment fields
    progress_panel.rs     # Progress bar, completion, clickable output links
    compare_panel.rs      # Manifest diff/comparison view
  hashing/
    algorithms.rs         # StreamHasher trait + XXH64/XXH128/XXH3/MD5/SHA1/C4
    engine.rs             # Parallel streaming hash coordinator
    directory_hash.rs     # Content + structure hashes for directories
  mhl/
    types.rs              # Data structures (HashEntry, Manifest, DiffResult, etc.)
    writer.rs             # MHL XML serialization via quick-xml
    chain.rs              # ascmhl_chain.xml read/write + C4 hashing
    naming.rs             # MHL filename convention (sequence + timestamp)
  manifest_reader.rs      # Parse CSV/TSV/MHL into common Manifest format
  diff.rs                 # Manifest comparison engine
  checksum_file.rs        # CSV/TSV and per-file sidecar writer
  util.rs                 # Time formatting, byte formatting, file metadata
```

## License

Copyright 2026 Saker Klippsten

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) for details.
