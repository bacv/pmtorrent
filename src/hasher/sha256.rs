use ring::digest;
use serde::Serializer;

use crate::{encode_hex, AsBytes, Hasher};

/// A hasher that hashes provided data with Sha256 algorithm.
pub struct Sha256Hasher;

impl Hasher for Sha256Hasher {
    type Hash = Sha256Hash;

    fn digest(&self, data: &[u8]) -> Sha256Hash {
        let h = digest::digest(&digest::SHA256, data);
        Sha256Hash(h.as_ref().try_into().expect("32 byte value"))
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct Sha256Hash([u8; 32]);

impl Sha256Hash {
    pub fn new(d: [u8; 32]) -> Self {
        Self(d)
    }
}

impl AsBytes for Sha256Hash {
    fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

impl serde::Serialize for Sha256Hash {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        s.serialize_str(&encode_hex(&self.0))
    }
}
