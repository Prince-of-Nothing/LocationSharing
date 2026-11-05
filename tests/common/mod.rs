//! Minimal recorded HTTP/1.1 server for contract tests.
//!
//! Deliberately dependency-free: the tests need to observe exactly which
//! headers and body fields `ApiClient` puts on the wire (cookie + CSRF
//! double-submit per the Go handlers), not to simulate a full server.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct Recorded {
    pub method: String,
    pub path: String,
    /// Lowercased header names as received.
    pub headers: Vec<(String, String)>,
    pub body: String,
}

impl Recorded {
    pub fn header(&self, name: &str) -> Option<&str> {
        let name = name.to_ascii_lowercase();
        self.headers
            .iter()
            .find(|(k, _)| *k == name)
            .map(|(_, v)| v.as_str())
    }
}

#[derive(Debug, Clone)]
pub struct MockResponse {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: String,
}

impl MockResponse {
    pub fn json(status: u16, body: &str) -> Self {
        Self {
            status,
            headers: vec![("Content-Type".to_string(), "application/json".to_string())],
            body: body.to_string(),
        }
    }

    /// Adds a `Set-Cookie` header (may be called repeatedly).
    pub fn with_cookie(mut self, cookie: &str) -> Self {
        self.headers
            .push(("Set-Cookie".to_string(), cookie.to_string()));
        self
    }

    /// Adds a raw response header.
    pub fn with_header(mut self, name: &str, value: &str) -> Self {
        self.headers.push((name.to_string(), value.to_string()));
        self
    }
}

pub struct MockServer {
    pub base_url: String,
    recorded: Arc<Mutex<Vec<Recorded>>>,
}

impl MockServer {
    /// Serves requests from a FIFO of responses; the last response is
    /// repeated once the queue is exhausted.
    pub fn start(responses: Vec<MockResponse>) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind mock server");
        let addr = listener.local_addr().expect("local address");
        let recorded: Arc<Mutex<Vec<Recorded>>> = Arc::new(Mutex::new(Vec::new()));
        let queue = Arc::new(Mutex::new(responses));

        let rec = Arc::clone(&recorded);
        std::thread::spawn(move || {
            for stream in listener.incoming() {
                let Ok(stream) = stream else { continue };
                handle(stream, Arc::clone(&rec), Arc::clone(&queue));
            }
        });

        Self {
            base_url: format!("http://{addr}"),
            recorded,
        }
    }

    pub fn requests(&self) -> Vec<Recorded> {
        self.recorded.lock().unwrap().clone()
    }
}

fn handle(
    mut stream: TcpStream,
    rec: Arc<Mutex<Vec<Recorded>>>,
    queue: Arc<Mutex<Vec<MockResponse>>>,
) {
    let mut reader = BufReader::new(match stream.try_clone() {
        Ok(s) => s,
        Err(_) => return,
    });

    let mut request_line = String::new();
    if reader.read_line(&mut request_line).is_err() || request_line.is_empty() {
        return;
    }
    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or_default().to_string();
    let path = parts.next().unwrap_or_default().to_string();

    let mut headers: Vec<(String, String)> = Vec::new();
    let mut content_length = 0usize;
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line).is_err() {
            return;
        }
        let line = line.trim_end();
        if line.is_empty() {
            break;
        }
        if let Some((name, value)) = line.split_once(':') {
            let name = name.trim().to_ascii_lowercase();
            let value = value.trim().to_string();
            if name == "content-length" {
                content_length = value.parse().unwrap_or(0);
            }
            headers.push((name, value));
        }
    }

    let mut body_bytes = vec![0u8; content_length];
    if content_length > 0 && reader.read_exact(&mut body_bytes).is_err() {
        return;
    }
    let body = String::from_utf8_lossy(&body_bytes).to_string();

    rec.lock().unwrap().push(Recorded {
        method,
        path,
        headers,
        body,
    });

    let response = {
        let mut q = queue.lock().unwrap();
        if q.len() > 1 {
            q.remove(0)
        } else {
            q.first()
                .cloned()
                .unwrap_or_else(|| MockResponse::json(200, "{}"))
        }
    };

    let reason = match response.status {
        200 => "OK",
        201 => "Created",
        400 => "Bad Request",
        401 => "Unauthorized",
        403 => "Forbidden",
        422 => "Unprocessable Entity",
        500 => "Internal Server Error",
        _ => "Status",
    };

    let mut out = format!("HTTP/1.1 {} {}\r\n", response.status, reason);
    for (name, value) in &response.headers {
        out.push_str(&format!("{name}: {value}\r\n"));
    }
    out.push_str(&format!("Content-Length: {}\r\n", response.body.len()));
    out.push_str("Connection: close\r\n\r\n");
    out.push_str(&response.body);

    let _ = stream.write_all(out.as_bytes());
    let _ = stream.flush();
}
