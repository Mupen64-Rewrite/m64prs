//! [`TOption`] and its implementors [`TNone`] and [`TSome<T>`]; facilities
//! for type-level optional parameters when using `()` doesn't suffice.

use std::marker::PhantomData;

mod sealed {
    pub trait Sealed {}
}

/// Trait representing a type-level option. It can either be [`TNone`] or [`TSome<T>`].
/// 
/// The purpose of this is to encapsulate the concept of an optional type parameter.
/// In most cases, it suffices to use `()` as the `None` type, but on some occasions,
/// it is necessary to make the distinction between no type and unit type.
pub trait TOption: sealed::Sealed {
    /// If true, this `TOption` has a type.
    const HAS_TYPE: bool;
    /// The inner type of the option. Only valid if [`TOption::HAS_TYPE`] is true.
    type Type;
}

/// Marker type representing the absence of an inner type.
#[derive(Debug, Clone, Copy)]
pub struct TNone;
/// Marker type representing the presence of an inner type.
#[derive(Debug, Clone, Copy)]
pub struct TSome<T>(pub PhantomData<T>);

impl sealed::Sealed for TNone {}
impl TOption for TNone {
    const HAS_TYPE: bool = false;
    type Type = ();
}
impl<T> sealed::Sealed for TSome<T> {}
impl<T> TOption for TSome<T> {
    const HAS_TYPE: bool = true;
    type Type = T;
}
