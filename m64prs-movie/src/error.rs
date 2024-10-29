use std::str::Utf8Error;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum StringFieldError {
    #[error("UTF-8 validation of field failed")]
    Utf8Invalid(#[source] Utf8Error),
    #[error("Value is too long, max length is {max_len} bytes")]
    FieldTooLong {
        max_len: usize
    }
}