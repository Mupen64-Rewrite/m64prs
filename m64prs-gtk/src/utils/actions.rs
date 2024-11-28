use gtk::prelude::*;
use relm4::actions::{ActionName, RelmAction};

mod sealed {
    pub trait Sealed {}
}

pub trait RelmActionStateExt<T>: sealed::Sealed
where
    T: FromVariant + ToVariant,
{
    /// Sets the state of an action.
    fn set_state(&self, state: T);
}


impl<S> sealed::Sealed for RelmAction<S>
where
    S: ActionName,
{}
impl<S, T> RelmActionStateExt<T> for RelmAction<S>
where
    S: ActionName<State = T>,
    T: FromVariant + ToVariant,
{
    fn set_state(&self, state: T) {
        self.gio_action().set_state(&state.to_variant());
    }
}
