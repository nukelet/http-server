mod http;
use http::parser::*;
use http::protocol::{Method, Request, RequestHandler, StatusCode};
use http::server::Server;
use std::collections::HashMap;
use std::path::Path;
use std::str;

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

use std::fs;
use std::fs::File;

fn listen(mut stream: TcpStream, server: &mut Server) {
    let mut buf = String::new();
    stream.read_to_string(&mut buf).unwrap();
    print!("{}", buf);
    let mut file = File::create("output.txt").unwrap();
    file.write(buf.as_bytes());
    match parse_request(&buf) {
        Ok((_, request)) => {
            // println!("{:?}", request),
            let result = server.process_from_str(&buf).unwrap();
            println!("{}", result);

        },
        Err(e) => println!("{:?}", e),
    }
}

fn main() -> std::io::Result<()> {
    let root_path = "/home/nuke/faculdade/http-server/tests/root_dir/";
    let mut server = Server::new(root_path);
    // let result = server.process_from_file("tests/forbidden.txt")?;
    // println!("{}", result);

    let listener = TcpListener::bind("127.0.0.1:8000").unwrap();
    let mut buf = [0; 1024];
    for mut stream in listener.incoming() {
        listen(stream?, &mut server);
    }

    // let mut server = RequestHandler {
    //     version: "HTTP/1.1".to_string(),
    //     description: "HTTP Server v0.1".to_string(),
    //     root_dir: "/home/nuke/faculdade/http-server/tests/root_dir/".to_string(),
    //     response_status: StatusCode::Ok,
    // };

    // let path = Path::new("/home/nuke/http-server/tests/root_dir/").join("/index.html");
    // println!("{}", path.display());
    // let paths = std::fs::read_dir("tests").unwrap();
    // for path in paths {
    //     let path = path.unwrap();
    //     if path.path().is_dir() {
    //         continue;
    //     }
    //     println!("*** {} ***\n", path.path().display());
    //
    //     let buf = std::fs::read(path.path()).unwrap();
    //     let req = str::from_utf8(&buf).unwrap();
    //     match parse_request(req) {
    //         Ok((_, request)) => {
    //             println!("{:?}", request);
    //             let response = server.process_request(&request);
    //             println!("{:?}", response);
    //         }
    //
    //         Err(e) => {
    //             println!("{:?}", e);
    //         }
    //     }
    // }

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
