use crate::{Chunk, Hash};

pub trait Hasher {
    type Hash;

    fn digest(&self, data: &[u8]) -> Self::Hash;
}

pub trait AsBytes {
    fn as_bytes(&self) -> &[u8];
}

pub trait MerkleTree<D, H>
where
    D: AsBytes,
    H: Hasher,
    H::Hash: AsBytes + Clone,
{
    fn get_tree(&self) -> &[H::Hash];

    fn build_first_level(hasher: &H, leaves: &[D]) -> Vec<H::Hash> {
        leaves
            .iter()
            .map(|l| hasher.digest(l.as_bytes()))
            .collect::<Vec<H::Hash>>()
    }

    fn build_inner_level(hasher: &H, previous_level: &[H::Hash]) -> Vec<H::Hash> {
        previous_level
            .chunks(2)
            .map(|c| {
                let l = &c[0];
                let r = &c[1];
                hasher.digest(&[l.as_bytes(), r.as_bytes()].concat())
            })
            .collect::<Vec<H::Hash>>()
    }

    fn verify(&self, leaf: &D, hashes: Vec<H::Hash>) -> Result<(), String> {
        //
        todo!()
    }

    fn build_tree(hasher: &H, leaves: &[D]) -> Vec<H::Hash> {
        let mut tree: Vec<H::Hash> = vec![];

        let first_level = Self::build_first_level(hasher, leaves);
        let mut current_level = first_level;

        // Every level will have two times less nodes than the previous level.
        while current_level.len() > 2 {
            let level = Self::build_inner_level(hasher, &current_level);
            tree.append(&mut current_level);
            current_level = level;
        }

        // Append the root hash which was skiped by the while loop.
        tree.append(&mut current_level);

        tree
    }

    fn get_proof_hashes(&self, idx: usize) -> Result<Vec<H::Hash>, String> {
        let height = self.get_height();

        let mut hashes = Vec::default();
        let mut idx = idx;

        for _ in 0..height {
            let (s_hash, s_idx) = self.get_sibling(idx)?;
            let (_, p_idx) = self.get_parent(s_idx)?;

            hashes.push(s_hash);
            idx = p_idx;
        }

        Ok(hashes)
    }

    fn get_sibling(&self, idx: usize) -> Result<(H::Hash, usize), String> {
        let sibling_idx = if idx % 2 == 0 { idx + 1 } else { idx - 1 };
        let hash = self
            .get_tree()
            .get(sibling_idx)
            .ok_or_else(|| "invalid index".to_string())?
            .clone();

        Ok((hash, sibling_idx))
    }

    fn get_parent(&self, idx: usize) -> Result<(H::Hash, usize), String> {
        let tree = self.get_tree();
        let node_count = tree.len();
        let parent_idx = node_count - (node_count - idx - 1 + idx % 2) / 2;

        let hash = tree
            .get(parent_idx)
            .ok_or_else(|| "invalid index".to_string())?
            .clone();

        Ok((hash, parent_idx))
    }

    fn get_height(&self) -> usize {
        let leaves = self.get_leaf_count();
        let height = (leaves as f32).log2() + 1.;
        height as usize
    }

    fn get_leaf_count(&self) -> usize {
        let node_count = self.get_tree().len();
        (node_count + 1) / 2
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

mod tests {
    #[test]
    #[ignore]
    fn get_correct_sibling() {
        use super::*;

        let mut tree = Vec::new();
        for i in 0..7 {
            tree.push(Hash([i; 32]));
        }

        let file_tree = ChunkMerkleTree { tree };

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
