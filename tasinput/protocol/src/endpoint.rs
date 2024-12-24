use std::{
    cell::RefCell,
    collections::HashMap,
    future::Future,
    io,
    mem::ManuallyDrop,
    sync::atomic::{AtomicU64, Ordering},
    thread,
    time::Duration,
};

use futures::{SinkExt as _, StreamExt as _};
use interprocess::local_socket::{
    self,
    traits::tokio::{Listener as _, Stream as _},
    GenericNamespaced, ToNsName as _,
};
use rand::{rngs::OsRng, RngCore};
use tokio::{
    runtime::Handle,
    sync::{mpsc, oneshot},
};
use tokio_util::{
    codec::{FramedRead, FramedWrite},
    sync::CancellationToken,
};

use crate::{
    codec::MessageCodec,
    types::{IpcMessage, IpcPayload},
};

const PING_TIMEOUT: Duration = Duration::from_secs(5);

/// An IPC endpoint with its own event loop thread.
pub struct Endpoint<OwnMsg, RemoteMsg>
where
    OwnMsg: IpcMessage,
    RemoteMsg: IpcMessage,
{
    io_thread: ManuallyDrop<thread::JoinHandle<()>>,
    rt_handle: Handle,
    send_queue: mpsc::Sender<(OwnMsg::Request, oneshot::Sender<RemoteMsg::Reply>)>,
    cancel: CancellationToken,
}

pub struct EndpointHandle<OwnMsg, RemoteMsg>
where
    OwnMsg: IpcMessage,
    RemoteMsg: IpcMessage,
{
    send_queue: mpsc::Sender<(OwnMsg::Request, oneshot::Sender<RemoteMsg::Reply>)>,
    cancel: CancellationToken,
}

/// An endpoint that is listening for a connection.
pub struct EndpointListening<OwnMsg, RemoteMsg>
where
    OwnMsg: IpcMessage,
    RemoteMsg: IpcMessage,
{
    inner: Endpoint<OwnMsg, RemoteMsg>,
    io_ready: oneshot::Receiver<io::Result<()>>,
}

