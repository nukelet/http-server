use std::collections::HashMap;
use std::fs::File;
use std::io::{ErrorKind as IoErrorKind, Read};
use std::os::unix::fs::MetadataExt;
use std::path::Path;
use std::time::SystemTime;

use chrono::offset::Utc;

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
    // TODO: return this as Result<StatusCode, InvalidStatusCode>
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
    pub version: String,
    pub reason: String,
    pub headers: HashMap<String, String>,
    pub message: Vec<u8>,
}

// struct RequestHandler {
//     request: Request,
//
//     resource_path: String,
//     resource: Option<File>,
//     status: StatusCode,
// }
//
// impl RequestHandler {
//     pub fn new(req: &Request) -> RequestHandler {
//         let request_handler = RequestHandler {
//             request: req.clone(),
//             resource_path: req.resource.clone(),
//             resource: None,
//             status:  StatusCode::Ok,
//         };
//
//     }
// }

pub struct RequestHandler {
    pub version: String,
    pub description: String,
    pub root_dir: String,

    pub response_status: StatusCode,
}

struct Resource {
    data: String,
    size: u64,
    modified: SystemTime,
    content_type: String,
}

impl RequestHandler {
    pub fn process_request(&mut self, req: &Request) {
        match req.method {
            Method::Get => {
                let (code, res) = self.get_resource(&req.resource);
                self.response_status = code;
                match code {
                    StatusCode::Ok => {
                        self.assemble_header(req, res);

                        let mut data = String::new();
                        if let Some(resource) = res {
                            resource.read_to_string(&mut data);
                        }
                        println!("resource: {}", data);
                    }
                    _ => println!("error code: {}", code as u16),
                }
            }
            Method::Head => {
                let (code, res) = self.get_resource(&req.resource);
            }
            _ => {}
        };
    }

    fn assemble_header(&self, request: &Request, resource: &Resource) -> HashMap<String, String> {
        let mut headers: HashMap<String, String> = HashMap::new();

        let timestamp = Utc::now().format("%a, %d %b %Y %H:%M:%S GMT").to_string();
        println!("{}", timestamp);
        headers.insert("Date".to_string(), timestamp);

        match headers.get("Connection") {
            Some(c) => {
                headers.insert("Connection".to_string(), c.to_owned());
            }
            None => {
                headers.insert("Connection".to_string(), "close".to_string());
            }
        }

        match request.method {
            Method::Get | Method::Head => {
                if let Some(res) = resource {
                    let mtime = res.metadata().unwrap().mtime();
                    println!("{}", mtime);
                }
            }
            _ => {}
        }

        headers
    }

    fn get_resource(&self, resource: &str) -> Result<(StatusCode, Resource), StatusCode> {
        let path = Path::new(&self.root_dir).join(resource);
        println!("fetching resource at {}", path.display());
        match File::open(path) {
            Ok(file) => {
                // TODO: these can return errors depending on the platform,
                // but will run fine on Unix... maybe handle things more
                // gracefully instead of unwrapping everything?
                let metadata = file.metadata().unwrap();
                let size = metadata.len();
                let modified = metadata.modified().unwrap();
                // TODO: implement content_type detection properly
                let content_type = String::from("text/html");
                let data = String::new();
                // TODO: check if it's a directory
                file.read_to_string(&mut data);
                Ok((
                    StatusCode::Ok,
                    Resource {
                        data,
                        size,
                        modified,
                        content_type,
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
