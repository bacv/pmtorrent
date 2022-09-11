use std::fmt::{self, Debug};

use crate::{AsBytes, Hasher};

#[derive(Clone, Default, PartialEq, Eq)]
pub struct EmojiHash {
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
