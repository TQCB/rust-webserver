use std::{
    fs,
    io::{self, BufReader, prelude::*},
    net::{TcpListener, TcpStream},
    path::Path,
    thread,
    time::Duration,
};

fn main()
{
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

    for stream in listener.incoming()
    {
        let stream = stream.unwrap();
        handle_connection(stream);
    }
}

fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&stream);
    let request_line = buf_reader.lines().next().unwrap().unwrap();

    let (status_line, filename) = match &request_line[..] {
        "GET / HTTP/1.1" => ("HTTP/1.1 200 OK", "index.html"),
        "GET /sleep HTTP/1.1" => {
            thread::sleep(Duration::from_secs(5));
            ("HTTP/1.1 200 OK", "index.html")
        },
        _ => ("HTTP/1.1 404 NOT FOUND", "404.html"),
    };

    let content = fs::read_to_string(filename).expect("Should've been able to read .html file");
    let response = format!("{}\r\nContent-Length: {}\r\n\r\n{}", status_line, content.len(), content);

    stream.write_all(response.as_bytes()).unwrap();
}