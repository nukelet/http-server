use nom::{
    IResult,
    bytes::complete::{
        tag,
        take_while1, take_until1, take_until,
        is_not,
    },
    branch::alt,
    sequence::{preceded, tuple},
};

// use std::str;
use crate::http::protocol::{Method, Request};
use std::collections::HashMap;

fn whitespace(i: &str) -> IResult<&str, &str> {
    tag(" ")(i)
}

fn line_ending(i: &str) -> IResult<&str, &str> {
    tag("\r\n")(i)
}

pub fn method(i: &str) -> IResult<&str, &str> {
    alt((
        tag("GET"),
        tag("HEAD"),
        tag("TRACE"),
        tag("OPTIONS"),
        ))(i)
    
    // println!("parsing: {}", i);
    // match is_not(" ")(i) {
    //     "GET" => Ok((i, Method::Get)),
    //     "HEAD" => Ok((i, Method::Head)),
    //     "TRACE" => Ok((i, Method::Trace)),
    //     "OPTION" => Ok((i, Method::Options)),
    //     _ => Err(NomErr::Error(make_error(i, ErrorKind::Tag))),
    // }
}

fn http_version(i: &str) -> IResult<&str, &str> {
    let is_version = |c| ('0'..='9').contains(&c) || (c == '.');
    let version = take_while1(is_version);

    preceded(tag("HTTP/"), version)(i)
}

fn resource(i: &str) -> IResult<&str, &str> {
    // TODO: maybe make this parse absolute paths
    // or URLs properly
    is_not(" ")(i)
}

fn colon(i: &str) -> IResult<&str, &str> {
    tag(":")(i)
}

fn header_name(i: &str) -> IResult<&str, &str> {
    take_until1(":")(i)
}

fn header_field(i: &str) -> IResult<&str, &str> {
    take_until("\r\n")(i)
}

pub fn parse_request(input: &str) -> IResult<&str, Request> {
    // parse request line
    let (mut pos, (method, _, resource, _, version, _)) = 
        tuple((method, whitespace, resource,
              whitespace, http_version, line_ending))(input)?;

    // TODO: make the resource parser return an HttpMethod
    let request_method = match method {
        "GET" => Method::Get,
        "HEAD" => Method::Head,
        "OPTIONS" => Method::Options,
        "TRACE" => Method::Trace,
        _ => Method::Get,
    };
    
    let mut request = Request {
            method: request_method,
            resource: resource.to_string(),
            version: version.to_string(),
            headers: HashMap::new(),
    };

    while !pos.is_empty() {
        let (s, (name, _, field, _)) =
            tuple((header_name, colon, header_field, line_ending))(pos)?;

        request.headers
            .insert(name.to_string(), field.to_string());

        println!("inserting ({}, {})", name, field);

        pos = s;
    }

    Ok((input, request))
}
