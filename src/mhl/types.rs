use chrono::{DateTime, FixedOffset};
use std::fmt;

#[derive(Debug, Clone)]
pub struct HashList {
    pub creator_info: CreatorInfo,
    pub process_info: ProcessInfo,
    pub hashes: Vec<HashEntry>,
    pub directory_hashes: Vec<DirectoryHashEntry>,
}

#[derive(Debug, Clone)]
pub struct CreatorInfo {
    pub creation_date: DateTime<FixedOffset>,
    pub hostname: String,
    pub tool_name: String,
    pub tool_version: String,
    pub author: Option<AuthorInfo>,
    pub location: Option<String>,
    pub comment: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AuthorInfo {
    pub name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub role: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub process: ProcessType,
    pub root_hash: Option<RootHash>,
    pub ignore_patterns: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct RootHash {
    pub algorithm: HashAlgorithm,
    pub value: String,
}

#[derive(Debug, Clone)]
pub struct HashEntry {
    pub path: String,
    pub size: u64,
    pub creation_date: Option<DateTime<FixedOffset>>,
    pub last_modification_date: Option<DateTime<FixedOffset>>,
    pub hashes: Vec<HashValue>,
}

#[derive(Debug, Clone)]
pub struct DirectoryHashEntry {
    pub path: String,
    pub creation_date: Option<DateTime<FixedOffset>>,
    pub last_modification_date: Option<DateTime<FixedOffset>>,
    pub content_hash: HashValue,
    pub structure_hash: HashValue,
}

#[derive(Debug, Clone)]
pub struct HashValue {
    pub algorithm: HashAlgorithm,
    pub action: HashAction,
    pub hash_date: DateTime<FixedOffset>,
    pub value: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum HashAlgorithm {
    Xxh64,
    Xxh128,
    Xxh3,
    Md5,
    Sha1,
    C4,
}

impl HashAlgorithm {
    pub fn xml_tag(&self) -> &'static str {
        match self {
            HashAlgorithm::Xxh64 => "xxh64",
            HashAlgorithm::Xxh128 => "xxh128",
            HashAlgorithm::Xxh3 => "xxh3",
            HashAlgorithm::Md5 => "md5",
            HashAlgorithm::Sha1 => "sha1",
            HashAlgorithm::C4 => "c4",
        }
    }

    pub fn from_str_loose(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "xxh64" => Some(HashAlgorithm::Xxh64),
            "xxh128" => Some(HashAlgorithm::Xxh128),
            "xxh3" => Some(HashAlgorithm::Xxh3),
            "md5" => Some(HashAlgorithm::Md5),
            "sha1" => Some(HashAlgorithm::Sha1),
            "c4" => Some(HashAlgorithm::C4),
            _ => None,
        }
    }
}

impl fmt::Display for HashAlgorithm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.xml_tag())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum HashAction {
    Original,
    Verified,
    Failed,
}

impl HashAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            HashAction::Original => "original",
            HashAction::Verified => "verified",
            HashAction::Failed => "failed",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum ProcessType {
    InPlace,
    Transfer,
    Flatten,
}

impl ProcessType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ProcessType::InPlace => "in-place",
            ProcessType::Transfer => "transfer",
            ProcessType::Flatten => "flatten",
        }
    }
}

// --- Checksum file config ---

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ChecksumColumn {
    Checksum,
    Algorithm,
    FilePath,
    FileName,
    FileSize,
    CreationDate,
    LastModificationDate,
}

impl ChecksumColumn {
    pub fn header(&self) -> &'static str {
        match self {
            ChecksumColumn::Checksum => "Checksum",
            ChecksumColumn::Algorithm => "Algorithm",
            ChecksumColumn::FilePath => "File Path",
            ChecksumColumn::FileName => "File Name",
            ChecksumColumn::FileSize => "File Size",
            ChecksumColumn::CreationDate => "Created",
            ChecksumColumn::LastModificationDate => "Modified",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            ChecksumColumn::Checksum => "Checksum",
            ChecksumColumn::Algorithm => "Algorithm",
            ChecksumColumn::FilePath => "File Path (incl. name)",
            ChecksumColumn::FileName => "File Name",
            ChecksumColumn::FileSize => "File Size",
            ChecksumColumn::CreationDate => "Created",
            ChecksumColumn::LastModificationDate => "Modified",
        }
    }

    pub fn all() -> &'static [ChecksumColumn] {
        &[
            ChecksumColumn::Checksum,
            ChecksumColumn::Algorithm,
            ChecksumColumn::FilePath,
            ChecksumColumn::FileName,
            ChecksumColumn::FileSize,
            ChecksumColumn::CreationDate,
            ChecksumColumn::LastModificationDate,
        ]
    }

    pub fn from_str_loose(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "checksum" => Some(ChecksumColumn::Checksum),
            "algorithm" | "algo" => Some(ChecksumColumn::Algorithm),
            "path" | "filepath" => Some(ChecksumColumn::FilePath),
            "name" | "filename" => Some(ChecksumColumn::FileName),
            "size" | "filesize" => Some(ChecksumColumn::FileSize),
            "created" | "creationdate" => Some(ChecksumColumn::CreationDate),
            "modified" | "lastmodificationdate" => Some(ChecksumColumn::LastModificationDate),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ChecksumFileConfig {
    pub columns: Vec<ChecksumColumn>,
    pub separator: char,
}

// --- Manifest comparison types ---
#[allow(dead_code)]

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ManifestFormat {
    Csv,
    Tsv,
    Mhl,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Manifest {
    pub source_path: String,
    pub format: ManifestFormat,
    pub entries: Vec<ManifestEntry>,
}

#[derive(Debug, Clone)]
pub struct ManifestEntry {
    pub path: String,
    pub hashes: Vec<(HashAlgorithm, String)>,
    pub size: Option<u64>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DiffResult {
    pub manifest_a: String,
    pub manifest_b: String,
    pub added: Vec<DiffEntry>,
    pub removed: Vec<DiffEntry>,
    pub modified: Vec<DiffModified>,
    pub unchanged: Vec<DiffEntry>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DiffEntry {
    pub path: String,
    pub hashes: Vec<(HashAlgorithm, String)>,
    pub size: Option<u64>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DiffModified {
    pub path: String,
    pub hashes_a: Vec<(HashAlgorithm, String)>,
    pub hashes_b: Vec<(HashAlgorithm, String)>,
    pub size_a: Option<u64>,
    pub size_b: Option<u64>,
}

impl Default for ChecksumFileConfig {
    fn default() -> Self {
        Self {
            columns: vec![
                ChecksumColumn::Checksum,
                ChecksumColumn::FilePath,
                ChecksumColumn::FileName,
                ChecksumColumn::FileSize,
            ],
            separator: ',',
        }
    }
}
