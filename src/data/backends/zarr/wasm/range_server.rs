//! A local HTTP server for tests that answers `HEAD` with the size and range
//! requests (`bytes=a-b`) with `206`, `404` for unknown paths, and counts the
//! body bytes it sends and the requests it answers.

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

pub struct RangeServer {
    pub base: String,
    sent: Arc<AtomicU64>,
    requests: Arc<AtomicU64>,
}

impl RangeServer {
    /// Serves `objects` (path without the leading `/` -> bytes) until the
    /// test process ends.
    pub fn start(objects: HashMap<String, Vec<u8>>) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
        let base = format!("http://{}", listener.local_addr().expect("addr"));
        let (sent, requests) = (Arc::new(AtomicU64::new(0)), Arc::new(AtomicU64::new(0)));
        let (bytes, count) = (sent.clone(), requests.clone());
        std::thread::spawn(move || {
            for stream in listener.incoming().flatten() {
                count.fetch_add(1, Ordering::SeqCst);
                answer(stream, &objects, &bytes);
            }
        });
        Self {
            base,
            sent,
            requests,
        }
    }

    /// Body bytes sent so far.
    pub fn sent(&self) -> u64 {
        self.sent.load(Ordering::SeqCst)
    }

    /// Requests answered so far.
    pub fn requests(&self) -> u64 {
        self.requests.load(Ordering::SeqCst)
    }
}

fn answer(mut stream: TcpStream, objects: &HashMap<String, Vec<u8>>, sent: &AtomicU64) {
    let mut reader = BufReader::new(stream.try_clone().expect("clone"));
    let (mut request, mut range, mut line) = (String::new(), None, String::new());
    while reader.read_line(&mut line).is_ok_and(|n| n > 0) && line != "\r\n" {
        if request.is_empty() {
            request = line.clone();
        } else if let Some(v) = line.to_ascii_lowercase().strip_prefix("range: bytes=") {
            range = Some(v.trim().to_string());
        }
        line.clear();
    }
    let mut words = request.split(' ');
    let head = words.next() == Some("HEAD");
    let path = words.next().unwrap_or("").trim_start_matches('/');
    let Some(object) = objects.get(path) else {
        let _ = stream
            .write_all(b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n");
        return;
    };
    let total = object.len() as u64;
    if head {
        let reply =
            format!("HTTP/1.1 200 OK\r\nContent-Length: {total}\r\nConnection: close\r\n\r\n");
        let _ = stream.write_all(reply.as_bytes());
        return;
    }
    let (start, end) = match range.as_deref().and_then(|r| r.split_once('-')) {
        Some((a, b)) => (
            a.parse().unwrap_or(0),
            b.parse::<u64>().map_or(total, |b| b + 1),
        ),
        None => (0, total),
    };
    let end = end.min(total);
    let body = &object[start as usize..end as usize];
    let reply = format!(
        "HTTP/1.1 206 Partial Content\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );
    // Counted before the client can read the body, so a test reading `sent`
    // right after its fetch sees every byte.
    sent.fetch_add(body.len() as u64, Ordering::SeqCst);
    let _ = stream.write_all(reply.as_bytes());
    let _ = stream.write_all(body);
}
