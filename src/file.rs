use std::convert::TryInto;

use ring::digest;

use crate::{
    merkle::{AsBytes, Hasher, MerkleError, MerkleTree},
    Chunk, Hash,
};

const CHUNK_BYTES: usize = 1024;
const FILLER_HASH: Hash = Hash([0u8; 32]);

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

struct File {
    chunks: Vec<Chunk>,
    tree: ChunkMerkleTree,
    size: usize,
}

impl File {
    pub fn new(data: &[u8]) -> Result<Self, FileError> {
        let chunks = Self::get_chunks(data);
        let tree = ChunkMerkleTree::new(&chunks)?;

        Ok(Self {
            chunks,
            tree,
            size: data.len(),
        })
    }

    pub fn get_chunk(&self, idx: usize) -> Result<(Chunk, Vec<crate::Hash>), FileError> {
        let chunk = self.chunks.get(idx).cloned().ok_or(FileError::File)?;
        let proof = self.tree.get_proof_hashes(chunk.leaf_idx)?;

        Ok((chunk, proof))
    }

    fn get_chunks(data: &[u8]) -> Vec<Chunk> {
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

fn next_pow2(n: usize) -> usize {
    let mut n = n - 1;
    let mut i = 0;

    while i <= 4 {
        n |= n >> 2u8.pow(i);
        i += 1;
    }

    n + 1
}

pub struct Sha256Hasher;

impl Hasher for Sha256Hasher {
    type Hash = Hash;

    fn digest(&self, data: &[u8]) -> Hash {
        let h = digest::digest(&digest::SHA256, data);
        Hash(h.as_ref().try_into().expect("32 byte value"))
    }
}

pub struct ChunkMerkleTree {
    tree: Vec<Hash>,
}

impl ChunkMerkleTree {
    pub fn new(chunks: &[Chunk]) -> Result<Self, FileError> {
        let hasher = Sha256Hasher {};
        let tree = Self::build_tree(&hasher, chunks)?;

        Ok(Self { tree })
    }
}

impl MerkleTree<Chunk, Sha256Hasher> for ChunkMerkleTree {
    fn get_tree(&self) -> &[Hash] {
        &self.tree
    }

    fn build_first_level(
        hasher: &Sha256Hasher,
        leaves: &[Chunk],
    ) -> Result<Vec<<Sha256Hasher as Hasher>::Hash>, MerkleError> {
        let mut padded_hashes = leaves
            .iter()
            .map(|l| {
                if l.len() < CHUNK_BYTES {
                    let mut p = [0u8; CHUNK_BYTES];
                    for (i, b) in l.data.iter().enumerate() {
                        p[i] = *b;
                    }
                    return hasher.digest(&p);
                }
                hasher.digest(l.as_bytes())
            })
            .collect::<Vec<Hash>>();

        let next_pow2 = next_pow2(leaves.len());
        if next_pow2 != leaves.len() {
            padded_hashes.resize(next_pow2, FILLER_HASH);
        }

        Ok(padded_hashes)
    }
}

mod tests {
    use std::convert::TryInto;

    use crate::{
        file::FILLER_HASH,
        merkle::{Hasher, MerkleTree},
        Chunk,
    };

    use super::{next_pow2, ChunkMerkleTree, File, Sha256Hasher};

    #[test]
    fn test_bytes_to_chunks() {
        let data = [1u8; 6144]; // exactly 6 full chunks.
        let chunks = File::get_chunks(&data);

        assert_eq!(chunks.len(), 6);
        assert_eq!(chunks.get(5).unwrap().data.get(42).unwrap(), &1);

        let data = [1u8; 6145]; // 7 chunks, the last one has only one byte.
        let chunks = File::get_chunks(&data);

        assert_eq!(chunks.len(), 7);
        assert_eq!(chunks.get(6).unwrap().data.first().unwrap(), &1);
        assert_eq!(chunks.get(6).unwrap().data.get(1), None);
    }

    #[test]
    fn test_build_first_level() {
        let chunks = File::get_chunks(&[1u8; 6144]);
        let chunk_tree = ChunkMerkleTree::new(&chunks);
        assert!(chunk_tree.is_ok());

        let chunk_tree = chunk_tree.unwrap();
        assert_eq!(chunk_tree.tree.len(), 15);
        assert_eq!(chunk_tree.tree[6], FILLER_HASH);
        assert_eq!(chunk_tree.tree[7], FILLER_HASH);
    }

    #[test]
    fn test_next_pow2() {
        assert_eq!(next_pow2(6), 8);
        assert_eq!(next_pow2(9), 16);
    }
}
