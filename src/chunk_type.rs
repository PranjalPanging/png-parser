use std::convert::TryFrom;
use std::fmt;
use std::str::FromStr;

use crate::error::{Error, Result};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ChunkType {
    bytes: [u8; 4],
}

impl ChunkType {

    pub fn bytes(&self) -> [u8; 4] {
        self.bytes
    }

    pub fn as_str(&self) -> &str {
        std::str::from_utf8(&self.bytes).unwrap()
    }
    pub fn is_critical(&self) -> bool {
        self.bytes[0].is_ascii_uppercase()
    }

    pub fn is_ancillary(&self) -> bool {
        !self.is_critical()
    }
    pub fn is_private(&self) -> bool {
        self.bytes[1].is_ascii_lowercase()
    }

    pub fn is_public(&self) -> bool {
        !self.is_private()
    }
    pub fn is_reserved_bit_valid(&self) -> bool {
        self.bytes[2].is_ascii_uppercase()
    }

    pub fn is_safe_to_copy(&self) -> bool {
        self.bytes[3].is_ascii_lowercase()
    }

    pub fn is_valid(&self) -> bool {
        self.bytes.iter().all(|b| b.is_ascii_alphabetic())
            && self.is_reserved_bit_valid()
    }

    pub fn validate(&self) -> Result<()> {
        for (i, &b) in self.bytes.iter().enumerate() {
            if !b.is_ascii_alphabetic() {
                return Err(Error::InvalidChunkType(format!(
                    "byte {} ('{:?}') is not ASCII alphabetic",
                    i, b as char,
                )));
            }
        }
        if !self.is_reserved_bit_valid() {
            return Err(Error::InvalidChunkType(format!(
                "byte 2 ('{}') must be uppercase (reserved bit)",
                self.bytes[2] as char,
            )));
        }
        Ok(())
    }

    pub fn steg() -> Self {
        Self { bytes: *b"stEg" }
    }

    pub fn ihdr() -> Self { Self { bytes: *b"IHDR" } }
    pub fn idat() -> Self { Self { bytes: *b"IDAT" } }
    pub fn iend() -> Self { Self { bytes: *b"IEND" } }
    pub fn plte() -> Self { Self { bytes: *b"PLTE" } }
}

impl TryFrom<[u8; 4]> for ChunkType {
    type Error = Error;

    fn try_from(bytes: [u8; 4]) -> Result<Self> {
        let ct = ChunkType { bytes };
        ct.validate()?;
        Ok(ct)
    }
}

impl FromStr for ChunkType {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        if s.len() != 4 {
            return Err(Error::InvalidChunkType(format!(
                "chunk type must be exactly 4 characters, got {}",
                s.len()
            )));
        }

        let mut bytes = [0u8; 4];
        bytes.copy_from_slice(s.as_bytes());

        let ct = ChunkType { bytes };
        ct.validate()?;
        Ok(ct)
    }
}

impl fmt::Display for ChunkType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}