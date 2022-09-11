use crate::{hasher::EmojiHash, EmojiHasher, MerkleError, MerkleTree};

#[derive(Debug, PartialEq, Eq)]
struct DummyMerkleTree {
    tree: Vec<EmojiHash>,
}

/// Dummy implementation of MerkleTree for tests.
impl DummyMerkleTree {
    #[allow(dead_code)]
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
    #[test]
    fn test_build_tree() {
        use super::*;

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
        use super::*;

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
        use super::*;

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
        use super::*;
        use crate::merkle::root_from_partial;

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
        use super::*;

        let leaves: Vec<&str> = "💁 💂 💃 💄 💅 💆 👏".split(' ').into_iter().collect();
        let dummy_tree = DummyMerkleTree::new(&leaves);
        assert_eq!(dummy_tree, Err(MerkleError::LeafCount));
    }
}
