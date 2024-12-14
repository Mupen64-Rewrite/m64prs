use gtk::prelude::*;
use m64prs_gtk_utils::actions::StateParamAction;

struct MovieDialogActions {
    start_type: StateParamAction<u8, u8>
}