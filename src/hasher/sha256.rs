use ring::digest;
use serde::Serializer;

use crate::{encode_hex, AsBytes, Hasher};

pub struct Sha256Hasher;

impl Hasher for Sha256Hasher {
    type Hash = Hash;

    fn digest(&self, data: &[u8]) -> Hash {
        let h = digest::digest(&digest::SHA256, data);
        Hash(h.as_ref().try_into().expect("32 byte value"))
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct Hash([u8; 32]);

impl Hash {
    pub fn new(d: [u8; 32]) -> Self {
        Self(d)
    }
}

impl AsBytes for Hash {
    fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

impl serde::Serialize for Hash {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        s.serialize_str(&encode_hex(&self.0))
    }
}
