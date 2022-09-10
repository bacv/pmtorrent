use merkle::AsBytes;
use serde::{Serialize, Serializer};
use std::fmt::Write;

mod file;
mod merkle;
mod repo;

pub use file::*;
pub use merkle::*;
pub use repo::*;

type Data = Vec<u8>;

#[derive(Clone, Debug)]
pub struct Chunk {
    pub data: Data,
    pub leaf_idx: usize,
}

impl Chunk {
    pub fn len(&self) -> usize {
        self.data.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl AsBytes for Chunk {
    fn as_bytes(&self) -> &[u8] {
        &self.data
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct Hash([u8; 32]);

impl AsBytes for Hash {
    fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

impl serde::Serialize for Chunk {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        s.serialize_str(&base64::encode(&self.data))
    }
}

impl serde::Serialize for Hash {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        s.serialize_str(&encode_hex(&self.0))
    }
}

pub fn encode_hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        write!(&mut s, "{:02x}", b).unwrap();
    }
    s
}

#[derive(Serialize, Clone, Debug)]
pub struct Piece {
    pub content: Chunk,
    pub proof: Vec<Hash>,
}
