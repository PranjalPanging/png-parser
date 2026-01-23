use crate::chunk::Chunk;
use crate::Error;
use std::convert::TryFrom;

pub struct Png {
    chunks: Vec<Chunk>,
}

impl Png {
    pub const STANDARD_HEADER: [u8; 8] = [137, 80, 78, 71, 13, 10, 26, 10];

    pub fn new(chunks: Vec<Chunk>) -> Self {
        Self { chunks }
    }

    pub fn chunks(&self) -> &[Chunk] {
        &self.chunks
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let mut bytes = Self::STANDARD_HEADER.to_vec();
        for chunk in &self.chunks {
            bytes.extend_from_slice(&chunk.as_bytes());
        }
        bytes
    }

    pub fn append_chunk(&mut self, chunk: Chunk) {
        if !self.chunks.is_empty() {
            self.chunks.insert(self.chunks.len() - 1, chunk);
        } else {
            self.chunks.push(chunk);
        }
    }

    pub fn remove_chunk(&mut self, chunk_type: &str) {
        self.chunks.retain(|c| c.chunk_type().to_string() != chunk_type);
    }
}

impl TryFrom<&[u8]> for Png {
    type Error = Error;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        if bytes.len() < 8 || &bytes[0..8] != Png::STANDARD_HEADER {
            return Err(Error::InvalidSignature);
        }

        let mut chunks = Vec::new();
        let mut pos = 8;

        while pos < bytes.len() {
            if pos + 4 > bytes.len() { break; }
            let length = u32::from_be_bytes(bytes[pos..pos+4].try_into().unwrap());
            let chunk_end = pos + 12 + length as usize;
            
            if chunk_end > bytes.len() { break; }
            let chunk = Chunk::try_from(&bytes[pos..chunk_end])?;
            chunks.push(chunk);
            pos = chunk_end;
        }

        Ok(Png { chunks })
    }
}