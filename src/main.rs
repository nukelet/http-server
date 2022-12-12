mod http;
use http::parser::*;
use http::server::Server;

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

use std::fs::File;

#[allow(dead_code)]
fn listen(mut stream: TcpStream, server: &mut Server) {
    let mut buf = String::new();
    stream.read_to_string(&mut buf).unwrap();
    print!("{}", buf);
    let mut file = File::create("output.txt").unwrap();
    file.write_all(buf.as_bytes()).unwrap();
    match parse_request(&buf) {
        Ok((_, request)) => {
            // println!("{:?}", request),
            let result = server.process_request_str(&buf).unwrap();
            println!("{}", result);

        },
        Err(e) => println!("{:?}", e),
    }
}

fn main() -> std::io::Result<()> {
    // let listener = TcpListener::bind("127.0.0.1:8000").unwrap();
    // let mut buf = [0; 1024];
    // for mut stream in listener.incoming() {
    //     listen(stream?, &mut server);
    // }

    let root_dir = "webspace";
    let mut server = Server::new(root_dir);

    let tests_dir = "tests/requests";
    let dir_entries = std::fs::read_dir(tests_dir).unwrap();
    for entry in dir_entries {
        let path = entry.unwrap().path();
        if path.is_dir() {
            continue;
        }
        println!("*** {} ***\n", path.display());

        match server.process_request_file(&path) {
            Ok(response) => {
                println!("{}", response);
            },
            Err(e) => {
                println!("{}", e);
            }
        }
    }

    Ok(())
}
