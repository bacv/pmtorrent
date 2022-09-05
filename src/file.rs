use crate::{
    merkle::{ChunkMerkleTree, MerkleTree},
    Chunk,
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
