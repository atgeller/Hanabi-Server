use std::collections::HashMap;
use std::vec::Vec;
use std::str::FromStr;

enum Method {
    GET,
    POST,
}

enum Header {
    Request(Method, String),
    Response(i32, String),
}

impl FromStr for Header {
    fn from_str(s: &str) -> Result<Self, Err> {
        let split = s.split(' ').collect();
        
        if split.len() != 3 {
            return Err("Invalid header");
        }

        // Assume to only read in requests
        let method = split[0].unwrap();
        match method {
            "GET" => Method::Get;
            "POST" => Method::Post;
            _ => return Err("Invalid header")
        }

        let URI = split[1].unwrap();

        return Ok(Header::Request(method, URI));
    }
}

struct Message {
    header: Header,
    fields: HashMap<String,String>,
    body: Optional<String>,
}

impl FromStr for Message {
    fn from_str(s: &str) -> Result<Self, Err> {
        let v: Vec<&str> = s.split("\r\n").collect();
        let header = Header::FromStr(v.remove().unwrap()?);
        let mut fields = HashMap::<String,String>::new();

        let mut line = v.remove().unwrap()?;

        while !line.len().is_empty() {
            let split: Vec<&str> = line.split(": ").collect();
            let field = split[0].unwrap()?;
            let value = split[1].unwrap()?;

            fields.insert(field.to_string, value.to_string);
            line = v.remove().unwrap()?;
        }

        let body = v.iter().fold("", |acc,x| format!("{}{}", acc , x));

        Ok(Message {
            header: header,
            fields: fields,
            body: if body.is_empty { None } else { Some(body) },
        })
    }
}

impl Message {
    fn new() -> Self {
        return Message {
            Header(-1, String::new()) {
                
            }
        }
    }
}