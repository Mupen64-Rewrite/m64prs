use std::marker::PhantomData;

use gio::prelude::{ActionExt, FromVariant, ToVariant};
use glib::SignalHandlerId;

use gio::prelude::*;

use super::mp_lib::{TNone, TOption, TSome};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypedAction<OS, OP>
where
    OS: OptionVariantType,
    OP: OptionVariantType,
{
    inner: gio::SimpleAction,
    _marker: PhantomData<(OS, OP)>,
}

pub type BaseAction = TypedAction<TNone, TNone>;
pub type StateAction<S> = TypedAction<TSome<S>, TNone>;
pub type ParamAction<P> = TypedAction<TNone, TSome<P>>;
pub type StateParamAction<S, P> = TypedAction<TSome<S>, TSome<P>>;

impl<OS, OP> TypedAction<OS, OP>
where
    OS: OptionVariantType,
    OP: OptionVariantType,
{
    unsafe fn with_inner(action: &gio::SimpleAction) -> Self {
        Self {
            inner: action.clone(),
            _marker: PhantomData,
        }
    }
}

impl TypedAction<TNone, TNone> {
    pub fn new(name: &str) -> Self {
        Self {
            inner: gio::SimpleAction::new(name, None),
            _marker: PhantomData,
        }
    }
}

impl<S> TypedAction<TSome<S>, TNone>
where
    S: FromVariant + ToVariant,
{
    pub fn with_state(name: &str, init_state: &S) -> Self {
        Self {
            inner: gio::SimpleAction::new_stateful(name, None, &init_state.to_variant()),
            _marker: PhantomData,
        }
    }
}

impl<P> TypedAction<TNone, TSome<P>>
where
    P: FromVariant + ToVariant,
{
    pub fn with_param(name: &str) -> Self {
        Self {
            inner: gio::SimpleAction::new(name, Some(&P::static_variant_type())),
            _marker: PhantomData,
        }
    }
}

impl<S, P> TypedAction<TSome<S>, TSome<P>>
where
    S: FromVariant + ToVariant,
    P: FromVariant + ToVariant,
{
    pub fn with_state_and_param(name: &str, init_state: &S) -> Self {
        Self {
            inner: gio::SimpleAction::new_stateful(
                name,
                Some(&P::static_variant_type()),
                &init_state.to_variant(),
            ),
            _marker: PhantomData,
        }
    }
}

impl<OS> TypedAction<OS, TNone>
where
    OS: OptionVariantType,
{
    pub fn connect_activate<F: Fn(&Self) + 'static>(&self, f: F) -> SignalHandlerId {
        self.inner.connect_activate(move |action, _| {
            let action = unsafe { Self::with_inner(action) };
            f(&action)
        })
    }

    pub fn activate(&self) {
        self.inner.activate(None);
    }
}

impl<OS, P> TypedAction<OS, TSome<P>>
where
    OS: OptionVariantType,
    P: ToVariant + FromVariant,
{
    pub fn connect_activate<F: Fn(&Self, P) + 'static>(&self, f: F) -> SignalHandlerId {
        self.inner.connect_activate(move |action, param| {
            let action = unsafe { Self::with_inner(action) };
            let inner = P::from_variant(&param.expect("action should have parameter"))
                .expect("failed to convert variant to P");
            f(&action, inner)
        })
    }

    pub fn activate(&self, param: &P) {
        self.inner.activate(Some(&param.to_variant()));
    }
}

impl<S, OP> TypedAction<TSome<S>, OP>
where
    S: ToVariant + FromVariant,
    OP: OptionVariantType,
{
    pub fn state(&self) -> S {
        S::from_variant(&self.inner.state().expect("action state should exist"))
            .expect("action state should match type S")
    }

    pub fn set_state(&self, state: &S) {
        self.inner.set_state(&S::to_variant(state));
    }
}

mod sealed {
    pub trait Sealed {}
}
impl<T: TOption> sealed::Sealed for T {}

pub trait OptionVariantType: sealed::Sealed {}
impl OptionVariantType for TNone {}
impl<T: ToVariant + FromVariant> OptionVariantType for TSome<T> {}
