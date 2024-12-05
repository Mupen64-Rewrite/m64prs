use std::marker::PhantomData;

mod sealed {
    pub trait Sealed {}
}

pub trait TBool: sealed::Sealed {
    const VALUE: bool;
}
pub trait TOption: sealed::Sealed {
    type HasType: TBool;
    type Type;
}

pub struct TTrue;
pub struct TFalse;

impl sealed::Sealed for TTrue {}
impl TBool for TTrue {
    const VALUE: bool = true;
}

impl sealed::Sealed for TFalse {}
impl TBool for TFalse {
    const VALUE: bool = false;
}

pub struct TNone;
pub struct TSome<T>(PhantomData<T>);

impl sealed::Sealed for TNone {}
impl TOption for TNone {
    type HasType = TFalse;
    type Type = ();
}
impl<T> sealed::Sealed for TSome<T> {}
impl<T> TOption for TSome<T> {
    type HasType = TTrue;
    type Type = T;
}
