use std::path::PathBuf;

use gtk::prelude::*;

use relm4::RelmWidgetExt;

use m64prs_vcr::movie::StartType;
use relm4::Component;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MovieDialogRequest {
    Show {
        mode: MovieDialogMode,
    },
    OkPressed,
    CancelPressed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MovieDialogResponse {
    NewMovie {
        path: PathBuf,
        author: String,
        description: String,
        start_type: StartType,
    },
    OpenMovie {
        path: PathBuf,
    },
    Cancel,
}

pub struct MovieDialogSettings {
    transient_to: Option<gtk::Window>,
}

impl Default for MovieDialogSettings {
    fn default() -> Self {
        Self {
            transient_to: Default::default(),
        }
    }
}

impl MovieDialogSettings {
    pub fn with_transient_to(mut self, widget: &impl IsA<gtk::Widget>) -> Self {
        self.transient_to = widget.toplevel_window();
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MovieDialogMode {
    New,
    Load,
}

#[derive(Debug)]
pub struct MovieDialog {
    current_mode: MovieDialogMode,
}

#[relm4::component(pub)]
impl Component for MovieDialog {
    type Input = MovieDialogRequest;
    type Output = MovieDialogResponse;
    type CommandOutput = ();

    type Init = MovieDialogSettings;

    view! {
        #[root]
        #[name(main_window)]
        gtk::Window {
            set_title: Some("Open Movie..."),
            set_visible: false,

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_margin_all: 10,
                set_spacing: 10,

                gtk::Grid {
                    set_halign: gtk::Align::Fill,
                    set_hexpand: true,
                    set_column_spacing: 10,
                    set_valign: gtk::Align::Fill,
                    set_vexpand: true,
                    set_row_spacing: 10,

                    attach[0, 0, 1, 1] = &gtk::Label {
                        set_halign: gtk::Align::End,
                        set_valign: gtk::Align::Center,
                        set_label: "Movie File:",
                    },
                    attach[1, 0, 1, 1] = &gtk::Box {
                        set_halign: gtk::Align::Fill,
                        set_hexpand: true,
                        set_spacing: 10,

                        gtk::Entry {
                            set_halign: gtk::Align::Fill,
                            set_hexpand: true,
                            set_placeholder_text: Some("Path")
                        },
                        gtk::Button {
                            set_label: "Browse..."
                        }
                    },
                    attach[0, 1, 1, 1] = &gtk::Label {
                        set_halign: gtk::Align::End,
                        set_valign: gtk::Align::Center,
                        set_label: "Author:",
                    },
                    attach[1, 1, 1, 1] = &gtk::Entry {
                        set_halign: gtk::Align::Fill,
                        set_hexpand: true,
                        set_placeholder_text: Some("Author"),
                    },
                    attach[0, 2, 1, 1] = &gtk::Label {
                        set_halign: gtk::Align::End,
                        set_valign: gtk::Align::Center,
                        set_label: "Description:",
                    },
                    attach[1, 2, 1, 1] = &gtk::Frame {
                        set_halign: gtk::Align::Fill,
                        set_hexpand: true,
                        set_size_request: (0, 140),
                        gtk::ScrolledWindow {
                            gtk::TextView {
                                set_wrap_mode: gtk::WrapMode::WordChar
                            }
                        }
                    },
                    attach[0, 3, 1, 1] = &gtk::Label {
                        set_halign: gtk::Align::End,
                        set_valign: gtk::Align::Center,
                        set_label: "Start type:",
                    },
                    attach[1, 3, 1, 1] = &gtk::Box {
                        set_halign: gtk::Align::Fill,
                        set_hexpand: true,
                        set_spacing: 10,

                        #[name(start_type_leader)]
                        gtk::ToggleButton {
                            set_label: "Savestate",
                        },
                        gtk::ToggleButton {
                            set_label: "Reset",
                            set_group: Some(&start_type_leader),
                        },
                        gtk::ToggleButton {
                            set_label: "EEPROM",
                            set_sensitive: false,
                            set_group: Some(&start_type_leader),
                        },
                    },
                },
                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_halign: gtk::Align::End,
                    set_valign: gtk::Align::Fill,
                    set_spacing: 10,

                    gtk::Button {
                        set_label: "OK",
                        connect_clicked => MovieDialogRequest::OkPressed,
                    },
                    gtk::Button {
                        set_label: "Cancel",
                        connect_clicked => MovieDialogRequest::CancelPressed,
                    }
                }
            }
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        _sender: relm4::ComponentSender<Self>,
    ) -> relm4::ComponentParts<Self> {
        let model = Self {
            current_mode: MovieDialogMode::New,
        };

        let widgets = view_output!();
        widgets
            .main_window
            .set_transient_for(init.transient_to.as_ref());

        relm4::ComponentParts { model, widgets }
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::Input,
        sender: relm4::ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            MovieDialogRequest::Show { mode } => {
                self.current_mode = mode;
                widgets.main_window.set_visible(true);
            }
            MovieDialogRequest::OkPressed => {
                widgets.main_window.set_visible(false);
            },
            MovieDialogRequest::CancelPressed => {
                widgets.main_window.set_visible(false);
            },
        }

        self.update_view(widgets, sender);
    }
}
