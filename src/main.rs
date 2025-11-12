use std::{
    io::{BufReader, prelude::*},
    net::{TcpListener, TcpStream},
};

fn main()
{
    // binding can fail but we just unwrap
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

    for stream in listener.incoming()
    {
        // iterating over attempted client connections
        let stream = stream.unwrap();
        handle_connection(stream);
    }
}

fn handle_connection(mut stream: TcpStream)
{
    let buf_reader = BufReader::new(&stream);
    let http_request: Vec<String> = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    let response = "HTTP/1.1 200 OK\r\n\r\n";
    stream.write_all(response.as_bytes()).unwrap();
}