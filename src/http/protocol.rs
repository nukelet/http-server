use std::collections::HashMap;
use std::fs::File;
use std::io::{ErrorKind as IoErrorKind, Read};
use std::os::unix::fs::MetadataExt;
use std::path::Path;
use std::time::SystemTime;

use chrono::offset::Utc;
use chrono::DateTime;

/*
 * TODO:
 * - Clean up the ugly String hacks (use lifetimes and
 *   slices instead)
 * - Set up lifetimes instead of creating copies
 */

#[derive(Debug, Clone, Copy)]
pub enum Method {
    Get,
    Head,
    Trace,
    Options,
}

#[derive(Debug, Clone)]
pub struct Request {
    pub method: Method,
    pub resource: String,
    pub version: String,
    pub headers: HashMap<String, String>,
    pub raw_request: String,
}

#[repr(u16)]
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum StatusCode {
    Ok = 100,
    BadRequest = 400,
    Forbidden = 403,
    NotFound = 404,
    MethodNotAllowed = 405,
    NotImplemented = 501,
    HttpVersionNotSupported = 505,
}

impl StatusCode {
    // TODO: return this as Result<StatusCode, InvalidStatusCode>?
    pub fn from_u16(code: u16) -> StatusCode {
        match code {
            200 => StatusCode::Ok,
            400 => StatusCode::BadRequest,
            403 => StatusCode::Forbidden,
            404 => StatusCode::NotFound,
            405 => StatusCode::MethodNotAllowed,
            501 => StatusCode::NotImplemented,
            505 => StatusCode::HttpVersionNotSupported,
            _ => StatusCode::NotImplemented,
        }
    }
}

#[derive(Debug)]
pub struct Response {
    pub status: StatusCode,
    pub headers: HashMap<String, String>,
    pub message: String,
}

pub struct RequestHandler {
    pub version: String,
    pub description: String,
    pub root_dir: String,

    pub response_status: StatusCode,
}

struct Resource {
    data: String,
    size: u64,
    last_modified: SystemTime,
}

impl RequestHandler {
    pub fn process_request(&mut self, request: &Request) -> Response {
        // prepare the headers
        let mut headers = self.assemble_initial_headers(request);
        let mut message = String::from("");
        match request.method {
            Method::Get | Method::Head => {
                if let Ok((code, res)) = self.get_resource(&request.resource) {
                    self.response_status = code;
                    let date: DateTime<Utc> = res.last_modified.into();
                    // println!("{}", date.format("%a %d %b %Y %H:%M:%S GMT"));
                    headers.insert(
                        "Last-Modified".to_string(),
                        date.format("%a %d %b %Y %H:%M:%S GMT").to_string(),
                    );
                    headers.insert("Content-Length".to_string(), format!("{}", res.size));
                    headers.insert("Content-Type".to_string(), "text/html".to_string());
                }
            }

            Method::Options => {
                headers.insert("Allow".to_string(), "OPTIONS, GET, HEAD, TRACE".to_string());
            }

            Method::Trace => {
                headers.insert("Content-Type".to_string(), "message/html".to_string());
            }
        };

        // assemble the message body (if applicable)
        match request.method {
            Method::Head | Method::Options => {}
            Method::Get => {
                // TODO: this really could be handled more elegantly (we're currently
                // opening/reading the resource twice)
                match self.get_resource(&request.resource) {
                    Ok((code, resource)) => {
                        message.push_str(resource.data.as_str());
                        self.response_status = code;
                    }
                    Err(code) => {
                        self.response_status = code;
                    }
                }
            }

            Method::Trace => {
                message.push_str(request.raw_request.as_str());
            }
        }

        Response {
            status: self.response_status,
            headers,
            message,
        }
    }

    fn assemble_initial_headers(&self, request: &Request) -> HashMap<String, String> {
        let mut headers: HashMap<String, String> = HashMap::new();

        headers.insert("Server".to_string(), self.version.clone());
        let timestamp = Utc::now().format("%a %d %b %Y %H:%M:%S GMT").to_string();
        println!("{}", timestamp);
        headers.insert("Date".to_string(), timestamp);

        match request.headers.get("Connection") {
            Some(c) => {
                headers.insert("Connection".to_string(), c.to_owned());
            }
            None => {
                headers.insert("Connection".to_string(), "close".to_string());
            }
        }

        headers
    }

    fn get_resource(&self, resource: &str) -> Result<(StatusCode, Resource), StatusCode> {
        println!("{}", self.root_dir);
        let mut path = Path::new(&self.root_dir).join(resource);
        println!("{}", path.display());
        if path.is_dir() {
            println!(
                "{} is a directory; looking for {} or {}...",
                path.display(),
                path.clone().join("index.html").as_path().display(),
                path.clone().join("welcome.html").as_path().display()
            );
            if path.clone().join("index.html").exists() {
                println!("found index.html");
                path = path.join("index.html");
            } else if path.clone().join("welcome.html").exists() {
                println!("found welcome.html");
                path = path.join("welcome.html");
            } else {
                return Err(StatusCode::NotFound);
            }
        }

        println!("fetching resource at {}", path.display());

        match File::open(path) {
            Ok(mut file) => {
                // TODO: these can return errors depending on the platform,
                // but will run fine on Unix... maybe handle things more
                // gracefully instead of unwrapping everything?
                let metadata = file.metadata().unwrap();
                let size = metadata.len();
                let last_modified = metadata.modified().unwrap();
                // TODO: implement content_type detection properly
                let mut data = String::new();
                // TODO: check if it's a directory
                file.read_to_string(&mut data);
                Ok((
                    StatusCode::Ok,
                    Resource {
                        data,
                        size,
                        last_modified,
                    },
                ))
            }
            Err(e) => match e.kind() {
                IoErrorKind::NotFound => Err(StatusCode::NotFound),
                _ => Err(StatusCode::Forbidden),
            },
        }
    }
}
