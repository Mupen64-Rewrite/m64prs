use std::cell::Cell;

use gtk::{gio, prelude::*, Widget};
use relm4::{RelmWidgetExt, SimpleComponent};

#[derive(Debug, Clone)]
pub enum AlertDialogRequest {
    Show { message: String, detail: String },
}

#[derive(Debug)]
pub enum AlertDialogResponse {
    Choice(usize),
}

pub struct AlertDialogSettings {
    transient_to: Option<gtk::Window>,
    buttons: Vec<String>,
    cancel_index: usize,
    default_index: Option<usize>,
    modal: bool,
}

impl Default for AlertDialogSettings {
    fn default() -> Self {
        Self {
            transient_to: None,
            buttons: vec!["OK".to_owned(), "Cancel".to_owned()],
            cancel_index: 1,
            default_index: Some(0),
            modal: true,
        }
    }
}

impl AlertDialogSettings {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_transient_to(mut self, widget: &impl IsA<Widget>) -> Self {
        self.transient_to = widget.toplevel_window();
        self
    }
    pub fn with_buttons<T, I>(
        mut self,
        buttons: I,
        cancel_index: usize,
        default_index: Option<usize>,
    ) -> Self
    where
        I: IntoIterator<Item = T>,
        String: From<T>,
    {
        let buttons: Vec<String> = buttons.into_iter().map(|item| String::from(item)).collect();

        if cancel_index >= buttons.len()
            || default_index.is_some_and(|value| value >= buttons.len())
        {
            panic!(
                "expected cancel_index and default_index to be values within the size of buttons"
            )
        }

        self.buttons = buttons;
        self.cancel_index = cancel_index;
        self.default_index = default_index;
        self
    }
    pub fn with_modal(mut self, modal: bool) -> Self {
        self.modal = modal;
        self
    }

    fn into_widgets(self) -> AlertDialogWidgets {
        let dialog = gtk::AlertDialog::builder()
            .buttons(self.buttons)
            .cancel_button(self.cancel_index.try_into().unwrap())
            .default_button(
                self.default_index
                    .map_or(-1, |value| value.try_into().unwrap()),
            )
            .modal(self.modal)
            .build();

        AlertDialogWidgets {
            dialog,
            transient_window: self.transient_to,
        }
    }
}

#[derive(Debug)]
pub struct AlertDialogWidgets {
    dialog: gtk::AlertDialog,
    transient_window: Option<gtk::Window>,
}

#[derive(Debug)]
pub struct AlertDialog {
    next_request: Option<AlertDialogRequest>,
    handled: Cell<bool>,
}

impl SimpleComponent for AlertDialog {
    type Input = AlertDialogRequest;

    type Output = AlertDialogResponse;

    type Init = AlertDialogSettings;

    type Root = ();

    type Widgets = AlertDialogWidgets;

    fn init_root() -> Self::Root {
        ()
    }

    fn init(
        settings: Self::Init,
        _root: Self::Root,
        _sender: relm4::ComponentSender<Self>,
    ) -> relm4::ComponentParts<Self> {
        let model = Self {
            next_request: None,
            handled: Cell::new(true),
        };
        let widgets = settings.into_widgets();

        relm4::ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: relm4::ComponentSender<Self>) {
        self.next_request = Some(message);
        self.handled.set(false);
    }

    fn update_view(&self, widgets: &mut Self::Widgets, sender: relm4::ComponentSender<Self>) {
        if let (false, Some(request)) = (self.handled.get(), &self.next_request) {
            let transient = widgets.transient_window.as_ref();

            match request {
                AlertDialogRequest::Show { detail, message } => {
                    widgets.dialog.set_message(message);
                    widgets.dialog.set_detail(detail);

                    widgets.dialog.choose(
                        transient,
                        Option::<&gio::Cancellable>::None,
                        move |result| {
                            if let Ok(index) = result {
                                let _ = sender.output(AlertDialogResponse::Choice(index as usize));
                            }
                        },
                    );
                }
            }
        }
    }
}
