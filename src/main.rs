mod http;
use http::parser::*;
use http::server::Server;

use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};

use std::fs::File;

#[allow(dead_code)]
fn listen(mut stream: TcpStream, server: &mut Server) {
    println!("Starting new connection");
    stream.set_nonblocking(true).unwrap();
    let mut buf = [0; 1024];
    let mut data: Vec<u8> = vec![];
    let mut reader = BufReader::new(stream.try_clone().unwrap());
    loop {
        match reader.read(&mut buf) {
            Ok(_) => {
                print!("GOT:\n{}", String::from_utf8_lossy(&buf));
                data.extend_from_slice(&buf);
            }
            Err(e) => {
                println!("{}", e);
                break;
            }
        }
    }
    // stream.read_to_string(&mut buf).unwrap();
    let mut string_buf = String::from_utf8_lossy(&data);
    print!("{}", string_buf);
    // let mut file = File::create("output.txt").unwrap();
    // file.write_all(buf.as_bytes()).unwrap();
    match parse_request(&string_buf) {
        Ok((_, request)) => {
            // println!("{:?}", request),
            let result = server.process_request_str(&string_buf).unwrap();
            println!("{}", String::from_utf8_lossy(&result));
            stream.write(&result).unwrap();
        }
        Err(e) => println!("{:?}", e),
    }
}

fn main() -> std::io::Result<()> {
    let root_dir = "webspace";
    let mut server = Server::new(root_dir);

    let listener = TcpListener::bind("127.0.0.1:8000").unwrap();
    let mut buf = [0; 1024];
    for mut stream in listener.incoming() {
        listen(stream?, &mut server);
    }
    // let tests_dir = "tests/requests";
    // let dir_entries = std::fs::read_dir(tests_dir).unwrap();
    // for entry in dir_entries {
    //     let path = entry.unwrap().path();
    //     if path.is_dir() {
    //         continue;
    //     }
    //     println!("*** {} ***\n", path.display());
    //
    //     match server.process_request_file(&path) {
    //         Ok(response) => {
    //             // println!("data: {:?}", response);
    //             println!("{}", String::from_utf8_lossy(&response));
    //         }
    //         Err(e) => {
    //             println!("{}", e);
    //         }
    //     }
    // }

    Ok(())
}
