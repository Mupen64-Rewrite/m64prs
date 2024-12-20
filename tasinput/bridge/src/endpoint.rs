use std::{io, sync::Arc, thread};

use futures::channel::{mpsc, oneshot};
use interprocess::local_socket as socket;
use tasinput_protocol::HostRequest;

pub(crate) struct Endpoint {
    io_thread: thread::JoinHandle<()>,
    send_queue: mpsc::Sender<HostRequest>,
    shutdown: oneshot::Sender<()>,
}

struct EndpointState {
    connection: socket::Stream,
    send_queue: mpsc::Receiver<HostRequest>,
    shutdown: oneshot::Receiver<()>
}

impl Endpoint {
    fn spawn() -> io::Result<Self> {
        todo!()
    }
}
