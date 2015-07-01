use std::net::TcpListener;

use solicit::http::Response;

use solicit::server::SimpleServer;

struct GrpcServer {
    listener: TcpListener,
}

impl GrpcServer {
    fn run(&mut self) {
        for stream in self.listener.incoming().map(|s| s.unwrap()) {
            println!("client connected!");
            let mut server = SimpleServer::new(stream, |req| {
                println!("request");

                Response {
                    headers: vec![],
                    body: vec![],
                    stream_id: req.stream_id,
                }
            }).unwrap();
            while let Ok(_) = server.handle_next() {}
            println!("client disconnected");
        }
    }
}

