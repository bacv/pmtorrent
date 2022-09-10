use std::fmt::{self, Debug};

#[derive(Debug, PartialEq, Eq)]
pub enum MerkleError {
    LeafCount,
    InvalidIdx,
}

pub trait Hasher {
    type Hash;

    fn digest(&self, data: &[u8]) -> Self::Hash;
}

pub trait AsBytes {
    fn as_bytes(&self) -> &[u8];
}

pub fn is_pow_of_two(l: usize) -> bool {
    l > 0 && (l & (l - 1)) == 0
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

#[derive(Clone, Default, PartialEq, Eq)]
struct EmojiHash {
    hash: [u8; 4],
}

impl EmojiHash {
    pub unsafe fn emoji(&self) -> char {
        let u_32 = u32::from_be_bytes(self.hash);

        char::from_u32_unchecked(u_32)
    }
}

impl Debug for EmojiHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let emoji = unsafe { self.emoji() };
        write!(f, "{}", emoji)
    }
}

impl AsBytes for EmojiHash {
    fn as_bytes(&self) -> &[u8] {
        &self.hash
    }
}

struct EmojiHasher;
impl Hasher for EmojiHasher {
    type Hash = EmojiHash;

    fn digest(&self, data: &[u8]) -> Self::Hash {
        let mut hash = 0u8;
        let prime = 181;
        for v in data.iter() {
            let (res, _) = (hash % prime).overflowing_add(*v);
            hash = res;
        }

        hash %= prime;
        // using 👂 as a base because it has 182 sequential emojis.
        let emoji = '\u{1F442}' as u32 + hash as u32;

        EmojiHash {
            hash: emoji.to_be_bytes(),
        }
    }
}

impl AsBytes for &'static str {
    fn as_bytes(&self) -> &[u8] {
        str::as_bytes(self)
    }
}

#[derive(Debug, PartialEq, Eq)]
struct DummyMerkleTree {
    tree: Vec<EmojiHash>,
}

impl DummyMerkleTree {
    pub fn new(leaves: &[&'static str]) -> Result<Self, MerkleError> {
        let hasher = EmojiHasher;
        Ok(DummyMerkleTree {
            tree: Self::build_tree(&hasher, leaves)?,
        })
    }
}

impl MerkleTree<&'static str, EmojiHasher> for DummyMerkleTree {
    fn get_tree(&self) -> &[EmojiHash] {
        &self.tree
    }
}

mod tests {
    use super::*;

    #[test]
    fn test_build_tree() {
        let leaves = &["this", "is", "sparta", "!"];
        let expected_leaves_emojis = ['👅', '👩', '👣', '👣'];
        let expected_root_emoji = '👯';

        let dummy_tree = DummyMerkleTree::new(leaves).expect("valid count of nodes");
        let dummy_tree = dummy_tree.get_tree();

        let expected_len = leaves.len() * 2 - 1;
        assert_eq!(dummy_tree.len(), expected_len);

        for (i, _) in leaves.iter().enumerate() {
            let emoji = unsafe { dummy_tree[i].emoji() };
            assert_eq!(emoji, expected_leaves_emojis[i]);
        }

        let root = unsafe { dummy_tree[expected_len - 1].emoji() };
        assert_eq!(root, expected_root_emoji);
    }

    #[test]
    fn test_get_sibling() {
        let expected_tree = [
            '📉', '📨', '👡', '👢', '💣', '👋', '📰', '📱', '👳', '💅', '💰', '👘', '👯', '📊',
            '💰',
        ];

        let leaves: Vec<&str> = "another valid number of a first level nodes"
            .split(' ')
            .collect();

        let dummy_tree = DummyMerkleTree::new(&leaves).expect("valid count of nodes");

        let (hash, idx) = dummy_tree.get_sibling(0).unwrap(); // 📉
        assert_eq!(unsafe { hash.emoji() }, expected_tree[1]); // 📨
        assert_eq!(idx, 1);

        let (hash, idx) = dummy_tree.get_sibling(9).unwrap(); // 💅
        assert_eq!(unsafe { hash.emoji() }, expected_tree[8]); // 👳
        assert_eq!(idx, 8);

        let res = dummy_tree.get_sibling(14); // root has no siblings 🥺.
        assert!(res.is_err());

        let res = dummy_tree.get_sibling(15); // non existent node.
        assert!(res.is_err());
    }

    #[test]
    fn test_get_parent() {
        let expected_tree = [
            '👂', '💛', '💍', '💎', '💅', '💛', '💜', '👅', '📩', '💛', '💅', '💑', '💁', '👢',
            '👹', '👂', '👔', '📝', '📢', '💣', '👆', '📘', '💥', '📈', '💨', '👇', '💕', '👺',
            '💱', '📑', '👄',
        ];

        // A dark twist to the emoji merkle tree.
        let leaves: Vec<&str> = "なぜそんなに真剣なんだ? 🃏".split("").into_iter().collect();

        let dummy_tree = DummyMerkleTree::new(&leaves).expect("valid count of nodes");

        let (hash, idx) = dummy_tree.get_parent(0).unwrap(); // 👂
        assert_eq!(unsafe { hash.emoji() }, expected_tree[16]); // 👔
        assert_eq!(idx, 16);

        let (hash, idx) = dummy_tree.get_parent(16).unwrap(); // 👔
        assert_eq!(unsafe { hash.emoji() }, expected_tree[24]); // 💨
        assert_eq!(idx, 24);

        let res = dummy_tree.get_parent(31); // root has no parents 🥺.
        assert!(res.is_err());

        let res = dummy_tree.get_parent(32); // non existent node.
        assert!(res.is_err());
    }

    #[test]
    fn test_root_from_partial() {
        let leaves: Vec<&str> = "💁 💂 💃 💄 💅 💆 👏 📮".split(' ').into_iter().collect();
        let hasher = EmojiHasher;

        let dummy_tree = DummyMerkleTree::new(&leaves).expect("valid count of nodes");
        let trusted_root = dummy_tree.get_tree().last().unwrap();

        let proof_parts = dummy_tree.get_proof_hashes(6).unwrap();
        let untrusted_root =
            root_from_partial(&hasher, &leaves[6], 6, leaves.len(), proof_parts).unwrap();

        assert_eq!(*trusted_root, untrusted_root);
    }

    #[test]
    fn test_leaf_count() {
        let leaves: Vec<&str> = "💁 💂 💃 💄 💅 💆 👏".split(' ').into_iter().collect();
        let dummy_tree = DummyMerkleTree::new(&leaves);
        assert_eq!(dummy_tree, Err(MerkleError::LeafCount));
    }
}
