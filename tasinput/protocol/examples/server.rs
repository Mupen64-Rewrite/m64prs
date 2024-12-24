use std::io;

use tasinput_protocol::{
    Endpoint, EndpointListening, HostMessage, HostReply, HostRequest, UiMessage, UiRequest,
};

fn main() {
    let server: EndpointListening<HostMessage, UiMessage> =
        Endpoint::listen("test-socket-id", |request| async {
            match request {
                UiRequest::Dummy => HostReply::Ack,
            }
        })
        .unwrap();

    println!("server listening...");

    let mut server = server.ready_blocking().unwrap();

    println!("connected!...");

    for i in 0..10 {
        println!("ping {}", i);
        let _ = server
            .post_request_blocking(HostRequest::Ping)
            .blocking_recv();
        println!("pong {}", i);
    }

    // wait
    {
        let mut line = String::new();
        let _ = io::stdin().read_line(&mut line);
    }

    println!("closing...");
    let _ = server
        .post_request_blocking(HostRequest::Close)
        .blocking_recv();
    println!("closed...");
}
