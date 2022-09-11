use std::fmt::Debug;

use crate::{AsBytes, Hasher};

/// An error that represents failure during merkle tree creation or when performing operation on it.
#[derive(Debug, PartialEq, Eq)]
pub enum MerkleError {
    /// An error representing wrong number of leaf nodes provided when the tree is being
    /// constructed.
    LeafCount,

    /// An error indicating that the incorrect node id was provided to the function.
    InvalidIdx,
}

/// MerkleTree is a trait that defines basic functions on a merkle tree and provides default
/// implementations for those functions.
///
/// # Examples:
/// To implement MerkleTree trait in a most basic form the [`MerkleTree::get_tree`] method needs to be provided:
///
/// ```
/// use pmtorrent::{EmojiHasher, EmojiHash, MerkleTree};
///
/// struct DummyMerkleTree {
///    tree: Vec<EmojiHash>,
/// }
///
/// impl MerkleTree<&'static str, EmojiHasher> for DummyMerkleTree {
///    fn get_tree(&self) -> &[EmojiHash] {
///        &self.tree
///    }
/// }
/// ```
/// For more in depth example please see `DummyMerkleTree` implementation in `./dummy.rs` file.
///
/// # Data layout.
///
/// The returned `Vec<H::Hash>` has a data layout of a reversed Heap.
/// First level (aka leaves) will be in a numerical order as the first elements of a Vec.
///
/// E.g. If a slice of leaves are provided with `[l1, l2, l3, l4]` as items, the returned tree will
/// have such layout: *`[h_l1, h_l2, h_l3, h_l4, h_p_l12, h_p_l34, h_root]`*.
pub trait MerkleTree<D, H>
where
    D: AsBytes,
    H: Hasher,
    H::Hash: AsBytes + Default + Clone,
{
    /// A method that provides an already build merkle tree.
    ///
    /// # Examples:
    /// ```
    /// use pmtorrent::{EmojiHasher, EmojiHash, MerkleTree};
    ///
    /// struct DummyMerkleTree {
    ///    tree: Vec<EmojiHash>,
    /// }
    ///
    /// impl MerkleTree<&'static str, EmojiHasher> for DummyMerkleTree {
    ///     fn get_tree(&self) -> &[EmojiHash] {
    ///         &self.tree
    ///     }
    /// }
    /// ```
    fn get_tree(&self) -> &[H::Hash];

    /// Builds a tree with a provided hasher and the first level of nodes (aka leaves).
    ///
    /// To have a custom builder one can implement [`MerkleTree::build_first_level`] and/or
    /// [`MerkleTree::build_inner_level`] methods with the custom functionality, such as different
    /// hash prefexes for different levels.
    ///
    /// # Data layout.
    ///
    /// The returned `Vec<H::Hash>` has a data layout of a reversed Heap.
    /// First level (aka leaves) will be in a numerical order as the first elements of a Vec.
    ///
    /// E.g. If a slice of leaves are provided with `[l1, l2, l3, l4]` as items, the returned tree will
    /// have such layout: `[h_l1, h_l2, h_l3, h_l4, h_p_l12, h_p_l34, h_root]`.
    fn build_tree(hasher: &H, leaves: &[D]) -> Result<Vec<H::Hash>, MerkleError> {
        let mut tree: Vec<H::Hash> = vec![];

        let first_level = Self::build_first_level(hasher, leaves)?;
        let mut current_level = first_level;

        // Every level will have two times less nodes than the previous level.
        // Checking if we are not in a top level which has only root hash.
        while current_level.len() > 1 {
            let level = Self::build_inner_level(hasher, &current_level)?;
            tree.append(&mut current_level);
            current_level = level;
        }

        // Append the root hash which was skiped by the while loop.
        tree.append(&mut current_level);

        Ok(tree)
    }

    /// Default implementation for MerkleTree to build first level from a nodes that can be hashed.
    ///
    /// This method checks if the leaf count is a number that is in power of two. If this criteria
    /// is not met, then `MerkleError::LeafCount` is returned.
    fn build_first_level(hasher: &H, leaves: &[D]) -> Result<Vec<H::Hash>, MerkleError> {
        if !is_pow_of_two(leaves.len()) {
            return Err(MerkleError::LeafCount);
        }

        Ok(leaves
            .iter()
            .map(|l| hasher.digest(l.as_bytes()))
            .collect::<Vec<H::Hash>>())
    }

    /// Default implementation for MerkleTree to build inner level from a nodes that can be hashed.
    ///
    /// This method checks if the leaf count is a number that is in power of two. If this criteria
    /// is not met, then `MerkleError::LeafCount` is returned.
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

    /// Provides a minimal set of hashes for a leaf node at provided idx that are needed to
    /// calculate the hash of a root node.
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

    /// A method that is used by the default implementation of MerkleTree to retrieve a sibling of a
    /// node at the provided idx.
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

    /// A method that is used by the default implementation of MerkleTree to retrieve a parent of a
    /// node at the provided idx.
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

    /// A helper method for the default implementation of MerkleTree that returns a level count for
    /// a tree that is retrieved via `get_tree` method.
    fn get_height(&self) -> usize {
        let leaves = self.get_leaf_count();
        let height = (leaves as f32).log2() + 1.;
        height as usize
    }

    /// A method that uses formula of a perfect complete binary for a leaf count retrieval.
    fn get_leaf_count(&self) -> usize {
        let node_count = self.get_tree().len();
        (node_count + 1) / 2
    }
}

/// A method for calculating root hash from the partial data unit and related list of proof hashes
/// that were calculated via the `get_proof_hashes` method.
///
/// This method can be wrapped inside a custom `root_from_partial` implementation that modifies the
/// original data to meet the application specification.
///
/// # Examples:
/// ```
/// use pmtorrent::{EmojiHasher, EmojiHash, merkle, MerkleError};
///
/// pub fn root_from_partial(
///     hasher: &EmojiHasher,
///     leaf: &EmojiHash,
///     leaf_idx: usize,
///     leaf_count: usize,
///     hashes: Vec<EmojiHash>,
/// ) -> Result<EmojiHash, MerkleError> {
///     let padded_leaf: EmojiHash = if true {
///         // do something with a leaf.
///         todo!()
///     } else {
///         // or not.
///         todo!()
///     };
///
///     merkle::root_from_partial(hasher, &padded_leaf, leaf_idx, leaf_count, hashes)
/// }
/// ```
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

/// Returns true if a number is 2^x.
pub fn is_pow_of_two(l: usize) -> bool {
    l > 0 && (l & (l - 1)) == 0
}
