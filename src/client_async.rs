use std::thread;
use std::net::SocketAddr;
use std::net::ToSocketAddrs;

use futures::Future;
use futures::stream::channel;
use futures::stream::Sender;
use futures::stream::Receiver;
use futures::oneshot;
use futures::Oneshot;
use futures_mio::Loop;

use futures_grpc::GrpcFuture;

use channel_sync_sender::SyncSender;
use channel_sync_sender::channel_sync_sender;
use method::MethodDescriptor;
use result::GrpcError;


pub struct GrpcClientAsync {
    tx: SyncSender<CallRequest, GrpcError>,
}

struct CallRequest {

}

impl GrpcClientAsync {
    pub fn new(host: &str, port: u16) -> GrpcClientAsync {

        let (tx, rx) = channel_sync_sender();

        // TODO: sync
        let socket_addr = (host, port).to_socket_addrs().unwrap().next().unwrap();

        thread::spawn(move || {
            run_event_loop(socket_addr, rx);
        });

        let r = GrpcClientAsync {
            tx: tx,
        };

        r
    }

    pub fn call<Req : Send + 'static, Resp : Send + 'static>(&self, req: Req, method: MethodDescriptor<Req, Resp>) -> GrpcFuture<Resp> {
        let (complete, oneshot) = oneshot();

        self.tx.send(Ok(CallRequest {
            // TODO
        }));

        oneshot.map_err(|e| GrpcError::Other).boxed()
    }
}

fn run_event_loop(socket_addr: SocketAddr, rx: Receiver<CallRequest, GrpcError>) {
    let mut lp = Loop::new().unwrap();

    let connect = lp.handle().tcp_connect(&socket_addr);

    let client = connect;

    lp.run(client).unwrap();
}
