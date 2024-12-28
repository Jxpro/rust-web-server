use rust_web_server::ThreadPool;
use std::{
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
    {fs, thread, time::Duration},
};

fn main() {
    // 输出：
    // part 1 未开始请求
    // Worker 0 created.
    // Worker 1 created.
    // Worker 2 created.
    // Worker 3 created.
    // Server listening on port 7878
    //
    // part 2 第一次请求
    // Worker 0 got a job; executing.
    // Shutting down.
    // Worker 1 got a job; executing.
    // Shutting down worker 0.
    // Worker 2 disconnected; shutting down.
    // Worker 3 disconnected; shutting down.
    // Worker 0 finished the job.
    // Worker 0 disconnected; shutting down.
    // Shutting down worker 1.
    //
    // part 3 第二次请求
    // Worker 1 finished the job.
    // Worker 1 disconnected; shutting down.
    // Shutting down worker 2.
    // Shutting down worker 3.
    //
    // 分析：
    // part1：这很正常
    // part2：这比较有趣，ThreadPool 甚至在 Worker 0 执行完任务前就开始 Drop，第二个请求已经提前分给了 Worker 1 并关闭了信道
    // 所以 Worker 2 和 Worker 3 直接 disconnect，然后 Worker 0 执行完任务后也 disconnect
    // part3：Worker 1 执行完任务后 disconnect，然后 Worker 2 和 Worker 3 关机
    // 总体来说，应该是与 TcpListener 的 take(2) 有关，如果不指定 take(2)，则会一直监听请求，并以直观的顺序打印日志

    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let pool = ThreadPool::new(4);
    println!("Server listening on port 7878");

    // 仅处理两个请求，模拟优雅停机
    for stream in listener.incoming().take(2) {
        let stream = stream.unwrap();
        pool.execute(|| {
            handle_connection(stream);
        });
    }

    println!("Shutting down.");
}

fn handle_connection(mut stream: TcpStream) {
    let buf_header = BufReader::new(&stream);

    // 第一个 unwrap 是 Option::unwrap，第二个 unwrap 是 Result::unwrap
    let request_line = buf_header.lines().next().unwrap().unwrap();
    let (status, contents) = match &request_line[..] {
        "GET / HTTP/1.1" => ("HTTP/1.1 200 OK", fs::read_to_string("index.html").unwrap()),
        "GET /sleep HTTP/1.1" => {
            thread::sleep(Duration::from_secs(5));
            ("HTTP/1.1 200 OK", fs::read_to_string("index.html").unwrap())
        }
        _ => (
            "HTTP/1.1 404 NOT FOUND",
            fs::read_to_string("404.html").unwrap(),
        ),
    };

    let response = format!(
        "{}\r\nContent-Length: {}\r\n\r\n{}",
        status,
        contents.len(),
        contents
    );

    stream.write_all(response.as_bytes()).unwrap();
}
