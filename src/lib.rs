use std::fmt::Write;

mod file;
mod hasher;
mod merkle;
mod repo;

pub use file::*;
pub use hasher::*;
pub use merkle::*;
pub use repo::*;

pub fn encode_hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        write!(&mut s, "{:02x}", b).unwrap();
    }
    s
}

pub trait AsBytes {
    fn as_bytes(&self) -> &[u8];
}
