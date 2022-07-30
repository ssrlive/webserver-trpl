use async_std::{
    net::{TcpListener, TcpStream},
    prelude::*,
    task::{self, spawn},
};
use futures::StreamExt;
use std::fs;
use std::time::Duration;

#[async_std::main]
async fn main() {
    let opt = Opt::from_args();
    let addr = format!("{}:{}", opt.host, opt.port);

    let listener = TcpListener::bind(addr).await.unwrap();
    let local_addr = listener.local_addr().unwrap();
    println!("Serving on {}", local_addr);

    spawn(async move {
        listener
            .incoming()
            .for_each_concurrent(None, |stream| async move {
                if let Ok(stream) = stream {
                    // handle_connection(stream).await;
                    spawn(handle_connection(stream));
                }
            })
            .await;
    })
    .await;
}

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "webserver", about = "A webserver demo.")]
struct Opt {
    #[structopt(short = "s", long, default_value = "127.0.0.1")]
    host: String,

    #[structopt(short, long, default_value = "7878")]
    port: u16,
}

async fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    let _ = stream.read(&mut buffer).await.unwrap();

    let get = b"GET / HTTP/1.1\r\n";
    let sleep = b"GET /sleep HTTP/1.1\r\n";

    let (status_line, file_name) = if buffer.starts_with(get) {
        ("HTTP/1.1 200 OK", "hello.html")
    } else if buffer.starts_with(sleep) {
        task::sleep(Duration::from_secs(5)).await;
        ("HTTP/1.1 200 OK", "hello.html")
    } else {
        ("HTTP/1.1 404 NOT FOUND", "404.html")
    };

    let contents = fs::read_to_string(file_name).unwrap();
    let response = format!(
        "{}\r\nContent-Length: {}\r\n\r\n{}",
        status_line,
        contents.len(),
        contents,
    );
    let _ = stream.write(response.as_bytes()).await.unwrap();
    stream.flush().await.unwrap();
}
