mod http;
use http::parser::*;
use http::protocol::{Method, Request, RequestHandler, StatusCode};
use std::collections::HashMap;
use std::str;

use std::io::{prelude, Read};
use std::net::{TcpListener, TcpStream};

fn echo(mut stream: TcpStream) {
    let mut buf = String::new();
    stream.read_to_string(&mut buf).unwrap();
    println! {"{}\n", buf};
}

fn main() -> std::io::Result<()> {
    // let listener = TcpListener::bind("127.0.0.1:8000").unwrap();
    // let mut buf = [0; 1024];
    // for mut stream in listener.incoming() {
    //     echo(stream?);
    // }

    let mut server = RequestHandler {
        version: "HTTP/1.1".to_string(),
        description: "HTTP Server v0.1".to_string(),
        root_dir: "/home/nuke/http-server/tests/root_dir/".to_string(),
        response_status: StatusCode::Ok,
    };

    let paths = std::fs::read_dir("tests").unwrap();
    for path in paths {
        let path = path.unwrap();
        if path.path().is_dir() {
            continue;
        }
        println!("*** {} ***\n", path.path().display());

        let buf = std::fs::read(path.path()).unwrap();
        let req = str::from_utf8(&buf).unwrap();
        match parse_request(req) {
            Ok((_, request)) => {
                println!("{:?}", request);
                let response = server.process_request(&request);
                println!("{:?}", response);
            }

            Err(e) => {
                println!("{:?}", e);
            }
        }
    }

    // let request = Request {
    //     method: Method::Get,
    //     resource: "index.html".to_string(),
    //     version: "HTTP/1.1".to_string(),
    //     headers: HashMap::new(),
    // };

    // for (name, field) in response.headers {
    //     println!("{}: {}", name, field);
    // }

    Ok(())
}
