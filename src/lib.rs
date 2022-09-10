use merkle::AsBytes;

mod file;
mod merkle;

type Data = Vec<u8>;

#[derive(Clone)]
pub struct Chunk {
    pub data: Data,
    pub leaf_idx: usize,
}

impl Chunk {
    pub fn len(&self) -> usize {
        self.data.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl AsBytes for Chunk {
    fn as_bytes(&self) -> &[u8] {
        &self.data
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Hash([u8; 32]);

impl AsBytes for Hash {
    fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}