impl<OwnMsg, RemoteMsg> Endpoint<OwnMsg, RemoteMsg>
where
    OwnMsg: IpcMessage,
    RemoteMsg: IpcMessage,
{
    /// Begins listening on a socket.
    pub fn listen<H, Fut>(
        socket_id: &str,
        handler: H,
    ) -> io::Result<EndpointListening<OwnMsg, RemoteMsg>>
    where
        H: FnMut(RemoteMsg::Request) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = OwnMsg::Reply>,
    {
        let socket_name = socket_id.to_ns_name::<GenericNamespaced>()?.into_owned();
        let (send_queue, send_queue_out) =
            mpsc::channel::<(OwnMsg::Request, oneshot::Sender<RemoteMsg::Reply>)>(16);
        let cancel = CancellationToken::new();

        let io_rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;
        let (io_ready_src, io_ready) = oneshot::channel::<io::Result<()>>();

        let rt_handle = io_rt.handle().clone();

        let io_thread = thread::spawn({
            let cancel = cancel.clone();
            move || {
                io_rt.block_on(async move {
                    let cancel2 = cancel.clone();

                    let mut io_ctx = tokio::select! {
                        // This ensures that dropping cancels the thread if it's still listening.
                        _ = cancel2.cancelled() => {
                            return;
                        },
                        // Listen for a connection.
                        io_ctx = EndpointContext::<OwnMsg, RemoteMsg, H, Fut>::listen(
                            handler,
                            socket_name,
                            cancel,
                            send_queue_out,
                        ) => match io_ctx {
                            Ok(io_ctx) => {
                                let _ = io_ready_src.send(Ok(()));
                                io_ctx
                            }
                            Err(error) => {
                                let _ = io_ready_src.send(Err(error));
                                return;
                            }
                        }
                    };
                    // Run the IO main loop.
                    io_ctx.main_loop().await;
                })
            }
        });

        Ok(EndpointListening {
            inner: Self {
                io_thread: ManuallyDrop::new(io_thread),
                rt_handle,
                send_queue,
                cancel,
            },
            io_ready,
        })
    }

    /// Connects to an existing socket.
    pub fn connect<H, Fut>(socket_id: &str, handler: H) -> io::Result<Self>
    where
        H: FnMut(RemoteMsg::Request) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = OwnMsg::Reply>,
    {
        let socket_name = socket_id.to_ns_name::<GenericNamespaced>()?.into_owned();
        let (send_queue, send_queue_out) =
            mpsc::channel::<(OwnMsg::Request, oneshot::Sender<RemoteMsg::Reply>)>(16);
        let cancel = CancellationToken::new();

        let io_rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;
        let (io_ready_src, io_ready) = oneshot::channel::<io::Result<()>>();

        let rt_handle = io_rt.handle().clone();

        let io_thread = thread::spawn({
            let cancel = cancel.clone();
            move || {
                io_rt.block_on(async move {
                    let cancel2 = cancel.clone();

                    let mut io_ctx = tokio::select! {
                        // This ensures that dropping cancels the thread if it's still listening.
                        _ = cancel2.cancelled() => {
                            return;
                        },
                        // Listen for a connection.
                        io_ctx = EndpointContext::<OwnMsg, RemoteMsg, H, Fut>::connect(
                            handler,
                            socket_name,
                            cancel,
                            send_queue_out,
                        ) => match io_ctx {
                            Ok(io_ctx) => {
                                let _ = io_ready_src.send(Ok(()));
                                io_ctx
                            }
                            Err(error) => {
                                let _ = io_ready_src.send(Err(error));
                                return;
                            }
                        }
                    };
                    // Run the IO main loop.
                    io_ctx.main_loop().await;
                })
            }
        });

        io_ready.blocking_recv().unwrap()?;

        Ok(Self {
            io_thread: ManuallyDrop::new(io_thread),
            rt_handle,
            send_queue,
            cancel,
        })
    }

    /// Posts a message to the socket, returning a one-shot receiver that may contain the reply.
    /// Dropping the receiver will simply discard the reply.
    pub fn post_request_blocking(
        &mut self,
        request: OwnMsg::Request,
    ) -> oneshot::Receiver<RemoteMsg::Reply> {
        let (waiter_src, waiter) = oneshot::channel();
        self.send_queue
            .blocking_send((request, waiter_src))
            .unwrap();
        waiter
    }

    /// Posts a message to the socket, returning a one-shot receiver that may contain the reply.
    /// Dropping the receiver will simply discard the reply.
    pub async fn post_request(
        &mut self,
        request: OwnMsg::Request,
    ) -> oneshot::Receiver<RemoteMsg::Reply> {
        let (waiter_src, waiter) = oneshot::channel();
        self.send_queue.send((request, waiter_src)).await.unwrap();
        waiter
    }

    /// Spawns a future on the I/O runtime. If your future runs for a long period of time,
    /// it is recommended to allow cancellation through the provided handle.
    pub fn spawn<T, F, Fut>(&mut self, future_gen: F) -> tokio::task::JoinHandle<T>
    where
        T: Send + 'static,
        F: FnOnce(EndpointHandle<OwnMsg, RemoteMsg>) -> Fut,
        Fut: Future<Output = T> + Send + 'static,
    {
        let future = future_gen(self.create_handle());
        self.rt_handle.spawn(future)
    }

    fn create_handle(&self) -> EndpointHandle<OwnMsg, RemoteMsg> {
        EndpointHandle {
            send_queue: self.send_queue.clone(),
            cancel: self.cancel.clone(),
        }
    }
}

impl<OwnMsg, RemoteMsg> Drop for Endpoint<OwnMsg, RemoteMsg>
where
    OwnMsg: IpcMessage,
    RemoteMsg: IpcMessage,
{
    fn drop(&mut self) {
        self.cancel.cancel();
        unsafe {
            // The thread is only ever taken here.
            let _ = ManuallyDrop::take(&mut self.io_thread).join();
        }
    }
}

impl<OwnMsg, RemoteMsg> EndpointListening<OwnMsg, RemoteMsg>
where
    OwnMsg: IpcMessage,
    RemoteMsg: IpcMessage,
{
    /// Waits until the endpoint acquires a connection.
    pub async fn ready(self) -> io::Result<Endpoint<OwnMsg, RemoteMsg>> {
        self.io_ready.await.unwrap()?;
        Ok(self.inner)
    }
    /// Blocks until the endpoint acquires a connection.
    pub fn ready_blocking(self) -> io::Result<Endpoint<OwnMsg, RemoteMsg>> {
        self.io_ready.blocking_recv().unwrap()?;
        Ok(self.inner)
    }
}

impl<OwnMsg, RemoteMsg> EndpointHandle<OwnMsg, RemoteMsg>
where
    OwnMsg: IpcMessage,
    RemoteMsg: IpcMessage,
{
    /// Posts a message to the socket, returning a one-shot receiver that may contain the reply.
    /// Dropping the receiver will simply discard the reply.
    pub fn post_request(
        &mut self,
        request: OwnMsg::Request,
    ) -> oneshot::Receiver<RemoteMsg::Reply> {
        let (waiter_src, waiter) = oneshot::channel();
        self.send_queue
            .blocking_send((request, waiter_src))
            .unwrap();
        waiter
    }

    /// Posts a message to the socket, returning a one-shot receiver that may contain the reply.
    /// Dropping the receiver will simply discard the reply.
    pub async fn post_request_async(
        &mut self,
        request: OwnMsg::Request,
    ) -> oneshot::Receiver<RemoteMsg::Reply> {
        let (waiter_src, waiter) = oneshot::channel();
        self.send_queue.send((request, waiter_src)).await.unwrap();
        waiter
    }

    /// Returns a cancellation token that is triggered when this endpoint is dropped.
    pub fn cancel_token(&self) -> CancellationToken {
        self.cancel.clone()
    }
}

