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

    // let response = match request_line.as_str() {
    //     "GET / HTTP/1.1" => { match ContentPage::from_html("index.html") {
    //             Ok(content_page) => Response::Page(content_page),
    //             Err(e) => {
    //                 eprintln!("Error reading index.html: {}", e);
    //                 Response::ErrPage(ErrPage::internal())
    //             }
    //         }
    //     },
    //     _ => Response::ErrPage(ErrPage::not_found())
    // };

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

trait Respond {
    fn respond(&self) -> String;
}

enum Response {
    Page(ContentPage),
    ErrPage(ErrPage),
}

impl Respond for Response {
    fn respond(&self) -> String {
        match self {
            Response::Page(content_page) => content_page.respond(),
            Response::ErrPage(err_page) => err_page.respond()
        }
    }
}

struct ContentPage {
    status: String,
    content: String,
    length: usize
}

impl ContentPage{
    fn from_html(file_path: impl AsRef<Path>) -> io::Result<Self> {
        let body = fs::read_to_string(file_path)?;
        let length = body.len();

        Ok(ContentPage {
            status: String::from("200 OK"),
            content: body,
            length: length})
    }
}

impl Respond for ContentPage {
    fn respond(&self) -> String {
        format!(
            "HTTP/1.1 {}\r\nContent-Length: {}\r\n\r\n{}",
            self.status, self.length, self.content,
        )
    }
}

struct ErrPage {
    status: String,
    content: String,
    length: usize
}

impl ErrPage{
    fn not_found() -> Self {
        let content = fs::read_to_string("404.html").unwrap();
        let length = content.len();

        ErrPage {
            status: String::from("404 Not Found"),
            content: content,
            length: length
        }
    }

    fn internal() -> Self {
        ErrPage {
            status: String::from("500 Internal Server Error"),
            content: "<h1>500 Interval Server Error</h1>".to_string(),
            length: "<h1>500 Interval Server Error</h1>".len(),
        }
    }
}

impl Respond for ErrPage {
    fn respond(&self) -> String {
        format!(
            "HTTP/1.1 {}\r\nContent-Length: {}\r\n\r\n{}",
            self.status, self.length, self.content,
        )
    }
}