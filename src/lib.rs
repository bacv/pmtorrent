use merkle::AsBytes;

mod file;
mod merkle;

type Data = [u8; 1024];

#[derive(Clone)]
pub struct Chunk {
    pub data: Data,
    pub leaf_idx: usize,
}

impl AsBytes for Chunk {
    fn as_bytes(&self) -> &[u8] {
        &self.data
    }
}

#[derive(Clone, Debug, Default)]
pub struct Hash([u8; 32]);

impl AsBytes for Hash {
    fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}
