use crate::hasher::Sha256Hash;
use crate::merkle::{self, MerkleError, MerkleTree};
use crate::{AsBytes, Chunk, Hasher, Sha256Hasher};
use lazy_static::lazy_static;
use tokio::io::{AsyncRead, AsyncReadExt};

const CHUNK_BYTES: usize = 1024;
lazy_static! {
    static ref FILLER_HASH: Sha256Hash = Sha256Hash::new([0u8; 32]);
}

#[derive(Debug)]
pub enum FileError {
    Merkle(MerkleError),
    File,
}

impl From<MerkleError> for FileError {
    fn from(e: MerkleError) -> Self {
        FileError::Merkle(e)
    }
}

/// A structure to hold bytes of a file in chunks together with a custom merkle tree.
pub struct File {
    chunks: Vec<Chunk>,
    tree: ChunkMerkleTree,
}

impl File {
    pub async fn from_reader<R>(mut reader: R) -> Result<Self, FileError>
    where
        R: AsyncRead + Unpin,
    {
        let mut buf = [0; CHUNK_BYTES];
        let mut chunks = Vec::default();
        let mut idx = 0;

        loop {
            let bytes = reader
                .read(&mut buf[..])
                .await
                .map_err(|_| FileError::File)?;

            if bytes == 0 {
                break;
            }

            chunks.push(Chunk {
                data: buf[..bytes].to_vec(),
                leaf_idx: idx,
            });

            idx += 1;
        }

        let tree = ChunkMerkleTree::new(&chunks)?;
        Ok(Self { chunks, tree })
    }

    pub fn new(data: &[u8]) -> Result<Self, FileError> {
        let chunks = Self::to_chunks(data);
        let tree = ChunkMerkleTree::new(&chunks)?;

        Ok(Self { chunks, tree })
    }

    pub fn get_root(&self) -> Result<Sha256Hash, FileError> {
        self.tree.root()
    }

    pub fn get_size(&self) -> usize {
        self.chunks.len()
    }

    pub fn get_chunk(&self, idx: usize) -> Result<(Chunk, Vec<crate::Sha256Hash>), FileError> {
        let chunk = self.chunks.get(idx).cloned().ok_or(FileError::File)?;
        let proof = self.tree.get_proof_hashes(chunk.leaf_idx)?;

        Ok((chunk, proof))
    }

    pub fn trusted_root(&self) -> Result<Sha256Hash, FileError> {
        Ok(self.tree.tree.last().ok_or(FileError::File)?.to_owned())
    }

    fn to_chunks(data: &[u8]) -> Vec<Chunk> {
        let mut chunks = vec![];
        for (i, c) in data.chunks(CHUNK_BYTES).enumerate() {
            chunks.push(Chunk {
                data: c.to_owned(),
                leaf_idx: i,
            });
        }

        chunks
    }
}

pub struct ChunkMerkleTree {
    tree: Vec<Sha256Hash>,
}

impl ChunkMerkleTree {
    pub fn new(chunks: &[Chunk]) -> Result<Self, FileError> {
        let hasher = Sha256Hasher {};
        let tree = Self::build_tree(&hasher, chunks)?;

        Ok(Self { tree })
    }

    pub fn root(&self) -> Result<Sha256Hash, FileError> {
        Ok(self
            .tree
            .last()
            .ok_or(FileError::Merkle(MerkleError::InvalidIdx))?
            .to_owned())
    }
}

impl MerkleTree<Chunk, Sha256Hasher> for ChunkMerkleTree {
    fn get_tree(&self) -> &[Sha256Hash] {
        &self.tree
    }

    /// Custom implementation for [`MerkleTree::build_first_level`] method.
    /// It pads the last leaf if it doesn't have the exact size of `CHUNK_BYTES` and appends
    /// `FILLER_HASH` to the leaf vector if it's size is not in power of 2.
    fn build_first_level(
        hasher: &Sha256Hasher,
        leaves: &[Chunk],
    ) -> Result<Vec<<Sha256Hasher as Hasher>::Hash>, MerkleError> {
        let mut padded_hashes = leaves
            .iter()
            .map(|l| pad_payload(hasher, l))
            .collect::<Vec<Sha256Hash>>();

        let next_pow2 = next_pow2(leaves.len());
        if next_pow2 != leaves.len() {
            padded_hashes.resize(next_pow2, FILLER_HASH.clone());
        }

        Ok(padded_hashes)
    }
}

