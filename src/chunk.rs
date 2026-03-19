use std::convert::{TryFrom, TryInto};
use std::fmt;

use crc::{Crc, CRC_32_ISO_HDLC};

use crate::chunk_type::ChunkType;
use crate::error::{Error, Result};

pub const PNG_CRC: Crc<u32> = Crc::<u32>::new(&CRC_32_ISO_HDLC);

#[derive(Debug, Clone)]
pub struct Chunk {
    chunk_type: ChunkType,
    data:       Vec<u8>,
}

impl Chunk {

    pub fn new(chunk_type: ChunkType, data: Vec<u8>) -> Self {
        Self { chunk_type, data }
    }

    pub fn length(&self) -> u32 {
        self.data.len() as u32
    }

    pub fn chunk_type(&self) -> &ChunkType {
        &self.chunk_type
    }

    pub fn chunk_type_str(&self) -> &str {
        self.chunk_type.as_str()
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }

    pub fn data_mut(&mut self) -> &mut Vec<u8> {
        &mut self.data
    }

    pub fn crc(&self) -> u32 {
        let mut digest = PNG_CRC.digest();
        digest.update(&self.chunk_type.bytes());
        digest.update(&self.data);
        digest.finalize()
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(12 + self.data.len());
        bytes.extend_from_slice(&self.length().to_be_bytes());
        bytes.extend_from_slice(&self.chunk_type.bytes());
        bytes.extend_from_slice(&self.data);
        bytes.extend_from_slice(&self.crc().to_be_bytes());
        bytes
    }

    pub fn zero_data(&mut self) {
        for b in self.data.iter_mut() {
            *b = 0;
        }
    }

    pub fn is_critical(&self) -> bool {
        self.chunk_type.is_critical()
    }

    pub fn is_steg(&self) -> bool {
        self.chunk_type_str() == "stEg"
    }
}

impl TryFrom<&[u8]> for Chunk {
    type Error = Error;

    fn try_from(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < 12 {
            return Err(Error::TruncatedChunk);
        }

        let length = u32::from_be_bytes(
            bytes[0..4]
                .try_into()
                .map_err(|_| Error::TruncatedChunk)?,
        ) as usize;

        let type_bytes: [u8; 4] = bytes[4..8]
            .try_into()
            .map_err(|_| Error::TruncatedChunk)?;

        let chunk_type = ChunkType::try_from(type_bytes)
            .map_err(|e| Error::InvalidChunkType(e.to_string()))?;

        let data_end = 8 + length;

        if bytes.len() < data_end + 4 {
            return Err(Error::TruncatedChunk);
        }

        let data = bytes[8..data_end].to_vec();

        let stored_crc = u32::from_be_bytes(
            bytes[data_end..data_end + 4]
                .try_into()
                .map_err(|_| Error::TruncatedChunk)?,
        );

        let chunk = Self::new(chunk_type, data);

        let computed_crc = chunk.crc();
        if computed_crc != stored_crc {
            return Err(Error::CrcMismatch {
                chunk_type: chunk.chunk_type_str().to_string(),
                expected:   stored_crc,
                actual:     computed_crc,
            });
        }

        Ok(chunk)
    }
}

impl fmt::Display for Chunk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Chunk {{ type: {}, length: {}, crc: {:#010x} }}",
            self.chunk_type_str(),
            self.length(),
            self.crc(),
        )
    }
}
