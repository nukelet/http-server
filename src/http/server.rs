use crate::http::parser;
use crate::http::protocol::{RequestHandler, StatusCode};

use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::str;

use sha2::{Digest, Sha256};
use base64ct::{Base64, Encoding};

use itertools::Itertools;

pub struct AuthManager {
    root_path: PathBuf,
    salt: String,
    user: String,
    pwd: String,
}

impl AuthManager {
    pub fn new(root_path: &str, user: &str, pwd: &str) -> AuthManager {
        AuthManager {
            root_path: PathBuf::from(root_path),
            salt: "EA".to_string(),
            user: user.to_string(),
            pwd: pwd.to_string(),
        }
    }

    pub fn has_permission(&self, resource_path: &Path) -> bool {
        let mut hasher = Sha256::new();
        let salted = format!("{}{}", self.pwd, self.salt);
        hasher.update(salted);
        let hash = hasher.finalize();
        let hash_b64 = Base64::encode_string(&hash);
        println!("AuthManager: {}:{}", self.user, hash_b64);
        println!("AuthManager: requested resource \"{}\"", resource_path.display());
        
        match self.get_access_config(resource_path) {
            Some(path) => {
                let table = self.get_auth_table(&path);
                println!("table: {:?}", table);
                // Check if the user is in the table and if the salted hash is correct
                match table.get(self.user.as_str()) {
                    Some(h) => {
                        if h.eq(&hash_b64) {
                            println!("AuthManager: auth success");
                            true
                        } else {
                            println!("AuthManager: auth failure");
                            false
                        }
                    },
                    None => {
                        println!("AuthManager: auth failure");
                        false
                    },
                }
            },
            None => true
        }
    }

    pub fn get_auth_table(&self, config_path: &Path) -> HashMap<String, String> {
        let file = File::open(config_path).unwrap();
        let reader = BufReader::new(file);
        let mut table = HashMap::new();
        for line in reader.lines() {
            if let Some((user, hash)) = line.unwrap().split(':').collect_tuple() {
                table.insert(user.to_string(), hash.to_string()); 
            }
        }

        table
    }

    pub fn get_access_config(&self, resource_path: &Path) -> Option<PathBuf> {
        let mut path = PathBuf::from(resource_path);
        if path.is_file() {
            path.pop();
        }
        println!("AuthManager: looking for config in {}", path.display());

        while path != self.root_path {
            println!("AuthManager: currently on {:?}", path);
            if path.join(".htaccess").exists() {
                println!("AuthManager: found .htaccess in {}", path.display());
                return Some(path.join(".htaccess"));
            }
            path.pop();
        }

        println!("AuthManager: could not find .htaccess");
        None
    }
}

pub struct SessionManager {
    request_handler: RequestHandler,
}

#[allow(dead_code)]
impl SessionManager {
    pub fn new(root_path: &str, user: &str, pwd: &str) -> SessionManager {
        SessionManager {
            request_handler: RequestHandler {
                version: "HTTP/1.1".to_string(),
                description: "HTTP Server v0.1".to_string(),
                root_dir: root_path.to_string(),
                response_status: StatusCode::Ok,
                auth_manager: AuthManager::new(root_path, user, pwd),
            },
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
