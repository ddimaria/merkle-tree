use thiserror::Error;

#[derive(Error, Debug)]
pub enum MerkleTreeError {
    #[error("Cannot find leaf: {0}")]
    CannotFindLeaf(String),

    #[error("Cannot initialize with zero leaves")]
    Empty,

    #[error("Offset {0} out of bounds (leaf length is {1}")]
    OffsetOutOfBounds(usize, usize),
}

/// Utility result type to be used throughout
pub type Result<T> = std::result::Result<T, MerkleTreeError>;