pub fn gen_socket_id(prefix: &str) -> String {
    thread_local! {
        static OS_RAND: RefCell<OsRng> = RefCell::new(OsRng::default());
    };

    let mut uid_bytes = [0u8; 16];
    OS_RAND.with_borrow_mut(|rng| rng.fill_bytes(&mut uid_bytes));
    let uid = u128::from_ne_bytes(uid_bytes);

    format!("{}{:016X}", prefix, uid)
}

struct EndpointContext<OwnMsg, RemoteMsg, Handler, Fut>
where
    OwnMsg: IpcMessage,
    RemoteMsg: IpcMessage,
    Handler: FnMut(RemoteMsg::Request) -> Fut,
    Fut: Future<Output = OwnMsg::Reply>,
{
    // socket data
    recv: FramedRead<local_socket::tokio::RecvHalf, MessageCodec<RemoteMsg>>,
    send: FramedWrite<local_socket::tokio::SendHalf, MessageCodec<OwnMsg>>,
    // handler
    handler: Handler,
    // shutdown token
    cancel: CancellationToken,
    // request channels
    send_queue: mpsc::Receiver<(OwnMsg::Request, oneshot::Sender<RemoteMsg::Reply>)>,
    waiters: HashMap<u64, oneshot::Sender<RemoteMsg::Reply>>,
    id_counter: AtomicU64,
}

impl<OwnMsg, RemoteMsg, Handler, Fut> EndpointContext<OwnMsg, RemoteMsg, Handler, Fut>
where
    OwnMsg: IpcMessage,
    RemoteMsg: IpcMessage,
    Handler: FnMut(RemoteMsg::Request) -> Fut,
    Fut: Future<Output = OwnMsg::Reply>,
{
    async fn listen(
        handler: Handler,
        socket_name: local_socket::Name<'_>,
        cancel: CancellationToken,
        send_queue: mpsc::Receiver<(OwnMsg::Request, oneshot::Sender<RemoteMsg::Reply>)>,
    ) -> io::Result<Self> {
        let listener = local_socket::ListenerOptions::new()
            .name(socket_name)
            .create_tokio()?;

        let (recv, send) = listener.accept().await?.split();
        let recv = FramedRead::new(recv, MessageCodec::new());
        let send = FramedWrite::new(send, MessageCodec::new());

        Ok(Self {
            recv,
            send,
            handler,
            cancel,
            send_queue,
            waiters: HashMap::new(),
            id_counter: AtomicU64::new(0),
        })
    }

    async fn connect(
        handler: Handler,
        socket_name: local_socket::Name<'_>,
        cancel: CancellationToken,
        send_queue: mpsc::Receiver<(OwnMsg::Request, oneshot::Sender<RemoteMsg::Reply>)>,
    ) -> io::Result<Self> {
        let (recv, send) = local_socket::tokio::Stream::connect(socket_name)
            .await?
            .split();
        let recv = FramedRead::new(recv, MessageCodec::new());
        let send = FramedWrite::new(send, MessageCodec::new());

        Ok(Self {
            recv,
            send,
            handler,
            cancel,
            send_queue,
            waiters: HashMap::new(),
            id_counter: AtomicU64::new(0),
        })
    }

    async fn main_loop(&mut self) {
        'main_loop: loop {
            tokio::select! {
                biased;
                _ = self.cancel.cancelled() => {
                    break 'main_loop;
                },
                packet = self.recv.next() => match packet {
                    Some(Ok(msg)) => self.handle_message(msg).await,
                    Some(Err(_)) => (),
                    None => (),
                },
                packet = self.send_queue.recv() => match packet {
                    Some((request, waiter)) => {
                        let id = self.id_counter.fetch_add(1, Ordering::AcqRel);
                        self.waiters.insert(id, waiter);
                        self.send.send(OwnMsg::encode_request(id, request)).await.unwrap();
                    },
                    None => (),
                },
            }
        }
    }

    async fn handle_message(&mut self, msg: RemoteMsg) {
        let (id, content) = msg.decode_message();
        match content {
            IpcPayload::Request(request) => {
                let reply = (self.handler)(request).await;
                self.send
                    .send(OwnMsg::encode_reply(id, reply))
                    .await
                    .unwrap();
            }
            IpcPayload::Reply(reply) => {
                let waiter = self.waiters.remove(&id).unwrap();
                let _ = waiter.send(reply);
            }
        }
    }
}
