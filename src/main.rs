use std::fs;
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread;
use std::time::Duration;
use webserver::ThreadPool;

fn main() {
    let opt = Opt::from_args();
    let addr = format!("{}:{}", opt.host, opt.port);
    println!("Serving on {}", addr);

    let listener = TcpListener::bind(addr).unwrap();
    let local_addr = listener.local_addr().unwrap();

    let shutdown_signal = Arc::new(AtomicBool::new(false));
    let shutdown_signal_copy = shutdown_signal.clone();

    let handle = thread::spawn(move || {
        let pool = ThreadPool::new(4);
        for stream in listener.incoming() {
            if shutdown_signal_copy.load(Ordering::Relaxed) {
                break;
            }
            match stream {
                Ok(stream) => {
                    pool.execute(|| handle_connection(stream));
                }
                Err(e) => {
                    println!("Error: {:#?}", e);
                }
            }
        }
    });

    ctrlc::set_handler(move || {
        println!("Shutting down...");
        shutdown_signal.store(true, Ordering::Relaxed);
        let _ = TcpStream::connect(local_addr);
    })
    .expect("Error setting Ctrl-C handler");

    handle.join().unwrap();
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
