use std::collections::BinaryHeap;

use crate::{Chunk, Hash, Proof};

pub trait MerkleTree<D> {
    fn get(&self, idx: usize) -> Result<Proof, String>;
    fn verify(&self, leaf: &D, hashes: Vec<Hash>) -> Result<(), String>;
}

pub struct FileTree {
    tree: BinaryHeap<Hash>,
}

impl FileTree {
    pub fn new(chunks: &[Chunk]) -> Self {
        todo!()
    }
}

impl MerkleTree<Chunk> for FileTree {
    fn get(&self, idx: usize) -> Result<Proof, String> {
        todo!()
    }

    fn verify(&self, leaf: &Chunk, hashes: Vec<Hash>) -> Result<(), String> {
        todo!()
    }
}
