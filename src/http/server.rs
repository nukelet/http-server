use crate::http::parser;
use crate::http::protocol::{
    RequestHandler,
    StatusCode,
};
use std::fs;
use std::path::Path;
use std::str;

pub struct Server {
    request_handler: RequestHandler,
}

#[allow(dead_code)]
impl Server {
    pub fn new(root_path: &str) -> Server {
        Server{
            request_handler: RequestHandler {
                version: "HTTP/1.1".to_string(),
                description: "HTTP Server v0.1".to_string(),
                root_dir: root_path.to_string(),
                response_status: StatusCode::Ok,
            }
        }
    }

    pub fn process_request_str(&mut self, input: &str) -> std::io::Result<String> {
        let response = match parser::parse_request(input) {
            Ok((_, request)) => {
                self.request_handler.process_request(&request)
            },
            Err(_) => {
                self.request_handler.bad_request()
            },
        };

        let mut s = String::from("");

        s.push_str(response.status.as_str());
        s.push_str("\r\n");
        
        for (name, content) in response.headers {
            s.push_str(&format!("{}: {}\r\n", name, content));
        }

        s.push_str("\r\n");
        s.push_str(&response.message);
        s.push_str("\r\n");

        Ok(s)
    }

    pub fn process_request_file(&mut self, path: &Path) -> std::io::Result<String> {
        let file = fs::read(path)?;
        // TODO: we need to handle this (in case there are
        // non-utf8 characters in the file)
        let input = str::from_utf8(&file).unwrap();
        self.process_request_str(input)
    }

    pub fn process_request_buf(&mut self, buf: &[u8]) -> std::io::Result<String> {
        // TODO: handle non-utf8 buffer
        self.process_request_str(std::str::from_utf8(buf).unwrap())
    }
}
