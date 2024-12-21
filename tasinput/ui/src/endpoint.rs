use std::{
    collections::HashMap,
    io,
    mem::ManuallyDrop,
    sync::atomic::{AtomicU64, Ordering},
    thread,
};

use futures::{SinkExt as _, StreamExt as _};
use interprocess::local_socket::{
    self, traits::tokio::Stream as _, GenericNamespaced, ToNsName as _,
};
use tasinput_protocol::{
    codec::MessageCodec, HostContent, HostMessage, HostReply, HostRequest, UiMessage, UiReply,
    UiRequest,
};
use tokio::sync::{mpsc, oneshot};
use tokio_util::{
    codec::{FramedRead, FramedWrite},
    sync::CancellationToken,
};

pub(crate) struct Endpoint {
    io_thread: ManuallyDrop<thread::JoinHandle<()>>,
    cancel: CancellationToken,
    send_queue: mpsc::Sender<(UiRequest, oneshot::Sender<HostReply>)>,
}

impl Endpoint {
    pub(crate) async fn connect(socket_id: String) -> io::Result<Self> {
        let socket_name = socket_id
            .clone()
            .to_ns_name::<GenericNamespaced>()?
            .into_owned();
        let (send_queue, send_queue_out) =
            mpsc::channel::<(UiRequest, oneshot::Sender<HostReply>)>(16);
        let cancel = CancellationToken::new();

        let io_rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;
        let (io_ready_src, io_ready) = oneshot::channel::<io::Result<()>>();

        let io_thread = thread::spawn({
            let cancel = cancel.clone();
            move || {
                io_rt.block_on(async move {
                    let mut io_data =
                        match EndpointLoop::setup(socket_name, cancel, send_queue_out).await {
                            Ok(io_data) => {
                                let _ = io_ready_src.send(Ok(()));
                                io_data
                            }
                            Err(error) => {
                                let _ = io_ready_src.send(Err(error));
                                return;
                            }
                        };

                    io_data.main_loop().await
                })
            }
        });

        io_ready.await.unwrap()?;

        Ok(Self {
            io_thread: ManuallyDrop::new(io_thread),
            cancel,
            send_queue,
        })
    }

    pub(crate) async fn send_message(&mut self, message: UiRequest) -> HostReply {
        let (waiter_src, waiter) = oneshot::channel::<HostReply>();
        self.send_queue
            .blocking_send((message, waiter_src))
            .unwrap();
        waiter.await.unwrap()
    }
}

impl Drop for Endpoint {
    fn drop(&mut self) {
        self.cancel.cancel();
        unsafe { ManuallyDrop::take(&mut self.io_thread) }
            .join()
            .unwrap();
    }
}

struct EndpointLoop {
    // socket data
    recv: FramedRead<local_socket::tokio::RecvHalf, MessageCodec<HostMessage>>,
    send: FramedWrite<local_socket::tokio::SendHalf, MessageCodec<UiMessage>>,
    // shutdown token
    cancel: CancellationToken,
    // request channels
    send_queue: mpsc::Receiver<(UiRequest, oneshot::Sender<HostReply>)>,
    waiters: HashMap<u64, oneshot::Sender<HostReply>>,
    id_counter: AtomicU64,
}

impl EndpointLoop {
    async fn setup(
        socket_name: local_socket::Name<'_>,
        cancel: CancellationToken,
        send_queue: mpsc::Receiver<(UiRequest, oneshot::Sender<HostReply>)>,
    ) -> io::Result<Self> {
        let (recv, send) = local_socket::tokio::Stream::connect(socket_name)
            .await?
            .split();
        let recv = FramedRead::new(recv, MessageCodec::new());
        let send = FramedWrite::new(send, MessageCodec::new());

        Ok(Self {
            recv,
            send,
            cancel,
            send_queue,
            waiters: HashMap::new(),
            id_counter: AtomicU64::new(0),
        })
    }
    async fn main_loop(&mut self) {
        loop {
            tokio::select! {
                _ = self.cancel.cancelled() => {
                    return
                },
                opt_msg = self.recv.next() => {
                    match opt_msg {
                        Some(Ok(HostMessage { request_id, content })) => match content {
                            HostContent::Request(request) => {
                                let reply = self.handle_request(request).await;
                                let _ = self.send.send(UiMessage {
                                    request_id,
                                    content: reply.into(),
                                });
                            }
                            HostContent::Reply(reply) => {
                                let sender = self.waiters.remove(&request_id).unwrap();
                                let _ = sender.send(reply);
                            }
                        }
                        Some(Err(_)) => (),
                        None => (),
                    }

                }
                next = self.send_queue.recv() => 'label: {
                    let (msg, waiter) = match next {
                        Some(value) => value,
                        None => break 'label,
                    };
                    let id = self.id_counter.fetch_add(1, Ordering::AcqRel);

                    self.waiters.insert(id, waiter);
                    self
                        .send
                        .send(UiMessage {
                            request_id: id,
                            content: msg.into()
                        })
                        .await
                        .expect("Failed to send to UI process!");
                },
            };
        }
    }

    async fn handle_request(&mut self, request: HostRequest) -> UiReply {
        match request {
            HostRequest::Ping => UiReply::Ack,
            HostRequest::Close => todo!(),
            HostRequest::InitControllers { active } => todo!(),
        }
    }
}
