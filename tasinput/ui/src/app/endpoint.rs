use glib::SendWeakRef;
use gtk::{prelude::*, subclass::prelude::*};
use m64prs_sys::Buttons;
use tasinput_protocol::{
    HostMessage, HostRequest, RequestHandler, UiMessage, UiReply, PING_TIMEOUT,
};
use tokio::{
    sync::mpsc::{self, error::TryRecvError},
    time::{sleep_until, Instant},
};
use tokio_util::sync::CancellationToken;

use crate::main_window::MainWindow;

pub(super) struct UiRequestHandler {
    pub(super) app_ref: SendWeakRef<super::TasDiApp>,
    pub(super) timeout_send: mpsc::Sender<Instant>,
}

impl RequestHandler<UiMessage, HostMessage> for UiRequestHandler {
    async fn handle_request(&mut self, request: HostRequest) -> UiReply {
        // These need to be cloned so that each future gets a copy.
        let app_ref = self.app_ref.clone();
        let timeout_send = self.timeout_send.clone();

        use tasinput_protocol::HostRequest::*;
        match request {
            Ping => {
                timeout_send
                    .try_send(Instant::now() + PING_TIMEOUT)
                    .unwrap();
                UiReply::Ack
            }
            Close => {
                let app_ref = app_ref.clone();
                glib::spawn_future(async move {
                    if let Some(app) = app_ref.upgrade() {
                        app.quit();
                    }
                });
                UiReply::Ack
            }
            InitControllers { active } => {
                let app_ref = app_ref.clone();
                glib::spawn_future(async move {
                    if let Some(app) = app_ref.upgrade() {
                        let bits = active.bits();
                        let mut windows = app.imp().windows.borrow_mut();
                        for i in 0..4 {
                            if (bits & (1 << i)) != 0 {
                                let window = MainWindow::new(&app);
                                window.set_title(Some(&format!("tasinput-ui [{}]", i + 1)));
                                windows[i] = Some(window);
                            }
                        }
                    }
                });
                UiReply::Ack
            }
            SetVisible { visible } => {
                let app_ref = app_ref.clone();
                glib::spawn_future(async move {
                    if let Some(app) = app_ref.upgrade() {
                        let windows = app.imp().windows.borrow();
                        for window in &*windows {
                            window
                                .as_ref()
                                .inspect(|&window| window.set_visible(visible));
                        }
                    }
                });
                UiReply::Ack
            }
            PollState { controller } => {
                let app_ref = app_ref.clone();
                let buttons = glib::spawn_future(async move {
                    if let Some(app) = app_ref.upgrade() {
                        let windows = app.imp().windows.borrow();
                        windows[controller as usize]
                            .as_ref()
                            .map_or(Buttons::BLANK, |val| val.poll_input())
                    } else {
                        Buttons::BLANK
                    }
                })
                .await
                .unwrap();

                UiReply::PolledState { buttons }
            }
        }
    }
}

pub(super) async fn ping_timeout(
    app_ref: SendWeakRef<super::TasDiApp>,
    mut timeout_recv: mpsc::Receiver<Instant>,
    cancel_token: CancellationToken,
) {
    // Wait for the first ping.
    let mut next_time = tokio::select! {
        biased;
        _ = cancel_token.cancelled() => return,
        first_time = timeout_recv.recv() => first_time.unwrap()
    };
    // Ensure that any subsequent pings occur before the timeout.
    loop {
        tokio::select! {
            biased;
            _ = cancel_token.cancelled() => return,
            _ = sleep_until(next_time) => (),
        }
        match timeout_recv.try_recv() {
            Ok(time) => {
                next_time = time;
            }
            Err(TryRecvError::Empty) => {
                let app_ref = app_ref.clone();
                glib::spawn_future(async move {
                    if let Some(app) = app_ref.upgrade() {
                        app.quit();
                    }
                });
            }
            Err(TryRecvError::Disconnected) => {
                panic!("The sender should not disconnect.")
            }
        }
    }
}
