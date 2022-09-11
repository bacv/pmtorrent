use serde::Serializer;

use crate::AsBytes;

#[derive(Clone, Debug)]
pub struct Chunk {
    pub data: Vec<u8>,
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

impl serde::Serialize for Chunk {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        s.serialize_str(&base64::encode(&self.data))
    }
}

impl AsBytes for Chunk {
    fn as_bytes(&self) -> &[u8] {
        &self.data
    }
}
