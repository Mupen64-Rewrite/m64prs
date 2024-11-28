use std::path::PathBuf;

use glib::{object::IsA, translate::ToGlibContainerFromSlice};
use gtk::{
    gio::{self, ListStore},
    prelude::*,
    Widget,
};
use relm4::{Component, ComponentParts, RelmWidgetExt};

#[derive(Debug, Clone, Copy)]
pub enum FileDialogRequest {
    Open,
    Save,
}

#[derive(Debug)]
pub enum FileDialogResponse {
    Accept(PathBuf),
    Cancel,
}

pub struct FileDialogSettings {
    transient_to: Option<gtk::Window>,
    default_filter: Option<gtk::FileFilter>,
    filters: Vec<gtk::FileFilter>,
    title: String,
    accept_label: Option<String>,
    modal: bool,
}
impl Default for FileDialogSettings {
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
impl FileDialogSettings {
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

    fn into_widgets(self) -> FileDialogWidgets {
        let dialog = gtk::FileDialog::new();

        let Self { transient_to, default_filter, filters, title, accept_label, modal } = self;

        if !filters.is_empty() {
            let store: gio::ListStore = filters.into_iter().collect();
            dialog.set_filters(Some(&store));
            dialog.set_default_filter(default_filter.as_ref());
        }

        dialog.set_title(&title);
        dialog.set_accept_label(accept_label.as_deref());
        dialog.set_modal(modal);

        FileDialogWidgets {
            dialog,
            transient_window: transient_to,
        }
    }
}

#[derive(Debug)]
pub struct FileDialogWidgets {
    dialog: gtk::FileDialog,
    transient_window: Option<gtk::Window>,
}

#[derive(Debug)]
pub struct FileDialog;

impl Component for FileDialog {
    type Input = FileDialogRequest;

    type Output = FileDialogResponse;

    type Init = FileDialogSettings;

    type Root = ();

    type Widgets = FileDialogWidgets;

    type CommandOutput = ();

    fn init_root() -> Self::Root {
        ()
    }

    fn init(
        settings: Self::Init,
        _root: Self::Root,
        _sender: relm4::ComponentSender<Self>,
    ) -> relm4::ComponentParts<Self> {
        let model = Self;
        let widgets = settings.into_widgets();

        ComponentParts::<Self> { model, widgets }
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        request: Self::Input,
        sender: relm4::ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        let transient = widgets.transient_window.as_ref();

        match request {
            FileDialogRequest::Open => widgets.dialog.open(
                transient,
                Option::<&gio::Cancellable>::None,
                move |result| {
                    let _ = match result {
                        Ok(file) => sender.output(FileDialogResponse::Accept(file.path().unwrap())),
                        Err(_) => sender.output(FileDialogResponse::Cancel),
                    };
                },
            ),
            FileDialogRequest::Save => widgets.dialog.save(
                transient,
                Option::<&gio::Cancellable>::None,
                move |result| {
                    let _ = match result {
                        Ok(file) => sender.output(FileDialogResponse::Accept(file.path().unwrap())),
                        Err(_) => sender.output(FileDialogResponse::Cancel),
                    };
                },
            ),
        }
    }
}
