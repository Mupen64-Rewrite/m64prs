use std::sync::{Arc, Mutex};

use tasinput_protocol::{Endpoint, HostMessage, HostRequest, UiMessage, UiReply};
use tokio::sync::oneshot;

fn main() {
    let (close_src, close) = oneshot::channel::<()>();

    let close_src = Arc::new(Mutex::new(Some(close_src)));

    let _client: Endpoint<UiMessage, HostMessage> =
        Endpoint::connect("test-socket-id", move |request| {
            let close_src = Arc::clone(&close_src);
            async move {
                match request {
                    HostRequest::Ping => {
                        println!("ping");
                        UiReply::Ack
                    }
                    HostRequest::Close => {
                        println!("close");
                        let _ = close_src.lock().unwrap().take().unwrap().send(());
                        UiReply::Ack
                    }
                    _ => UiReply::Ack,
                }
            }
        })
        .unwrap();

    println!("client ready");

    close.blocking_recv().unwrap();
}
