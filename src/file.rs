use std::convert::TryInto;

use ring::digest;

use crate::{
    merkle::{self, AsBytes, Hasher, MerkleError, MerkleTree},
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

pub struct File {
    chunks: Vec<Chunk>,
    tree: ChunkMerkleTree,
}

impl File {
    pub fn new(data: &[u8]) -> Result<Self, FileError> {
        let chunks = Self::to_chunks(data);
        let tree = ChunkMerkleTree::new(&chunks)?;

        Ok(Self { chunks, tree })
    }

    pub fn get_root(&self) -> Result<Hash, FileError> {
        self.tree.root()
    }

    pub fn get_size(&self) -> usize {
        self.chunks.len()
    }

    pub fn get_chunk(&self, idx: usize) -> Result<(Chunk, Vec<crate::Hash>), FileError> {
        let chunk = self.chunks.get(idx).cloned().ok_or(FileError::File)?;
        let proof = self.tree.get_proof_hashes(chunk.leaf_idx)?;

        Ok((chunk, proof))
    }

    pub fn trusted_root(&self) -> Result<Hash, FileError> {
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

    pub fn root(&self) -> Result<Hash, FileError> {
        Ok(self
            .tree
            .last()
            .ok_or(FileError::Merkle(MerkleError::InvalidIdx))?
            .to_owned())
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
            .map(|l| pad_payload(hasher, l))
            .collect::<Vec<Hash>>();

        let next_pow2 = next_pow2(leaves.len());
        if next_pow2 != leaves.len() {
            padded_hashes.resize(next_pow2, FILLER_HASH);
        }

        Ok(padded_hashes)
    }
}

pub fn root_from_partial(
    hasher: &Sha256Hasher,
    leaf: &Chunk,
    leaf_idx: usize,
    leaf_count: usize,
    hashes: Vec<Hash>,
) -> Result<Hash, FileError> {
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

fn pad_payload(hasher: &Sha256Hasher, l: &Chunk) -> Hash {
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
        let chunks = File::to_chunks(&[1u8; 6144]);
        let chunk_tree = ChunkMerkleTree::new(&chunks);
        assert!(chunk_tree.is_ok());

        let chunk_tree = chunk_tree.unwrap();
        assert_eq!(chunk_tree.tree.len(), 15);
        assert_eq!(chunk_tree.tree[6], FILLER_HASH);
        assert_eq!(chunk_tree.tree[7], FILLER_HASH);
    }

    #[test]
    fn test_new_file() {
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
        assert_eq!(next_pow2(4), 4);
        assert_eq!(next_pow2(6), 8);
        assert_eq!(next_pow2(9), 16);
    }
}