#[allow(dead_code)]
pub fn root_from_partial(
    hasher: &Sha256Hasher,
    leaf: &Chunk,
    leaf_idx: usize,
    leaf_count: usize,
    hashes: Vec<Sha256Hash>,
) -> Result<Sha256Hash, FileError> {
    let padded_leaf = if leaf.data.len() < CHUNK_BYTES {
        pad_data(leaf)
    } else {
        leaf.to_owned()
    };

    merkle::root_from_partial(hasher, &padded_leaf, leaf_idx, leaf_count, hashes)
        .map_err(FileError::Merkle)
}

fn pad_data(c: &Chunk) -> Chunk {
    let mut p = [0u8; CHUNK_BYTES];
    for (i, b) in c.as_bytes().iter().enumerate() {
        p[i] = *b;
    }
    Chunk {
        data: p.to_vec(),
        leaf_idx: c.leaf_idx,
    }
}

fn pad_payload(hasher: &Sha256Hasher, l: &Chunk) -> Sha256Hash {
    if l.len() < CHUNK_BYTES {
        let mut p = [0u8; CHUNK_BYTES];
        for (i, b) in l.as_bytes().iter().enumerate() {
            p[i] = *b;
        }
        return hasher.digest(&p);
    }
    hasher.digest(l.as_bytes())
}

fn next_pow2(n: usize) -> usize {
    let mut n = n - 1;
    let mut i = 0;

    while i <= 4 {
        n |= n >> 2u8.pow(i);
        i += 1;
    }

    n + 1
}

mod tests {
    #[test]
    fn test_bytes_to_chunks() {
        use super::*;

        let data = [1u8; 6144]; // exactly 6 full chunks.
        let chunks = File::to_chunks(&data);

        assert_eq!(chunks.len(), 6);
        assert_eq!(chunks.get(5).unwrap().data.get(42).unwrap(), &1);

        let data = [1u8; 6145]; // 7 chunks, the last one has only one byte.
        let chunks = File::to_chunks(&data);

        assert_eq!(chunks.len(), 7);
        assert_eq!(chunks.get(6).unwrap().data.first().unwrap(), &1);
        assert_eq!(chunks.get(6).unwrap().data.get(1), None);
    }

    #[test]
    fn test_build_first_level() {
        use super::*;

        let chunks = File::to_chunks(&[1u8; 6144]);
        let chunk_tree = ChunkMerkleTree::new(&chunks);
        assert!(chunk_tree.is_ok());

        let chunk_tree = chunk_tree.unwrap();
        assert_eq!(chunk_tree.tree.len(), 15);
        assert_eq!(chunk_tree.tree[6], FILLER_HASH.clone());
        assert_eq!(chunk_tree.tree[7], FILLER_HASH.clone());
    }

    #[test]
    fn test_new_file() {
        use super::*;

        let data = [0u8; 6145];
        let file = File::new(&data).unwrap();
        assert_eq!(file.chunks.len(), 7);

        let (chunk, proof) = file.get_chunk(6).unwrap();
        assert_eq!(chunk.data.first().unwrap(), &0);
        assert_eq!(chunk.data.get(1), None);
        assert_eq!(proof.len(), 3);

        let hasher = Sha256Hasher;
        let trusted_root = file.trusted_root().unwrap();
        let untrusted_root =
            super::root_from_partial(&hasher, &chunk, chunk.leaf_idx, 8, proof).unwrap();
        assert_eq!(untrusted_root, trusted_root);
    }

    #[test]
    fn test_next_pow2() {
        use super::*;

        assert_eq!(next_pow2(4), 4);
        assert_eq!(next_pow2(6), 8);
        assert_eq!(next_pow2(9), 16);
    }

    #[test]
    fn test_async_read() {
        assert_eq!(test_fail("aabb"), "2a2b".to_string());
        assert_eq!(test_fail("aaabbccc"), "3a2b3c".to_string());
        assert_eq!(test_fail("abcccaaba"), "1a1b3c2a1b1a".to_string());
    }

    #[allow(dead_code)]
    fn test_fail(txt: &str) -> String {
        let mut res = String::new();
        let mut counter = 0;
        let mut prev_char = char::default();

        for c in txt.chars() {
            if prev_char != c {
                if counter > 0 {
                    res.push_str(&counter.to_string());
                    res.push_str(&prev_char.to_string());
                }
                counter = 0;
            }

            counter += 1;
            prev_char = c;
        }

        if counter > 0 {
            res.push_str(&counter.to_string());
            res.push_str(&prev_char.to_string());
        }

        res
    }
}
