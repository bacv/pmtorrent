use crate::{
    merkle::{Hasher, MerkleTree},
    Chunk, Hash,
};

struct File {
    chunks: Vec<Chunk>,
    tree: ChunkMerkleTree,
}

impl File {
    pub fn new(data: &[u8]) -> Self {
        let chunks = Self::get_chunks(data);
        let tree = ChunkMerkleTree::new(&chunks);

        Self { chunks, tree }
    }

    pub fn get_chunk(&self, idx: usize) -> Result<(Chunk, Vec<crate::Hash>), String> {
        let chunk = self.chunks.get(idx).cloned().ok_or("invalid idx")?;
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
        todo!()
    }
}

pub struct ChunkMerkleTree {
    tree: Vec<Hash>,
}

impl ChunkMerkleTree {
    pub fn new(chunks: &[Chunk]) -> Self {
        let hasher = Sha256Hasher {};
        let tree = Self::build_tree(&hasher, chunks);

        Self { tree }
    }
}

impl MerkleTree<Chunk, Sha256Hasher> for ChunkMerkleTree {
    fn get_tree(&self) -> &[Hash] {
        &self.tree
    }
}
