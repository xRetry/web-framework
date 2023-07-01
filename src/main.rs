use std::{
    fs,
    io::{BufReader, prelude::*},
    net::{TcpListener, TcpStream},
    collections::HashMap,
};

fn create_html_content(body_content: &str, js_path: Option<&str>) -> String {
    let script_elem = match js_path {
        Some(path) => format!("<script src=\"{path}\"></script>"),
        None => "".to_string(),
    };

    format!(r#"<!DOCTYPE html>
    <html lang="en">
        <head>
            <meta charset="utf-8">
            <title>Hello!</title>
        </head>
        <body>
        {script_elem}
        {body_content}
        </body>
    </html>
    "#)
}

fn create_http_header(http_code: i32, content_length: usize, content_type: &str) -> String {
    format!("HTTP/1.1 {http_code} OK\r
Content-Type: {content_type}\r
Content-Length: {content_length}\r\n\r
")
}

struct Framework {
    api_handlers: HashMap<String, fn(&str) -> String>,
}

impl Framework {
    fn new() -> Self {
        return Self{
            api_handlers: HashMap::new(),
        };
    }

    fn create_html_response(&self, base_path: &str) -> String {
        let html_path = format!("pages{base_path}.html");

        println!("{}", &html_path);

        let body_content = fs::read_to_string(html_path)
            .unwrap_or(fs::read_to_string("pages/404.html").unwrap());
        let content = create_html_content(&body_content, Some(&format!("js{base_path}.js")));
        let header = create_http_header(200, content.len(), "text/html");

        return header + &content;
    }

    fn create_js_response(&self, base_path: &str) -> String {
        let js_path = format!("js/pages/{base_path}");
        println!("{}", &js_path);

        let content = fs::read_to_string(js_path)
            .unwrap_or("".to_string());
        let header = create_http_header(200, content.len(), "application/javascript");

        return header + &content;
    }

    fn create_api_response(&self, base_path: &str) -> String {
        let response = match self.api_handlers.get(base_path) {
            Some(handler) => handler(base_path),
            None => return "".to_string(),
        };

        return response;
    }

    fn handle_connection(&self, mut stream: TcpStream) {
        let buf_reader = BufReader::new(&mut stream);
        let request_line = buf_reader.lines().next().unwrap().unwrap();

        let mut base_path = request_line.split_whitespace().skip(1).next().unwrap();
        if base_path == "/" { base_path = "/index"; }

        let response = match base_path[1..base_path.len()].split_once("/") {
            Some(("js", path)) => self.create_js_response(path),
            Some(("api", path)) => self.create_api_response(path),
            _ => self.create_html_response(base_path),
        };


        stream.write_all(response.as_bytes()).unwrap();

    }

    fn run(&self) {
        let listener = TcpListener::bind("127.0.0.1:9000").unwrap();

        for stream in listener.incoming() {
            let stream = stream.unwrap();

            std::thread::spawn(|| {
                self.handle_connection(stream);
            });
        }
    }
}

fn main() {
    let framework = Framework::new();
    framework.run();

}
