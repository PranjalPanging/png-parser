use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid PNG signature — not a valid PNG file")]
    InvalidSignature,

    #[error("Invalid chunk type '{0}' — must be 4 ASCII letters")]
    InvalidChunkType(String),

    #[error("CRC mismatch in chunk '{chunk_type}' — expected {expected:#010x}, got {actual:#010x}")]
    CrcMismatch {
        chunk_type: String,
        expected: u32,
        actual: u32,
    },

    #[error("Truncated chunk — file may be corrupted")]
    TruncatedChunk,

    #[error("No stEg payload found in this image")]
    NoPayload,

    #[error("Unsupported image format '{0}' — use PNG, BMP, TIFF, or WebP")]
    UnsupportedFormat(String),

    #[error("JPEG is lossy and destroys embedded data — use PNG, BMP, TIFF, or WebP")]
    LossyFormat,

    #[error("Cannot detect image format — check file extension and magic bytes")]
    UnknownFormat,

    #[error("Encryption failed")]
    EncryptionFailed,

    #[error("Decryption failed — wrong password or corrupted payload")]
    DecryptionFailed,

    #[error("Key derivation failed: {0}")]
    KeyDerivation(String),

    #[error("This payload is encrypted — a password is required")]
    PasswordRequired,

    #[error("Wrong password — cannot decrypt payload")]
    WrongPassword,

    #[error("Payload header is corrupt or unreadable")]
    CorruptHeader,

    #[error(
        "Payload format version {found} is not supported \
         (this build supports up to version {supported})"
    )]
    UnsupportedVersion { found: u8, supported: u8 },

    #[error("Payload has expired")]
    Expired,
   
    #[error(
        "Payload too large: need {needed} bytes, \
         image provides {available} bytes"
    )]
    InsufficientCapacity { needed: usize, available: usize },

    #[error(
        "Not enough high-texture area to embed payload in pixel mode — \
         use a more detailed image or switch to chunk mode"
    )]
    InsufficientTexture,
 
    #[error("Password required to delete an encrypted payload")]
    DeletePasswordRequired,

    #[error("Wrong password — cannot authorise deletion")]
    DeleteWrongPassword,
   
    #[error("Compression failed: {0}")]
    CompressionFailed(String),

    #[error("Decompression failed: {0}")]
    DecompressionFailed(String),
   
    #[error("Shard {index} of {total} is missing — all shards required to merge")]
    MissingShard { index: u32, total: u32 },

    #[error("Shard file hash mismatch — shards are not from the same source file")]
    ShardHashMismatch,

    #[error("Expected {expected} shards but only {found} images provided")]
    ShardCountMismatch { expected: usize, found: usize },
   
    #[error("Serialisation error: {0}")]
    Serialisation(String),
}

impl From<bincode::Error> for Error {
    fn from(e: bincode::Error) -> Self {
        Error::Serialisation(e.to_string())
    }
}

impl From<image::ImageError> for Error {
    fn from(e: image::ImageError) -> Self {
        Error::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
    }
}


#[cfg(feature = "python")]
impl From<Error> for pyo3::PyErr {
    fn from(e: Error) -> pyo3::PyErr {
        use pyo3::exceptions::*;
        match &e {
            Error::Io(_) => PyIOError::new_err(e.to_string()),

            Error::WrongPassword
            | Error::DeleteWrongPassword
            | Error::Expired => PyPermissionError::new_err(e.to_string()),

            Error::InvalidSignature
            | Error::InvalidChunkType(_)
            | Error::CrcMismatch { .. }
            | Error::TruncatedChunk
            | Error::NoPayload
            | Error::UnsupportedFormat(_)
            | Error::LossyFormat
            | Error::UnknownFormat
            | Error::CorruptHeader
            | Error::UnsupportedVersion { .. }
            | Error::InsufficientCapacity { .. }
            | Error::InsufficientTexture
            | Error::ShardHashMismatch
            | Error::ShardCountMismatch { .. }
            | Error::MissingShard { .. } => PyValueError::new_err(e.to_string()),

            Error::EncryptionFailed
            | Error::DecryptionFailed
            | Error::KeyDerivation(_)
            | Error::CompressionFailed(_)
            | Error::DecompressionFailed(_)
            | Error::Serialisation(_) => PyRuntimeError::new_err(e.to_string()),

            Error::PasswordRequired
            | Error::DeletePasswordRequired => PyPermissionError::new_err(e.to_string()),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

