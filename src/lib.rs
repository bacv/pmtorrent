mod file;
mod merkle;

type Data = [u8; 1024];
type Proof = Vec<Hash>;

#[derive(Clone)]
pub struct Chunk {
    pub data: Data,
    pub leaf_idx: usize,
}

pub struct Hash(String);
