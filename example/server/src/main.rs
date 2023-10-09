extern crate example_server;
use example_server::ThreadPool;

use std::fs::File;
use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use std::thread;
use std::time::Duration;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let pool = ThreadPool::new(4);
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        handle_connection(stream);
    }
}

fn handle_connection(mut stream: TcpStream) {
    // リクエストを読む
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();
    println!("request: {}", String::from_utf8_lossy(&buffer));
    // GET を分岐させる
    let get = b"GET / HTTP/1.1\r\n";
    let sleep = b"GET /sleep HTTP/1.1\r\n";
    let (status_line, filename) = if buffer.starts_with(get) {
        ("HTTP/1.1 200 OK\r\n\r\n", "example/server/hello.html")
    } else if buffer.starts_with(sleep) {
        thread::sleep(Duration::from_secs(5));
        ("HTTP/1.1 200 OK\r\n\r\n", "example/server/hello.html")
    } else {
        ("HTTP/1.1 404 NOT FOUND\r\n\r\n", "example/server/404.html")
    };
    let mut content = String::new();
    File::open(filename)
        .unwrap()
        .read_to_string(&mut content)
        .unwrap();
    stream
        .write(format!("{}{}", status_line, content).as_bytes())
        .unwrap();
    stream.flush().unwrap();
}
