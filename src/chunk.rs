use std::convert::{TryFrom, TryInto};
use crate::chunk_type::ChunkType;
use crate::Error;
use crc::{Crc, CRC_32_ISO_HDLC};

pub const PNG_CRC: Crc<u32> = Crc::<u32>::new(&CRC_32_ISO_HDLC);

#[derive(Debug, Clone)]
pub struct Chunk {
    chunk_type: ChunkType,
    data: Vec<u8>,
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

    pub fn data(&self) -> &[u8] {
        &self.data
    }

    pub fn crc(&self) -> u32 {
        let bytes: Vec<u8> = self.chunk_type.bytes().iter()
            .chain(self.data.iter())
            .copied()
            .collect();
        PNG_CRC.checksum(&bytes)
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.length().to_be_bytes());
        bytes.extend_from_slice(&self.chunk_type.bytes());
        bytes.extend_from_slice(&self.data);
        bytes.extend_from_slice(&self.crc().to_be_bytes());
        bytes
    }
}

impl TryFrom<&[u8]> for Chunk {
    type Error = Error;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        if bytes.len() < 12 {
            return Err(Error::InvalidChunkType); 
        }

        let length = u32::from_be_bytes(bytes[0..4].try_into().map_err(|_| Error::InvalidChunkType)?);
        let type_bytes: [u8; 4] = bytes[4..8].try_into().map_err(|_| Error::InvalidChunkType)?;
        let chunk_type = ChunkType::try_from(type_bytes).map_err(|_| Error::InvalidChunkType)?;
        
        let data_end = 8 + length as usize;
        if bytes.len() < data_end + 4 {
            return Err(Error::InvalidChunkType);
        }

        let data = bytes[8..data_end].to_vec();
        
        let crc = u32::from_be_bytes(bytes[data_end..data_end+4].try_into().map_err(|_| Error::InvalidChunkType)?);
        let chunk = Self::new(chunk_type, data);
        
        if chunk.crc() != crc {
        }

        Ok(chunk)
    }
}