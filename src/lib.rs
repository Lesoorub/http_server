#![allow(dead_code)]

use std::{
    net::{
        TcpListener, TcpStream, Shutdown
    },
    io::{Write, BufReader, BufWriter, Read},
    str,
    collections::BTreeMap
};

const HTTP_VERSION: &'static str = "HTTP/1.1";
const SERVER_NAME: &'static str = "Rust/1.0.0 (Windows)";

const OK : ResponseCode = ResponseCode {
    code: 200,
    description: "Ok",
};
const NOT_FOUND : ResponseCode = ResponseCode {
    code: 404,
    description: "Not found",
};
const NOT_FOUND_BODY : &[u8] = b"<html><body>404</body></html>";

pub struct Server{
    listener: TcpListener,
    routes: BTreeMap<&'static str, fn(&mut Request) -> Response>,
    not_found: fn(&mut Request) -> Response,
}

impl Server {
    pub fn listen(ip: &String, port: &i16) -> Server {
        Server {
            listener: TcpListener::bind(format!("{}:{}", ip, port)).unwrap(),
            routes: BTreeMap::new(),
            not_found: |_| -> Response { bytes(NOT_FOUND_BODY.to_vec()) },
        }
    }
    pub fn start(&self) {
        for con in self.listener.incoming()
        {
            let stream = con.unwrap();
            if !self.handle_connection(&stream)
            {
                Self::send_str(&stream, "500 Internal Server Error");
            }
        }
    }
    fn read_stream_to_end(stream : &TcpStream) -> Vec<u8> {
        let mut reader = BufReader::with_capacity(128, stream);
        let mut data = Vec::<[u8; 128]>::new();
        let mut bytes_readed;
        const BUFLEN : usize = 128;
        loop {
            let mut buf : [u8; BUFLEN] = [b' '; BUFLEN];
            bytes_readed = reader.read(&mut buf).unwrap();
            data.push(buf);
            if bytes_readed != BUFLEN
            {
                break;
            }
        }
        let mut result = Vec::<u8>::new();
        for i in 0..data.len() - 1 {
            result.append(&mut data[i].to_vec());
        }
        result.append(&mut data[data.len() - 1][..bytes_readed].to_vec());
        result
    }
    fn handle_connection(&self, stream : &TcpStream) -> bool {
        let recieved = Self::read_stream_to_end(stream);

        let mut req = Request::new(String::from(match str::from_utf8(&recieved) {
            Ok(s) => s,
            Err(_) => return false,
        }));
        
        println!("->{}", req.to_string());
        
        match self.routes.get(req.path.as_str()) {
            Some(action) => {
                let result = &action(&mut req);
                self.send_with_code(
                    &stream, 
                    &result.code, 
                    &result.data, 
                    &result.headers);
            },
            None =>
                self.send_with_code(
                    &stream, 
                    &NOT_FOUND, 
                    &(self.not_found)(&mut req).data, 
                    &BTreeMap::new()),
        }

        match stream.shutdown(Shutdown::Both) {
            Ok(_) => return true,
            Err(_) => return false,
        };
    }
    fn send_with_code(
        &self, 
        stream: &TcpStream, 
        code: &ResponseCode, 
        data: &[u8], 
        headers: &BTreeMap<String, String>) {
        
        let mut headerssum = Vec::<String>::new();
        headerssum.push(format!("{}: {}\r\n", "Server", SERVER_NAME));
        for pair in headers {
            headerssum.push(format!("{}: {}\r\n", pair.0, pair.1));
        }

        let data = [ format!("{} {} {}\r\n{}\r\n\r\n", HTTP_VERSION, 
            code.code, code.description, headerssum.concat()).as_bytes(), data ].concat();

        //println!("<-{}", String::from_utf8(data.clone()).unwrap());
        
        Self::send_bytes(stream, &data);
    }
    fn send_str(stream : &TcpStream, data: &'static str) {
        Self::send_bytes(stream, data.as_bytes());
    }
    fn send_bytes(stream : &TcpStream, data: &[u8]) {
        let mut writer = BufWriter::new(stream);
        writer.write_all(data).unwrap();
        writer.flush().unwrap();
    }
    pub fn route(&mut self, selfpath: &'static str, action: fn(&mut Request) -> Response) {
        self.routes.insert(selfpath, action);
    }
    pub fn route404(&mut self, action: fn(&mut Request) -> Response) {
        self.not_found = action;
    }
}

#[derive(Debug)]
pub struct Request {
    pub http_version: String,
    pub method: String,
    pub path: String,
    pub headers: BTreeMap<String, String>
}

impl Request {
    pub fn new(raw: String) -> Request {
        let mut lines_enumerate = raw.split("\r\n").enumerate();
        let mut protocol_info_enumerate = lines_enumerate.next().unwrap().1.split(" ").enumerate();
        let mut headers = BTreeMap::<String, String>::new();
        for (_, line) in lines_enumerate {
            if line == "" {
                break;
            }
            let mut pair_enumerate = line.split(": ").enumerate();
            headers.insert(
                String::from(pair_enumerate.next().unwrap().1), 
                String::from(pair_enumerate.next().unwrap().1));
        }
    
        Request {
            method: String::from(protocol_info_enumerate.next().unwrap().1),
            path: String::from(protocol_info_enumerate.next().unwrap().1),
            http_version: String::from(protocol_info_enumerate.next().unwrap().1),
            headers: headers
        }
    }
    pub fn to_string(&mut self) -> String
    {
        String::from(format!("{} {}", self.method, self.path))
    }
}

struct ResponseCode{
    code: i32,
    description: &'static str,
}

fn _file(path: &'static str) -> Vec<u8> {
    use std::fs::File;

    let mut fs = match File::open(path) {
        Ok(f) => f,
        Err(_) => { 
            println!("Can't open file at path '{}'", path); 
            return Vec::new(); 
        },
    };
    let mut buffer = Vec::new();
    match fs.read_to_end(&mut buffer) {
        Ok(_) => return buffer,
        Err(_) => {
            println!("Can't read file at path '{}'", path); 
            return Vec::new();
        },
    };
}

pub fn file(path: &'static str) -> Response {
    bytes(_file(path))
}

pub fn bytes(data: Vec<u8>) -> Response {
    Response { 
        data: data,
        code: OK,
        headers: BTreeMap::new(),
    }
}

pub fn str(str: &'static str) -> Response {
    Response { 
        data: str.as_bytes().to_vec(),
        code: OK,
        headers: BTreeMap::new(),
    }
}

pub fn err(code: i32, description: &'static str) -> Response {
    Response { 
        data: Vec::new(),
        code: ResponseCode { 
            code: code, 
            description: description,
        },
        headers: BTreeMap::new()
    }
}

pub fn err_with_headers(code: i32, description: &'static str, headers: BTreeMap<String, String>) -> Response {
    Response { 
        data: Vec::new(),
        code: ResponseCode { 
            code: code, 
            description: description,
        },
        headers: headers
    }
}

pub fn redirect(newpath: &'static str)-> Response {
    let mut headers = BTreeMap::<String, String>::new();
    headers.insert(String::from("Location"), String::from(newpath));
    err_with_headers(301, "Moved Permanently", headers)
}

pub struct Response {
    data: Vec<u8>,
    code: ResponseCode,
    headers: BTreeMap<String, String>,
}