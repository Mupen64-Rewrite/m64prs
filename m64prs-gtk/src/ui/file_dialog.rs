use std::{cell::Cell, path::PathBuf};

use glib::object::IsA;
use gtk::{
    gio::{self, ListStore},
    prelude::*,
    Widget,
};
use relm4::{ComponentParts, RelmWidgetExt, SimpleComponent};

#[derive(Debug, Clone, Copy)]
pub enum Request {
    Open,
    Save,
}

#[derive(Debug)]
pub enum Response {
    Accept(PathBuf),
    Cancel,
}

pub struct Settings {
    transient_to: Option<gtk::Window>,
    default_filter: Option<gtk::FileFilter>,
    filters: Vec<gtk::FileFilter>,
    title: String,
    accept_label: Option<String>,
    modal: bool,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            transient_to: None,
            default_filter: None,
            filters: vec![],
            title: "Choose Files".to_owned(),
            accept_label: None,
            modal: true,
        }
    }
}
impl Settings {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_transient_to(mut self, widget: &impl IsA<Widget>) -> Self {
        self.transient_to = widget.toplevel_window();
        self
    }
    pub fn with_filters(mut self, filters: Vec<gtk::FileFilter>, default: Option<usize>) -> Self {
        self.default_filter = default.map(|index| filters[index].clone());
        self.filters = filters;
        self
    }
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }
    pub fn with_accept_label(mut self, accept_label: impl Into<String>) -> Self {
        self.accept_label = Some(accept_label.into());
        self
    }
    pub fn with_modal(mut self, modal: bool) -> Self {
        self.modal = modal;
        self
    }

    fn into_widgets(self) -> Widgets {
        let dialog = gtk::FileDialog::new();

        if !self.filters.is_empty() {
            let store = ListStore::new::<gtk::FileFilter>();
            store.extend_from_slice(&self.filters);
            dialog.set_filters(Some(&store));
            dialog.set_default_filter(self.default_filter.as_ref());
        }

        dialog.set_title(&self.title);
        dialog.set_accept_label(self.accept_label.as_ref().map(String::as_str));
        dialog.set_modal(self.modal);

        Widgets {
            dialog,
            transient_window: self.transient_to,
        }
    }
}

#[derive(Debug)]
pub struct Widgets {
    dialog: gtk::FileDialog,
    transient_window: Option<gtk::Window>,
}

#[derive(Debug)]
pub struct Model {
    next_request: Cell<Option<Request>>,
}

impl SimpleComponent for Model {
    type Input = Request;

    type Output = Response;

    type Init = Settings;

    type Root = ();

    type Widgets = Widgets;

    fn init_root() -> Self::Root {
        ()
    }

    fn init(
        settings: Self::Init,
        _root: Self::Root,
        _sender: relm4::ComponentSender<Self>,
    ) -> relm4::ComponentParts<Self> {
        let model = Self {
            next_request: Cell::new(None),
        };
        let widgets = settings.into_widgets();

        ComponentParts::<Self> { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: relm4::ComponentSender<Self>) {
        self.next_request.set(Some(message));
    }

    fn update_view(&self, widgets: &mut Self::Widgets, sender: relm4::ComponentSender<Self>) {
        if let Some(request) = self.next_request.take() {
            let transient = widgets.transient_window.as_ref();

            match request {
                Request::Open => widgets.dialog.open(
                    transient,
                    Option::<&gio::Cancellable>::None,
                    move |result| {
                        let _ = match result {
                            Ok(file) => sender.output(Response::Accept(file.path().unwrap())),
                            Err(_) => sender.output(Response::Cancel),
                        };
                    },
                ),
                Request::Save => widgets.dialog.save(
                    transient,
                    Option::<&gio::Cancellable>::None,
                    move |result| {
                        let _ = match result {
                            Ok(file) => sender.output(Response::Accept(file.path().unwrap())),
                            Err(_) => sender.output(Response::Cancel),
                        };
                    },
                ),
            }
        }
    }
}
