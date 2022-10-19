mod http;
use http::parser::*;
use http::protocol::{Server, Request, Method, StatusCode};
use std::collections::HashMap;
use std::str;



fn main() {
    // let paths = std::fs::read_dir("tests").unwrap();
    // for path in paths {
    //     let path = path.unwrap();
    //     println!("*** {} ***\n", path.path().display());
    //
    //     let buf = std::fs::read(path.path()).unwrap();
    //     let req = str::from_utf8(&buf).unwrap();
    //     match parse_request(req) {
    //         Ok((_, request)) => println!("{:?}", request),
    //         Err(e) => {
    //             println!("{:?}", e);
    //         },
    //     }
    // }


    let mut server = Server{
        version: "HTTP/1.1".to_string(),
        description: "HTTP Server v0.1".to_string(),
        root_dir: "tests/root_dir/".to_string(),
        response_status: StatusCode::Ok,
    };

    let request = Request {
        method: Method::Get,
        resource: "resource_without_read_perms.html".to_string(),
        version: "HTTP/1.1".to_string(),
        headers: HashMap::new(),
    };

    server.process_request(&request);
}
