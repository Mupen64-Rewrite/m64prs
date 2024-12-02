use std::sync::{
    atomic::{self, AtomicUsize},
    mpsc,
};

use relm4::{ComponentSender, Sender};

use crate::ui::core::{self, CoreResponse};

use super::{VideoExtensionParameters, VidextRequest, VidextResponse};

pub(super) struct RequestManager {
    uid_counter: AtomicUsize,
    outbound: Sender<CoreResponse>,
    inbound: mpsc::Receiver<(usize, VidextResponse)>,
}

impl RequestManager {
    pub(super) fn new(
        outbound: Sender<CoreResponse>,
        inbound: mpsc::Receiver<(usize, VidextResponse)>,
    ) -> Self {
        Self {
            uid_counter: AtomicUsize::new(0),
            outbound,
            inbound,
        }
    }

    pub(super) fn request(&self, req: VidextRequest) -> Result<VidextResponse, mpsc::RecvError> {
        // get request ID (used to verify that the request is indeed the correct one)
        let id = self.uid_counter.fetch_add(1, atomic::Ordering::AcqRel);
        // send out the request
        self.outbound
            .send(core::CoreResponse::VidextRequest(id, req));
        // wait for a reply
        self.inbound.recv().map(|(reply_id, resp)| {
            assert!(reply_id == id, "reply should correspond to request");
            resp
        })
    }

    pub(super) fn cleanup(self) -> VideoExtensionParameters {
        VideoExtensionParameters {
            outbound: self.outbound,
            inbound: self.inbound,
        }
    }
}
