use std::fmt::{self, Debug};

use crate::{AsBytes, Hasher};

/// 🖖 Emoji hash is a fun part of this project.
///
/// This type is mostly used for MerkleTree trait default implementation (the tests can be found in
/// `pmtorrent::merkle::dummy` module).
#[derive(Clone, Default, PartialEq, Eq)]
pub struct EmojiHash {
    hash: [u8; 4],
}

impl EmojiHash {
    /// Method to convert EmojiHash byte values to an emoji reporesentation.
    ///
    /// # Safety
    ///
    /// This method is marked as unsafe for more than one reason.
    /// * Even though it's possible to implement u32 conversion to a valid utf16 character with
    /// additional checks (and in this case the `EmojiHasher::digest` method will always return a
    /// valid utf16 char), this `unsafe` attribute is used to remind the caller that the whole
    /// EmojiHasher is just for fun, not for safety and reliability.
    /// * Some combinations of emoji hashes can appear offensive, use with caution.
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

pub struct EmojiHasher;
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
