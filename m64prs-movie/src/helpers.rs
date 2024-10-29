use core::str;
use std::{borrow::Borrow, fmt::Display, str::FromStr};

use m64prs_sys::Buttons;

use crate::error::StringFieldError;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StringField<const N: usize>([u8; N]);

impl<const N: usize> Default for StringField<N> {
    /// Returns an empty string field. Will not compile for `N == 0`.
    fn default() -> Self {
        assert!(N > 0, "Cannot read from an empty field");
        Self([0u8; N])
    }
}

impl<const N: usize> StringField<N> { 
    pub const MAX_LEN: usize = N;

    /// Returns a string field containing the specified string. Will not compile for `N == 0``.
    ///
    /// # Errors
    /// This function errors if the specified string is too long.
    pub fn new<T: Borrow<str>>(data: T) -> Result<Self, StringFieldError> {
        let mut result = StringField::default();
        result.try_write_utf8(data)?;
        Ok(result)
    }

    /// Tries to read a string from the string field.
    ///
    /// # Errors
    /// This function errors if the string field does not contain valid UTF-8.
    pub fn try_read_utf8(&self) -> Result<&str, StringFieldError> {
        assert!(N > 0, "Cannot read from an empty field");
        // Find the null-terminator
        let to_null_term = match self.0.iter().position(|x| *x == b'\0') {
            Some(pos) => &self.0[..pos],
            None => &self.0,
        };
        std::str::from_utf8(to_null_term).map_err(|err| StringFieldError::Utf8Invalid(err))
    }

    /// Tries to write a string to the string field.
    ///
    /// # Errors
    /// This function errors if the provided string is too long to fit in the string field.
    pub fn try_write_utf8<T: Borrow<str>>(&mut self, data: T) -> Result<(), StringFieldError> {
        assert!(N > 0, "Cannot write to an empty field");

        let data = data.borrow();
        let len = data.len();
        if len > N {
            Err(StringFieldError::FieldTooLong { max_len: N })
        } else if len == N {
            self.0.copy_from_slice(data.as_bytes());

            Ok(())
        } else {
            self.0[..len].copy_from_slice(data.as_bytes());
            self.0[len] = b'\0';
            Ok(())
        }
    }

    /// Reads from the string field.
    ///
    /// # Panics
    /// Panics if the string field doesn't contain valid UTF-8.
    pub fn read_utf8(&self) -> &str {
        self.try_read_utf8()
            .expect("StringField::try_read_utf8 failed")
    }

    /// Writes to the string field.
    ///
    /// # Panics
    /// Panics if the provided string is too long to fit.
    pub fn write_utf8<T: Borrow<str>>(&mut self, data: T) {
        self.try_write_utf8(data)
            .expect("StringField::try_write_utf8 failed")
    }

    /// Writes to the string field, truncating the string if it's too long.
    /// Returns the truncated string.
    pub fn write_utf8_clipped<'a>(&mut self, data: &'a str) -> &'a str {
        if data.len() <= N {
            self.try_write_utf8(data).unwrap();
            data
        } else {
            // work backwards and find the closest character boundary
            let split_pos = (0..=N).rev().find(|n| data.is_char_boundary(*n)).unwrap();
            let slice = &data[0..split_pos];
            self.try_write_utf8(slice).unwrap();
            slice
        }
    }
}

impl<const N: usize> Display for StringField<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.read_utf8())
    }
}

impl<const N: usize> FromStr for StringField<N> {
    type Err = StringFieldError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

pub(crate) fn fix_buttons_order(_inputs: &mut [Buttons]) {
    #[cfg(not(target_endian = "little"))]
    {
        // on BE platforms, swap the byte order. This can theoretically be SIMD-optimized but I won't worry about that.
        for frame in _inputs {
            frame.button_bits = ButtonFlags::from_bits_retain(frame.button_bits.bits().to_le());
        }
    }
}
