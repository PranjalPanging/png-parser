use std::convert::TryFrom;
use std::fmt;

use crate::chunk::Chunk;
use crate::error::{Error, Result};


pub struct Png {
    chunks: Vec<Chunk>,
}

impl Png {

    pub const SIGNATURE: [u8; 8] = [137, 80, 78, 71, 13, 10, 26, 10];
    pub fn new(chunks: Vec<Chunk>) -> Self {
        Self { chunks }
    }
    pub fn chunks(&self) -> &[Chunk] {
        &self.chunks
    }

    pub fn chunks_mut(&mut self) -> &mut Vec<Chunk> {
        &mut self.chunks
    }

    pub fn find_chunk(&self, type_str: &str) -> Option<&Chunk> {
        self.chunks
            .iter()
            .find(|c| c.chunk_type_str() == type_str)
    }

    pub fn find_chunk_mut(&mut self, type_str: &str) -> Option<&mut Chunk> {
        self.chunks
            .iter_mut()
            .find(|c| c.chunk_type_str() == type_str)
    }

    pub fn has_payload(&self) -> bool {
        self.find_chunk("stEg").is_some()
    }

    pub fn chunk_count_by_type(&self, type_str: &str) -> usize {
        self.chunks
            .iter()
            .filter(|c| c.chunk_type_str() == type_str)
            .count()
    }

    pub fn insert_before_iend(&mut self, chunk: Chunk) {
        let iend_pos = self
            .chunks
            .iter()
            .rposition(|c| c.chunk_type_str() == "IEND");

        match iend_pos {
            Some(pos) => self.chunks.insert(pos, chunk),
            None      => self.chunks.push(chunk),
        }
    }

    pub fn append_chunk(&mut self, chunk: Chunk) {
        self.chunks.push(chunk);
    }


    pub fn remove_chunk(&mut self, type_str: &str) {
        self.chunks.retain(|c| c.chunk_type_str() != type_str);
    }

    pub fn remove_chunk_secure(&mut self, type_str: &str) {
        for chunk in self.chunks.iter_mut() {
            if chunk.chunk_type_str() == type_str {
                chunk.zero_data();
            }
        }
        self.remove_chunk(type_str);
    }


    pub fn as_bytes(&self) -> Vec<u8> {
        let mut bytes = Self::SIGNATURE.to_vec();
        for chunk in &self.chunks {
            bytes.extend_from_slice(&chunk.as_bytes());
        }
        bytes
    }
    pub fn summary(&self) -> Vec<(String, u32)> {
        self.chunks
            .iter()
            .map(|c| (c.chunk_type_str().to_string(), c.length()))
            .collect()
    }

    pub fn byte_size(&self) -> usize {
        Self::SIGNATURE.len()
            + self
                .chunks
                .iter()
                .map(|c| 12 + c.length() as usize)
                .sum::<usize>()
    }
}


impl TryFrom<&[u8]> for Png {
    type Error = Error;

    fn try_from(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < 8 {
            return Err(Error::InvalidSignature);
        }
        if &bytes[..8] != Self::SIGNATURE {
            return Err(Error::InvalidSignature);
        }

        let mut chunks = Vec::new();
        let mut pos    = 8;

        while pos < bytes.len() {
            if bytes.len() - pos < 12 {
                break;
            }

            let length = u32::from_be_bytes(
                bytes[pos..pos + 4]
                    .try_into()
                    .map_err(|_| Error::TruncatedChunk)?,
            ) as usize;

            let chunk_end = pos + 12 + length;

            if chunk_end > bytes.len() {
                return Err(Error::TruncatedChunk);
            }

            let chunk = Chunk::try_from(&bytes[pos..chunk_end])?;
            let is_iend = chunk.chunk_type_str() == "IEND";
            chunks.push(chunk);
            pos = chunk_end;

            if is_iend {
                break;
            }
        }

        if chunks.is_empty() {
            return Err(Error::InvalidSignature);
        }

        let first = chunks.first().unwrap().chunk_type_str();
        if first != "IHDR" {
            return Err(Error::InvalidChunkType(
                "first chunk must be IHDR".to_string(),
            ));
        }

        Ok(Png { chunks })
    }
}


impl fmt::Display for Png {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "PNG ({} chunks, {} bytes total)",
            self.chunks.len(),
            self.byte_size()
        )?;
        for chunk in &self.chunks {
            writeln!(f, "  {}", chunk)?;
        }
        Ok(())
    }
}