pub mod chunk;
pub mod chunk_type;
pub mod png;
pub mod commands;

pub use chunk::Chunk;
pub use chunk_type::ChunkType;
pub use png::Png;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Invalid chunk type")]
    InvalidChunkType,
    #[error("Invalid PNG signature")]
    InvalidSignature,
    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, Error>;