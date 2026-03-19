use serde::{Deserialize, Serialize};

pub const HEADER_VERSION: u8 = 3;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Expiry {
    Never,
    At(i64),
}

impl Expiry {

    pub fn from_parts(
        days: Option<i64>,
        hours: Option<i64>,
        minutes: Option<i64>,
        seconds: Option<i64>,
    ) -> Self {
        let total_secs = days.unwrap_or(0) * 86_400
            + hours.unwrap_or(0) * 3_600
            + minutes.unwrap_or(0) * 60
            + seconds.unwrap_or(0);

        if total_secs <= 0 {
            return Expiry::Never;
        }

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("System clock before Unix epoch")
            .as_secs() as i64;

        Expiry::At(now + total_secs)
    }

    pub fn is_expired(&self) -> bool {
        match self {
            Expiry::Never => false,
            Expiry::At(ts) => {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .expect("System clock before Unix epoch")
                    .as_secs() as i64;
                now > *ts
            }
        }
    }

    pub fn to_display(&self) -> String {
        match self {
            Expiry::Never => "permanent".to_string(),
            Expiry::At(ts) => format!("unix:{}", ts),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum EmbedMode {
    Chunk,
    Pixel,
}

impl EmbedMode {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "chunk" => Some(EmbedMode::Chunk),
            "pixel" => Some(EmbedMode::Pixel),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            EmbedMode::Chunk => "chunk",
            EmbedMode::Pixel => "pixel",
        }
    }
}
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ShardInfo {
    pub index: u32,
    pub total: u32,
    pub file_hash: [u8; 32],
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PayloadHeader {
    pub version: u8,
    pub filename: String,
    pub file_size: u64,
    pub mode: EmbedMode,
    pub expiry: Expiry,
    pub shard: Option<ShardInfo>,
}

impl PayloadHeader {
    pub fn new(
        filename: String,
        file_size: u64,
        mode: EmbedMode,
        expiry: Expiry,
    ) -> Self {
        Self {
            version: HEADER_VERSION,
            filename,
            file_size,
            mode,
            expiry,
            shard: None,
        }
    }

    pub fn new_shard(
        filename: String,
        file_size: u64,
        mode: EmbedMode,
        expiry: Expiry,
        shard: ShardInfo,
    ) -> Self {
        Self {
            version: HEADER_VERSION,
            filename,
            file_size,
            mode,
            expiry,
            shard: Some(shard),
        }
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, bincode::Error> {
        bincode::serialize(self)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, bincode::Error> {
        bincode::deserialize(bytes)
    }
    pub fn pack(&self, file_bytes: &[u8]) -> Result<Vec<u8>, bincode::Error> {
        let header_bytes = self.to_bytes()?;
        let mut blob = (header_bytes.len() as u32).to_be_bytes().to_vec();
        blob.extend_from_slice(&header_bytes);
        blob.extend_from_slice(file_bytes);
        Ok(blob)
    }

    pub fn unpack(blob: &[u8]) -> crate::error::Result<(Self, &[u8])> {
    if blob.len() < 4 {
        return Err(crate::error::Error::CorruptHeader);
    }
    let header_len = u32::from_be_bytes(
        blob[..4].try_into().map_err(|_| crate::error::Error::CorruptHeader)?
    ) as usize;
    let header_end = 4 + header_len;
    if blob.len() < header_end {
        return Err(crate::error::Error::CorruptHeader);
    }
    let header = Self::from_bytes(&blob[4..header_end])
        .map_err(|e| crate::error::Error::Serialisation(e.to_string()))?;
    let file_bytes = &blob[header_end..];
    Ok((header, file_bytes))
}
}