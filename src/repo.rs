use serde::Serialize;
use std::collections::HashMap;

use crate::{
    encode_hex,
    file::{File, FileError},
    AsBytes, Chunk, MerkleError, Sha256Hash,
};

#[derive(Debug)]
pub enum RepoError {
    DoesntExist,
    File(FileError),
}

#[derive(Serialize)]
pub struct FileDescription {
    hash: String,
    pieces: usize,
}

impl From<FileError> for RepoError {
    fn from(e: FileError) -> Self {
        match e {
            FileError::Merkle(MerkleError::InvalidIdx) => RepoError::DoesntExist,
            _ => RepoError::File(e),
        }
    }
}

#[derive(Serialize, Clone, Debug)]
pub struct Piece {
    pub content: Chunk,
    pub proof: Vec<Sha256Hash>,
}

#[derive(Default)]
pub struct FileRepo {
    files: HashMap<String, File>,
}

impl FileRepo {
    pub fn add(&mut self, file: File) -> Result<(), RepoError> {
        let hash = encode_hex(file.get_root()?.as_bytes());
        self.files.insert(hash, file);
        Ok(())
    }

    pub fn get_available(&self) -> Vec<FileDescription> {
        self.files
            .iter()
            .map(|(h, f)| FileDescription {
                hash: h.clone(),
                pieces: f.get_size(),
            })
            .collect()
    }

    pub fn get_piece(&self, hash: String, piece: usize) -> Result<Piece, RepoError> {
        let file = self.files.get(&hash).ok_or(RepoError::DoesntExist)?;
        let (content, proof) = file.get_chunk(piece)?;
        Ok(Piece { content, proof })
    }
}
