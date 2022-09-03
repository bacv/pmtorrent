use crate::{
    merkle::{FileTree, MerkleTree},
    Chunk, Proof,
};

struct File {
    chunks: Vec<Chunk>,
    tree: FileTree,
}

impl File {
    pub fn new(data: &[u8]) -> Self {
        let chunks = Self::get_chunks(data);
        let tree = FileTree::new(&chunks);

        Self { chunks, tree }
    }

    pub fn get_chunk(&self, idx: usize) -> Result<(Chunk, Proof), String> {
        let chunk = self.chunks.get(idx).cloned().ok_or("invalid idx")?;
        let proof = self.tree.get(chunk.leaf_idx)?;

        Ok((chunk, proof))
    }

    fn get_chunks(data: &[u8]) -> Vec<Chunk> {
        todo!()
    }
}
