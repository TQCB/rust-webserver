use std::{
    fs,
    io::{BufReader, prelude::*},
    net::{TcpListener, TcpStream},
    thread,
    time::Duration,
};

use webserver::ThreadPool;

fn main() {
    let address = "127.0.0.1:7878";
    let listener = match TcpListener::bind(address) {
        Ok(listener) => {
            println!("Server listening on {}", address);
            listener
        }
        Err(e) => {
            eprintln!("Failed to bind to {}: {}", address, e);
            std::process::exit(1);
        }
    };

    let pool = ThreadPool::new(4);

    for stream in listener.incoming() {
        let stream = match stream {
            Ok(stream) => stream,
            Err(e) => {
                eprintln!("Error accepting connection: {}", e);
                continue;
            }
        };

        pool.execute(|| {
            handle_connection(stream);
        });
    }

    println!("Shutting server down.")
}

/// Represents a parsed HTTP request line.
struct HttpRequest {
    method: String,
    path: String,
    #[allow(dead_code)]
    version: String, // Parsed but not currently used in routing logic
}

/// Parses the HTTP request line into method, path, and version.
///
/// # Arguments
///
/// * `request_line` - The first line of the HTTP request (e.g., "GET / HTTP/1.1")
///
/// # Returns
///
/// Returns `Some(HttpRequest)` if parsing succeeds, `None` otherwise.
fn parse_request_line(request_line: &str) -> Option<HttpRequest> {
    let parts: Vec<&str> = request_line.split_whitespace().collect();
    if parts.len() != 3 {
        return None;
    }

    Some(HttpRequest {
        method: parts[0].to_string(),
        path: parts[1].to_string(),
        version: parts[2].to_string(),
    })
}

/// Reads HTTP headers from the request stream.
///
/// Headers are read line by line until an empty line is encountered,
/// which marks the end of the headers section.
///
/// # Arguments
///
/// * `lines` - An iterator over the lines of the request
///
/// # Returns
///
/// Returns a vector of header lines (excluding the empty line).
fn read_headers<I>(lines: &mut I) -> Vec<String>
where
    I: Iterator<Item = Result<String, std::io::Error>>,
{
    let mut headers = Vec::new();
    for line_result in lines {
        match line_result {
            Ok(line) => {
                // Empty line indicates end of headers
                if line.trim().is_empty() {
                    break;
                }
                headers.push(line);
            }
            Err(_) => break,
        }
    }
    headers
}

/// Handles an incoming TCP connection, parsing the HTTP request and sending a response.
///
/// This function:
/// 1. Parses the HTTP request line to extract method and path
/// 2. Reads HTTP headers (if present)
/// 3. Routes the request based on method and path
/// 4. Sends an appropriate HTTP response
///
/// All errors are handled gracefully, sending appropriate HTTP error responses
/// to the client rather than panicking.
///
/// # Arguments
///
/// * `stream` - The TCP stream representing the client connection
fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&stream);
    let mut lines = buf_reader.lines();

    // Parse the request line
    let request_line = match lines.next() {
        Some(Ok(line)) => line,
        Some(Err(e)) => {
            eprintln!("Error reading request line: {}", e);
            send_error_response(&mut stream, "400 Bad Request", "Invalid request");
            return;
        }
        None => {
            eprintln!("Empty request received");
            send_error_response(&mut stream, "400 Bad Request", "Empty request");
            return;
        }
    };

    // Parse method, path, and version
    let http_request = match parse_request_line(&request_line) {
        Some(req) => req,
        None => {
            eprintln!("Failed to parse request line: {}", request_line);
            send_error_response(&mut stream, "400 Bad Request", "Malformed request line");
            return;
        }
    };

    // Read headers (we don't use them yet, but we parse them to demonstrate understanding)
    let _headers = read_headers(&mut lines);

    // Route based on method and path
    let (status_line, filename) = match (http_request.method.as_str(), http_request.path.as_str()) {
        ("GET", "/") => ("HTTP/1.1 200 OK", "index.html"),
        ("GET", "/sleep") => {
            thread::sleep(Duration::from_secs(5));
            ("HTTP/1.1 200 OK", "index.html")
        }
        _ => ("HTTP/1.1 404 NOT FOUND", "404.html"),
    };

    // Read the file content
    let content = match fs::read_to_string(filename) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error reading file {}: {}", filename, e);
            send_error_response(&mut stream, "500 Internal Server Error", "Failed to read file");
            return;
        }
    };

    // Send the response
    let response = format!(
        "{}\r\nContent-Length: {}\r\n\r\n{}",
        status_line,
        content.len(),
        content
    );

    if let Err(e) = stream.write_all(response.as_bytes()) {
        eprintln!("Error sending response: {}", e);
    }
}

/// Sends an HTTP error response to the client.
///
/// # Arguments
///
/// * `stream` - The TCP stream to write the response to
/// * `status` - The HTTP status line (e.g., "500 Internal Server Error")
/// * `message` - The error message to include in the response body
fn send_error_response(stream: &mut TcpStream, status: &str, message: &str) {
    let response = format!(
        "HTTP/1.1 {}\r\nContent-Length: {}\r\n\r\n{}",
        status,
        message.len(),
        message
    );

    if let Err(e) = stream.write_all(response.as_bytes()) {
        eprintln!("Error sending error response: {}", e);
    }
}
