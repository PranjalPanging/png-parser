use std::convert::TryFrom;
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChunkType {
    bytes: [u8; 4],
}

impl ChunkType {
    pub fn bytes(&self) -> [u8; 4] {
        self.bytes
    }

    pub fn is_critical(&self) -> bool {
        (self.bytes[0] & 32) == 0
    }

    pub fn is_valid(&self) -> bool {
        self.bytes.iter().all(|&b| b.is_ascii_alphabetic()) && self.is_reserved_bit_valid()
    }

    fn is_reserved_bit_valid(&self) -> bool {
        (self.bytes[2] & 32) == 0
    }
}

impl TryFrom<[u8; 4]> for ChunkType {
    type Error = &'static str;
    fn try_from(bytes: [u8; 4]) -> Result<Self, Self::Error> {
        Ok(ChunkType { bytes })
    }
}

impl FromStr for ChunkType {
    type Err = &'static str; 

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 4 {
            return Err("Chunk type must be 4 characters");
        }
        let bytes = s.as_bytes();
        if !bytes.iter().all(|&b| b.is_ascii_alphabetic()) {
            return Err("Chunk type must be alphabetic");
        }
        let mut array = [0u8; 4];
        array.copy_from_slice(bytes);
        Ok(ChunkType { bytes: array })
    }
}

impl fmt::Display for ChunkType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", String::from_utf8_lossy(&self.bytes))
    }
}