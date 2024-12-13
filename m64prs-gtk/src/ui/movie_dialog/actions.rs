use gtk::prelude::*;

use crate::utils::actions::StateParamAction;

struct MovieDialogActions {
    start_type: StateParamAction<u8, u8>
}