use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Bitstream error: {0}")]
    BitstreamError(#[from] crate::bitstream::BitstreamError),
    #[error("Encode error: {0}")]
    EncodeError(#[from] crate::encoding::EncodeError),
    #[error("Invalid version: {0}")]
    InvalidVersion(usize),
    #[error("Invalid mask: {0}")]
    InvalidMask(usize),
    #[error("Layout error: {0}")]
    LayoutError(#[from] crate::layout::LayoutError),
    #[error("rsec error: {0}")]
    RsecError(#[from] crate::rsec::RsecError),
}

pub type Result<T> = std::result::Result<T, Error>;
