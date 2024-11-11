use core::str;
use std::{
    borrow::Borrow,
    fmt::{Debug, Display},
    str::FromStr,
};

use m64prs_sys::Buttons;

use crate::error::StringFieldError;

/// Represents a fixed-capacity UTF-8 field containing a null-terminated string.
#[derive(Clone, PartialEq, Eq, Hash)]
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
        let mut result = Self::default();
        result.try_write(data)?;
        Ok(result)
    }

    /// Tries to read a string from the string field.
    ///
    /// # Errors
    /// This function errors if the string field does not contain valid UTF-8.
    pub fn try_read(&self) -> Result<&str, StringFieldError> {
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
    pub fn try_write<T: Borrow<str>>(&mut self, data: T) -> Result<(), StringFieldError> {
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
    pub fn read(&self) -> &str {
        self.try_read().expect("field should be valid")
    }

    /// Writes to the string field.
    ///
    /// # Panics
    /// Panics if the provided string is too long to fit.
    pub fn write<T: Borrow<str>>(&mut self, data: T) {
        self.try_write(data)
            .expect("data should be short enough to fit")
    }

    /// Writes to the string field, truncating the string if it's too long.
    /// Returns the truncated string.
    pub fn write_clipped<'a>(&mut self, data: &'a str) -> &'a str {
        let split_pos = if data.len() <= N {
            data.len()
        } else {
            (0..=N).rev().find(|n| data.is_char_boundary(*n)).unwrap()
        };

        let slice = &data[0..split_pos];
        if slice.len() == N {
            self.0.copy_from_slice(slice.as_bytes());
        } else {
            self.0[..split_pos].copy_from_slice(slice.as_bytes());
            self.0[split_pos] = b'\0';
        }
        slice
    }
}

impl<const N: usize> Debug for StringField<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("StringField<{}>(\"{:?}\")", N, self.read()))
    }
}

impl<const N: usize> Display for StringField<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.read())
    }
}

impl<const N: usize> FromStr for StringField<N> {
    type Err = StringFieldError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

/// Represents a fixed-capacity ASCII field containing a null-terminated string.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct AsciiField<const N: usize>([u8; N]);

impl<const N: usize> Default for AsciiField<N> {
    /// Returns an empty string field. Will not compile for `N == 0`.
    fn default() -> Self {
        assert!(N > 0, "Cannot read from an empty field");
        Self([0u8; N])
    }
}

impl<const N: usize> AsciiField<N> {
    /// Returns a string field containing the specified string. Will not compile for `N == 0``.
    ///
    /// # Errors
    /// This function errors if the specified string is too long.
    pub fn new<T: Borrow<str>>(data: T) -> Result<Self, StringFieldError> {
        let mut result = Self::default();
        result.try_write(data)?;
        Ok(result)
    }

    /// Tries to read a string from the string field.
    ///
    /// # Errors
    /// This function errors if the string field does not contain valid UTF-8.
    pub fn try_read(&self) -> Result<&str, StringFieldError> {
        assert!(N > 0, "Cannot read from an empty field");
        // Find the null-terminator
        let mut iter = self.0.iter();
        let mut count = 0;
        while let Some(next) = iter.next() {
            if !next.is_ascii() {
                return Err(StringFieldError::AsciiInvalid);
            }
            if *next == b'\0' {
                break;
            }
            count += 1;
        }
        // SAFETY: we verified everything was ASCII in the for loop.
        return unsafe { Ok(std::str::from_utf8_unchecked(&self.0[0..count])) };
    }

    /// Tries to write a string to the string field.
    ///
    /// # Errors
    /// This function errors if the provided string is too long to fit in the string field.
    pub fn try_write<T: Borrow<str>>(&mut self, data: T) -> Result<(), StringFieldError> {
        assert!(N > 0, "Cannot write to an empty field");

        let data: &str = data.borrow();
        let len = data.len();

        if !data.is_ascii() {
            return Err(StringFieldError::AsciiInvalid);
        }

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
    /// Panics if the string field doesn't contain valid ASCII.
    pub fn read(&self) -> &str {
        self.try_read().expect("field should be valid")
    }

    /// Writes to the string field.
    ///
    /// # Panics
    /// Panics if the provided string is too long to fit.
    pub fn write<T: Borrow<str>>(&mut self, data: T) {
        self.try_write(data)
            .expect("data should be short enough, and also valid ASCII")
    }

    /// Writes to the string field, truncating the string if it's too long.
    /// Returns the truncated string.
    pub fn write_clipped<'a>(&mut self, data: &'a str) -> Result<&'a str, StringFieldError> {
        if !data.is_ascii() {
            return Err(StringFieldError::AsciiInvalid);
        }

        let split_pos = if data.len() < N { data.len() } else { N };

        let slice = &data[0..split_pos];
        if slice.len() == N {
            self.0.copy_from_slice(slice.as_bytes());
        } else {
            self.0[..split_pos].copy_from_slice(slice.as_bytes());
            self.0[split_pos] = b'\0';
        }
        Ok(slice)
    }
}

impl<const N: usize> Debug for AsciiField<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("AsciiField<{}>({:?})", N, self.read()))
    }
}

impl<const N: usize> Display for AsciiField<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.read())
    }
}

impl<const N: usize> FromStr for AsciiField<N> {
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
