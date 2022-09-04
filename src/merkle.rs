use crate::{Chunk, Hash, Proof};

pub trait MerkleTree<D> {
    fn get_proof_hashes(&self, idx: usize) -> Result<Proof, String>;
    fn get_sibling(&self, idx: usize) -> Result<(Hash, usize), String>;
    fn get_parent(&self, idx: usize) -> Result<(Hash, usize), String>;
    fn verify(&self, leaf: &D, hashes: Vec<Hash>) -> Result<(), String>;
}

pub struct FileTree {
    leaves: usize,
    height: usize,
    tree: Vec<Hash>,
}

impl FileTree {
    pub fn new(chunks: &[Chunk]) -> Self {
        todo!()
    }
}

impl MerkleTree<Chunk> for FileTree {
    fn get_proof_hashes(&self, idx: usize) -> Result<Proof, String> {
        let mut hashes = Proof::default();
        let mut idx = idx;

        for _ in 0..self.height {
            let (s_hash, s_idx) = self.get_sibling(idx)?;
            let (_, p_idx) = self.get_parent(s_idx)?;

            hashes.push(s_hash);
            idx = p_idx;
        }

        Ok(hashes)
    }

    fn get_sibling(&self, idx: usize) -> Result<(Hash, usize), String> {
        let sibling_idx = if idx % 2 == 0 { idx + 1 } else { idx - 1 };
        let hash = self
            .tree
            .get(sibling_idx)
            .ok_or_else(|| "invalid index".to_string())?
            .clone();

        Ok((hash, sibling_idx))
    }

    fn get_parent(&self, idx: usize) -> Result<(Hash, usize), String> {
        let node_count = self.tree.len();
        let parent_idx = node_count - (node_count - idx - 1 + idx % 2) / 2;

        let hash = self
            .tree
            .get(parent_idx)
            .ok_or_else(|| "invalid index".to_string())?
            .clone();

        Ok((hash, parent_idx))
    }

    fn verify(&self, leaf: &Chunk, hashes: Vec<Hash>) -> Result<(), String> {
        todo!()
    }
}

mod tests {
    #[test]
    #[ignore]
    fn get_correct_sibling() {
        use super::*;

        let mut tree = Vec::new();
        for i in 0..7 {
            tree.push(Hash([i; 32]));
        }

        let file_tree = FileTree {
            leaves: 0,
            height: 0,
            tree,
        };

        for i in 0..file_tree.tree.len() - 1 {
            let s = file_tree.get_sibling(i).unwrap();
            println!("i: {:?}; s: {:?}", i, s);
        }
    }

    #[test]
    #[ignore]
    fn node_parents() {
        let l = 8;
        // node count.
        let n = 2 * l - 1;

        // ignore the root, we know it's possition
        for i in 0..n - 1 {
            let p = n - (n - i - 1 + i % 2) / 2;
            println!("i: {:?}; p: {:?}", i, p);
        }
    }
}
