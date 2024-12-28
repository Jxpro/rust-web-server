use std::{
    fs,
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    println!("Server listening on port 7878");

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        println!("Connection established!");

        handle_connection(stream);
    }
}

fn handle_connection(mut stream: TcpStream) {
    let buf_header = BufReader::new(&stream);

    // 第一个 unwrap 是 Option::unwrap，第二个 unwrap 是 Result::unwrap
    let request_line = buf_header.lines().next().unwrap().unwrap();

    let (status, contents) = if request_line == "GET / HTTP/1.1" {
        ("HTTP/1.1 200 OK", fs::read_to_string("index.html").unwrap())
    } else {
        ("HTTP/1.1 404 NOT FOUND", fs::read_to_string("404.html").unwrap())
    };

    let response = format!(
        "{}\r\nContent-Length: {}\r\n\r\n{}",
        status,
        contents.len(),
        contents
    );

    stream.write_all(response.as_bytes()).unwrap();
}
