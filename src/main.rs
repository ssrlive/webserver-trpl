use std::fs;
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::time::Duration;
use webserver::ThreadPool;

fn main() {
    let opt = Opt::from_args();
    let addr = format!("{}:{}", opt.host, opt.port);
    println!("Serving on {}", addr);

    let listener = TcpListener::bind(addr).unwrap();
    {
        let pool = ThreadPool::new(4);
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    pool.execute(|| handle_connection(stream));
                }
                Err(e) => {
                    println!("Error: {:#?}", e);
                }
            }
        }
    }
}

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "webserver", about = "A webserver demo.")]
struct Opt {
    #[structopt(short, long, default_value = "127.0.0.1")]
    host: String,

    #[structopt(short, long, default_value = "7878")]
    port: u16,
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    let _ = stream.read(&mut buffer).unwrap();

    let get = b"GET / HTTP/1.1\r\n";
    let sleep = b"GET /sleep HTTP/1.1\r\n";

    let (status_line, file_name) = if buffer.starts_with(get) {
        ("HTTP/1.1 200 OK", "hello.html")
    } else if buffer.starts_with(sleep) {
        thread::sleep(Duration::from_secs(5));
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
    let _ = stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}
