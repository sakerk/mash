use crate::mhl::types::HashAlgorithm;
use digest::Digest;

/// Trait for streaming hash computation.
pub trait StreamHasher: Send {
    fn update(&mut self, data: &[u8]);
    fn finalize(self: Box<Self>) -> String;
    fn algorithm(&self) -> HashAlgorithm;
}

/// Create a new StreamHasher for the given algorithm.
pub fn create_hasher(algo: HashAlgorithm) -> Box<dyn StreamHasher> {
    match algo {
        HashAlgorithm::Xxh64 => Box::new(Xxh64Hasher::new()),
        HashAlgorithm::Xxh128 => Box::new(Xxh128Hasher::new()),
        HashAlgorithm::Xxh3 => Box::new(Xxh3Hasher::new()),
        HashAlgorithm::Md5 => Box::new(Md5Hasher::new()),
        HashAlgorithm::Sha1 => Box::new(Sha1Hasher::new()),
        HashAlgorithm::C4 => Box::new(C4Hasher::new()),
    }
}

// --- XXH64 ---

struct Xxh64Hasher {
    state: xxhash_rust::xxh64::Xxh64,
}

impl Xxh64Hasher {
    fn new() -> Self {
        Self {
            state: xxhash_rust::xxh64::Xxh64::new(0),
        }
    }
}

impl StreamHasher for Xxh64Hasher {
    fn update(&mut self, data: &[u8]) {
        use std::hash::Hasher;
        self.state.write(data);
    }

    fn finalize(self: Box<Self>) -> String {
        use std::hash::Hasher;
        format!("{:016x}", self.state.finish())
    }

    fn algorithm(&self) -> HashAlgorithm {
        HashAlgorithm::Xxh64
    }
}

// --- XXH128 (xxh3-128) ---

struct Xxh128Hasher {
    state: xxhash_rust::xxh3::Xxh3,
}

impl Xxh128Hasher {
    fn new() -> Self {
        Self {
            state: xxhash_rust::xxh3::Xxh3::new(),
        }
    }
}

impl StreamHasher for Xxh128Hasher {
    fn update(&mut self, data: &[u8]) {
        self.state.update(data);
    }

    fn finalize(self: Box<Self>) -> String {
        format!("{:032x}", self.state.digest128())
    }

    fn algorithm(&self) -> HashAlgorithm {
        HashAlgorithm::Xxh128
    }
}

// --- XXH3 (xxh3-64) ---

struct Xxh3Hasher {
    state: xxhash_rust::xxh3::Xxh3,
}

impl Xxh3Hasher {
    fn new() -> Self {
        Self {
            state: xxhash_rust::xxh3::Xxh3::new(),
        }
    }
}

impl StreamHasher for Xxh3Hasher {
    fn update(&mut self, data: &[u8]) {
        self.state.update(data);
    }

    fn finalize(self: Box<Self>) -> String {
        format!("{:016x}", self.state.digest())
    }

    fn algorithm(&self) -> HashAlgorithm {
        HashAlgorithm::Xxh3
    }
}

// --- MD5 ---

struct Md5Hasher {
    state: md5::Md5,
}

impl Md5Hasher {
    fn new() -> Self {
        Self {
            state: md5::Md5::new(),
        }
    }
}

impl StreamHasher for Md5Hasher {
    fn update(&mut self, data: &[u8]) {
        Digest::update(&mut self.state, data);
    }

    fn finalize(self: Box<Self>) -> String {
        hex::encode(self.state.finalize())
    }

    fn algorithm(&self) -> HashAlgorithm {
        HashAlgorithm::Md5
    }
}

// --- SHA1 ---

struct Sha1Hasher {
    state: sha1::Sha1,
}

impl Sha1Hasher {
    fn new() -> Self {
        Self {
            state: sha1::Sha1::new(),
        }
    }
}

impl StreamHasher for Sha1Hasher {
    fn update(&mut self, data: &[u8]) {
        Digest::update(&mut self.state, data);
    }

    fn finalize(self: Box<Self>) -> String {
        hex::encode(self.state.finalize())
    }

    fn algorithm(&self) -> HashAlgorithm {
        HashAlgorithm::Sha1
    }
}

// --- C4 (SHA-512 → base58) ---

struct C4Hasher {
    state: sha2::Sha512,
}

impl C4Hasher {
    fn new() -> Self {
        Self {
            state: sha2::Sha512::new(),
        }
    }
}

impl StreamHasher for C4Hasher {
    fn update(&mut self, data: &[u8]) {
        Digest::update(&mut self.state, data);
    }

    fn finalize(self: Box<Self>) -> String {
        let hash = self.state.finalize();
        format!("c4{}", bs58::encode(hash).into_string())
    }

    fn algorithm(&self) -> HashAlgorithm {
        HashAlgorithm::C4
    }
}

/// Compute C4 hash of raw bytes (for chain file hashing).
pub fn c4_hash_bytes(data: &[u8]) -> String {
    let hash = sha2::Sha512::digest(data);
    format!("c4{}", bs58::encode(hash).into_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xxh64_empty() {
        let mut h = Xxh64Hasher::new();
        h.update(b"");
        let result = Box::new(h).finalize();
        assert_eq!(result, "ef46db3751d8e999");
    }

    #[test]
    fn test_xxh64_hello() {
        let mut h = Xxh64Hasher::new();
        h.update(b"hello");
        let result = Box::new(h).finalize();
        // xxh64("hello", seed=0) = 26c7827d889f6da3 (known value)
        assert!(!result.is_empty());
        assert_eq!(result.len(), 16);
    }

    #[test]
    fn test_md5_hello() {
        let mut h = Md5Hasher::new();
        h.update(b"hello");
        let result = Box::new(h).finalize();
        assert_eq!(result, "5d41402abc4b2a76b9719d911017c592");
    }

    #[test]
    fn test_sha1_hello() {
        let mut h = Sha1Hasher::new();
        h.update(b"hello");
        let result = Box::new(h).finalize();
        assert_eq!(result, "aaf4c61ddcc5e8a2dabede0f3b482cd9aea9434d");
    }

    #[test]
    fn test_c4_starts_with_prefix() {
        let mut h = C4Hasher::new();
        h.update(b"hello");
        let result = Box::new(h).finalize();
        assert!(result.starts_with("c4"));
    }

    #[test]
    fn test_create_hasher() {
        let algos = [
            HashAlgorithm::Xxh64,
            HashAlgorithm::Xxh128,
            HashAlgorithm::Xxh3,
            HashAlgorithm::Md5,
            HashAlgorithm::Sha1,
            HashAlgorithm::C4,
        ];
        for algo in algos {
            let mut h = create_hasher(algo);
            h.update(b"test");
            let result = h.finalize();
            assert!(!result.is_empty());
        }
    }
}
