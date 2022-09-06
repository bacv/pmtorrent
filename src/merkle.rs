use std::fmt::{self, Debug};

pub trait Hasher {
    type Hash: Debug;

    fn digest(&self, data: &[u8]) -> Self::Hash;
}

pub trait AsBytes {
    fn as_bytes(&self) -> &[u8];
}

pub trait MerkleTree<D, H>
where
    D: AsBytes,
    H: Hasher,
    H::Hash: AsBytes + Default + Clone,
{
    fn get_tree(&self) -> &[H::Hash];

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

    fn root_from_partial(hasher: &H, leaf: &D, hashes: Vec<H::Hash>) -> Result<H::Hash, String> {
        let leaf_hash = hasher.digest(leaf.as_bytes());
        let mut root_hash = hasher.digest(&[leaf_hash.as_bytes(), hashes[0].as_bytes()].concat());

        for h in hashes[1..].iter() {
            root_hash = hasher.digest(&[root_hash.as_bytes(), h.as_bytes()].concat());
        }

        Ok(root_hash)
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

#[derive(Clone, Default)]
struct EmojiHash {
    hash: [u8; 4],
}

impl EmojiHash {
    pub unsafe fn emoji(&self) -> char {
        let u_32 = u32::from_be_bytes(self.hash);

        char::from_u32_unchecked(u_32)
    }
}

impl fmt::Debug for EmojiHash {
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
        // using 👂 as base because it has 182 sequential emojis.
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

#[derive(Debug)]
struct DummyMerkleTree {
    tree: Vec<EmojiHash>,
}

impl DummyMerkleTree {
    pub fn new(leaves: &[&'static str]) -> Self {
        let hasher = EmojiHasher;
        DummyMerkleTree {
            tree: Self::build_tree(&hasher, leaves),
        }
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
        let leaves = &["hi", "this", "is", "a"];
        let dummy = DummyMerkleTree::new(leaves);
        println!("{:?}", dummy)
    }

    #[test]
    fn test_get_sibling() {}

    #[test]
    fn test_get_parent() {}

    #[test]
    fn test_root_from_partial() {}

    #[test]
    #[ignore]
    fn emoji_sanity_test() {
        let hasher = EmojiHasher;
        for i in 0..100 {
            let hash = hasher.digest(format!("{}{}{}{}{}", i, i, i, i, i).as_bytes());
            unsafe {
                println!("{:?}", hash);
                println!("{}", hash.emoji());
            }
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
