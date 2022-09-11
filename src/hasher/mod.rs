mod emoji;
mod sha256;

pub use emoji::*;
pub use sha256::*;

pub trait Hasher {
    type Hash;

    fn digest(&self, data: &[u8]) -> Self::Hash;
}
