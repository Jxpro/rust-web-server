use std::{
    io::{BufRead, BufReader},
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

fn handle_connection(stream: TcpStream) {
    let buf_header = BufReader::new(&stream);
    let http_request: Vec<_> = buf_header
        .lines()
        .map(|line| line.unwrap())
        .take_while(|line| line.is_empty())
        .collect();
    println!("Request: {:#?}", http_request);
}
