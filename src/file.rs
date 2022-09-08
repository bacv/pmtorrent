use std::convert::TryInto;

use ring::digest;

use crate::{
    merkle::{Hasher, MerkleError, MerkleTree},
    Chunk, Hash,
};

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
}

impl File {
    pub fn new(data: &[u8]) -> Result<Self, FileError> {
        let chunks = Self::get_chunks(data);
        let tree = ChunkMerkleTree::new(&chunks)?;

        Ok(Self { chunks, tree })
    }

    pub fn get_chunk(&self, idx: usize) -> Result<(Chunk, Vec<crate::Hash>), FileError> {
        let chunk = self.chunks.get(idx).cloned().ok_or(FileError::File)?;
        let proof = self.tree.get_proof_hashes(chunk.leaf_idx)?;

        Ok((chunk, proof))
    }

    fn get_chunks(data: &[u8]) -> Vec<Chunk> {
        todo!()
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
}

impl MerkleTree<Chunk, Sha256Hasher> for ChunkMerkleTree {
    fn get_tree(&self) -> &[Hash] {
        &self.tree
    }
}
