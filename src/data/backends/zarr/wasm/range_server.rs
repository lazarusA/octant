//! A local HTTP server for tests that answers range requests (`bytes=a-b`,
//! `bytes=-n`) with `206` and `Content-Range`, `404` for unknown paths, and
//! counts the body bytes it sends.

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

pub struct RangeServer {
    pub base: String,
    sent: Arc<AtomicU64>,
}

impl RangeServer {
    /// Serves `objects` (path without the leading `/` -> bytes) until the
    /// test process ends.
    pub fn start(objects: HashMap<String, Vec<u8>>) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
        let base = format!("http://{}", listener.local_addr().expect("addr"));
        let sent = Arc::new(AtomicU64::new(0));
        let counter = sent.clone();
        std::thread::spawn(move || {
            for stream in listener.incoming().flatten() {
                answer(stream, &objects, &counter);
            }
        });
        Self { base, sent }
    }

    /// Body bytes sent so far.
    pub fn sent(&self) -> u64 {
        self.sent.load(Ordering::SeqCst)
    }
}

fn answer(mut stream: TcpStream, objects: &HashMap<String, Vec<u8>>, sent: &AtomicU64) {
    let mut reader = BufReader::new(stream.try_clone().expect("clone"));
    let mut path = String::new();
    let mut range = None;
    let mut line = String::new();
    while reader.read_line(&mut line).is_ok_and(|n| n > 0) && line != "\r\n" {
        if let Some(rest) = line.strip_prefix("GET ") {
            path = rest
                .split(' ')
                .next()
                .unwrap_or("")
                .trim_start_matches('/')
                .into();
        } else if let Some(v) = line.to_ascii_lowercase().strip_prefix("range: bytes=") {
            range = Some(v.trim().to_string());
        }
        line.clear();
    }
    let Some(object) = objects.get(&path) else {
        let _ = stream
            .write_all(b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n");
        return;
    };
    let total = object.len() as u64;
    let (start, end) = match range.as_deref().and_then(|r| r.split_once('-')) {
        Some(("", n)) => (total.saturating_sub(n.parse().unwrap_or(0)), total),
        Some((a, b)) => (
            a.parse().unwrap_or(0),
            b.parse::<u64>().map_or(total, |b| b + 1),
        ),
        None => (0, total),
    };
    let end = end.min(total);
    let body = &object[start as usize..end as usize];
    let head = format!(
        "HTTP/1.1 206 Partial Content\r\nContent-Length: {}\r\nContent-Range: bytes {start}-{}/{total}\r\nConnection: close\r\n\r\n",
        body.len(),
        end.saturating_sub(1)
    );
    let _ = stream.write_all(head.as_bytes());
    let _ = stream.write_all(body);
    sent.fetch_add(body.len() as u64, Ordering::SeqCst);
}
