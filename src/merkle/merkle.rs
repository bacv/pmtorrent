use std::fmt::Debug;

use crate::{AsBytes, Hasher};

#[derive(Debug, PartialEq, Eq)]
pub enum MerkleError {
    LeafCount,
    InvalidIdx,
}

pub trait MerkleTree<D, H>
where
    D: AsBytes,
    H: Hasher,
    H::Hash: AsBytes + Default + Clone,
{
    fn get_tree(&self) -> &[H::Hash];

    fn build_tree(hasher: &H, leaves: &[D]) -> Result<Vec<H::Hash>, MerkleError> {
        let mut tree: Vec<H::Hash> = vec![];

        let first_level = Self::build_first_level(hasher, leaves)?;
        let mut current_level = first_level;

        // Every level will have two times less nodes than the previous level.
        while current_level.len() > 1 {
            let level = Self::build_inner_level(hasher, &current_level)?;
            tree.append(&mut current_level);
            current_level = level;
        }

        // Append the root hash which was skiped by the while loop.
        tree.append(&mut current_level);

        Ok(tree)
    }

    fn build_first_level(hasher: &H, leaves: &[D]) -> Result<Vec<H::Hash>, MerkleError> {
        if !is_pow_of_two(leaves.len()) {
            return Err(MerkleError::LeafCount);
        }

        Ok(leaves
            .iter()
            .map(|l| hasher.digest(l.as_bytes()))
            .collect::<Vec<H::Hash>>())
    }

    fn build_inner_level(
        hasher: &H,
        previous_level: &[H::Hash],
    ) -> Result<Vec<H::Hash>, MerkleError> {
        if !is_pow_of_two(previous_level.len()) {
            return Err(MerkleError::LeafCount);
        }

        Ok(previous_level
            .chunks(2)
            .map(|c| {
                let l = &c[0];
                let r = &c[1];
                hasher.digest(&[l.as_bytes(), r.as_bytes()].concat())
            })
            .collect::<Vec<H::Hash>>())
    }

    fn root_from_partial(
        hasher: &H,
        leaf: &D,
        leaf_idx: usize,
        leaf_count: usize,
        hashes: Vec<H::Hash>,
    ) -> Result<H::Hash, MerkleError> {
        let node_count = leaf_count * 2 - 1;

        let mut l = &hasher.digest(leaf.as_bytes());
        let mut r = &hashes[0];

        if leaf_idx % 2 != 0 {
            std::mem::swap(&mut l, &mut r)
        }

        let mut root_hash = hasher.digest(&[l.as_bytes(), r.as_bytes()].concat());
        let mut parent_idx = node_count - (node_count - leaf_idx - 1 + leaf_idx % 2) / 2;

        for h in hashes[1..].iter() {
            let mut l = &root_hash;
            let mut r = h;
            if parent_idx % 2 != 0 {
                std::mem::swap(&mut l, &mut r);
            }

            root_hash = hasher.digest(&[l.as_bytes(), r.as_bytes()].concat());
            if parent_idx + 2 >= node_count {
                parent_idx = node_count - (node_count - parent_idx - 1 + parent_idx % 2) / 2;
            }
        }

        Ok(root_hash)
    }

    fn get_proof_hashes(&self, idx: usize) -> Result<Vec<H::Hash>, MerkleError> {
        let height = self.get_height();

        let mut hashes = Vec::default();
        let mut idx = idx;

        for _ in 0..height - 1 {
            let (s_hash, s_idx) = self.get_sibling(idx)?;
            let (_, p_idx) = self.get_parent(s_idx)?;

            hashes.push(s_hash);
            idx = p_idx;
        }

        Ok(hashes)
    }

    fn get_sibling(&self, idx: usize) -> Result<(H::Hash, usize), MerkleError> {
        if idx >= self.get_tree().len() {
            return Err(MerkleError::InvalidIdx);
        }

        let sibling_idx = if idx % 2 == 0 { idx + 1 } else { idx - 1 };
        let hash = self
            .get_tree()
            .get(sibling_idx)
            .ok_or(MerkleError::InvalidIdx)?
            .clone();

        Ok((hash, sibling_idx))
    }

    fn get_parent(&self, idx: usize) -> Result<(H::Hash, usize), MerkleError> {
        if idx >= self.get_tree().len() {
            return Err(MerkleError::InvalidIdx);
        }

        let tree = self.get_tree();
        let node_count = tree.len();
        let parent_idx = node_count - (node_count - idx - 1 + idx % 2) / 2;

        let hash = tree.get(parent_idx).ok_or(MerkleError::InvalidIdx)?.clone();

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

pub fn root_from_partial<D, H>(
    hasher: &H,
    leaf: &D,
    leaf_idx: usize,
    leaf_count: usize,
    hashes: Vec<H::Hash>,
) -> Result<H::Hash, MerkleError>
where
    D: AsBytes,
    H: Hasher,
    H::Hash: AsBytes,
{
    let node_count = leaf_count * 2 - 1;

    let mut l = &hasher.digest(leaf.as_bytes());
    let mut r = &hashes[0];

    if leaf_idx % 2 != 0 {
        std::mem::swap(&mut l, &mut r)
    }

    let mut root_hash = hasher.digest(&[l.as_bytes(), r.as_bytes()].concat());
    let mut parent_idx = node_count - (node_count - leaf_idx - 1 + leaf_idx % 2) / 2;

    for h in hashes[1..].iter() {
        let mut l = &root_hash;
        let mut r = h;
        if parent_idx % 2 != 0 {
            std::mem::swap(&mut l, &mut r);
        }

        root_hash = hasher.digest(&[l.as_bytes(), r.as_bytes()].concat());
        if parent_idx + 2 >= node_count {
            parent_idx = node_count - (node_count - parent_idx - 1 + parent_idx % 2) / 2;
        }
    }

    Ok(root_hash)
}

pub fn is_pow_of_two(l: usize) -> bool {
    l > 0 && (l & (l - 1)) == 0
}
