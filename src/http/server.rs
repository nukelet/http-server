use crate::http::parser;
use crate::http::protocol::{RequestHandler, StatusCode};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use std::str;

pub struct AuthManager {
    root_path: PathBuf,
    salt: String,
}

impl AuthManager {
    pub fn new(root_path: &str) -> AuthManager {
        AuthManager {
            root_path: PathBuf::from(root_path),
            salt: "EA".to_string(),
        }
    }

    pub fn has_permission(&self, user: &str, pwd: &str, resource_path: &Path) {
        let mut hasher = Sha256::new();
        let salted = format!("{}{}", pwd, self.salt);
        hasher.update(salted);
        let hash = hasher.finalize();

        let mut path = PathBuf::from(&self.root_path).join(resource_path);
        if path.is_file() {
            path.pop();
        }

        while path != self.root_path {
            println!("AuthManager: currently on {:?}", path);
            if fs::read_dir(path).contains(&".htaccess".to_string()) {
                break;
            }
            path.pop();
        }
    }
}

pub struct SessionManager {
    request_handler: RequestHandler,
    auth_manager: AuthManager,
}

#[allow(dead_code)]
impl SessionManager {
    pub fn new(root_path: &str) -> SessionManager {
        SessionManager {
            request_handler: RequestHandler {
                version: "HTTP/1.1".to_string(),
                description: "HTTP Server v0.1".to_string(),
                root_dir: root_path.to_string(),
                response_status: StatusCode::Ok,
            },

            auth_manager: AuthManager::new(root_path),
        }
    }

    pub fn process_request_str(&mut self, input: &str) -> std::io::Result<Vec<u8>> {
        let response = match parser::parse_request(input) {
            Ok((_, request)) => self.request_handler.process_request(&request),
            Err(_) => self.request_handler.bad_request(),
        };

        let mut s: Vec<u8> = vec![];

        s.extend_from_slice("HTTP/1.1 ".as_bytes());
        s.extend_from_slice(response.status.as_str().as_bytes());
        s.extend_from_slice("\r\n".as_bytes());

        for (name, content) in response.headers {
            s.extend_from_slice(format!("{}: {}\r\n", name, content).as_bytes());
        }

        s.extend_from_slice("\r\n".as_bytes());
        s.extend(response.message.clone());
        s.extend_from_slice("\r\n".as_bytes());

        Ok(s)
    }

    pub fn process_request_file(&mut self, path: &Path) -> std::io::Result<Vec<u8>> {
        let file = fs::read(path)?;
        // TODO: we need to handle this (in case there are
        // non-utf8 characters in the file)
        let input = str::from_utf8(&file).unwrap();
        self.process_request_str(input)
    }

    pub fn process_request_buf(&mut self, buf: &[u8]) -> std::io::Result<Vec<u8>> {
        // TODO: handle non-utf8 buffer
        self.process_request_str(std::str::from_utf8(buf).unwrap())
    }
}

pub struct Server {}
