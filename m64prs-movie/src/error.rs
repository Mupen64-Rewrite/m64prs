use std::{error, io, sync::Arc};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("{filename} is an invalid .m64 file")]
    InvalidM64Error {
        filename: String
    },
    #[error("I/O error occurred reading from {filename}")]
    M64ReadError {
        filename: String,
        #[source] error: Arc<io::Error>
    },
    #[error("I/O error occurred writing to {filename}")]
    M64WriteError {
        filename: String,
        #[source] error: Arc<io::Error>
    },
    #[error("description field can only hold 256 bytes")]
    M64DescriptionTooLong,
    #[error("author field can only hold 222 bytes")]
    M64AuthorTooLong,
}