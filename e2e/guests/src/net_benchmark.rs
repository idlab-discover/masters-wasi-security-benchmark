/// Networking benchmark — exercises `wasi:sockets` host calls via `std::net`.
///
/// Target service: a whoami HTTP endpoint reachable at 192.168.60.22:3006
/// (also available via <https://whoami.blauwhuis.org>).
///
/// Phases:
///   1. DNS resolution            — `wasi:sockets/ip-name-lookup`
///   2. TCP connect/close cycles  — raw socket establishment overhead
///   3. HTTP GET (new conn each)  — full request lifecycle per iteration
///   4. HTTP GET (keep-alive)     — amortised connection cost
///   5. HTTP POST (large body)    — data upload through the sandbox boundary

use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

/// Direct IP:port — bypasses DNS so we can benchmark sockets in isolation.
const TARGET_IP: &str = "192.168.60.22:3006";
/// Hostname:port — used for DNS resolution benchmarks.
const TARGET_HOST: &str = "whoami.blauwhuis.org:3006";
/// HTTP Host header value.
const HTTP_HOST: &str = "whoami.blauwhuis.org";

/// Number of DNS resolution iterations.
const DNS_ITERS: usize = 20;
/// Number of bare TCP connect/disconnect cycles.
const CONNECT_ITERS: usize = 50;
/// Number of HTTP requests with a fresh connection each time.
const REQUEST_ITERS: usize = 50;
/// Number of HTTP requests on a single keep-alive connection.
const KEEPALIVE_ITERS: usize = 100;
/// Size of the HTTP POST body in bytes (64 KiB).
const POST_BODY_SIZE: usize = 65_536;

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

fn main() {
    dns_resolve();
    tcp_connect_cycles();
    http_get_new_connections();
    http_get_keepalive();
    http_post();
}

// ---------------------------------------------------------------------------
// Benchmark phases
// ---------------------------------------------------------------------------

/// Resolve the target hostname repeatedly.
/// Exercises `wasi:sockets/ip-name-lookup` on each iteration.
fn dns_resolve() {
    for _ in 0..DNS_ITERS {
        let addrs: Vec<_> = TARGET_HOST
            .to_socket_addrs()
            .expect("DNS resolution failed")
            .collect();
        assert!(!addrs.is_empty(), "DNS returned zero addresses");
    }
}

/// Open a TCP connection and immediately close it.
/// Measures pure connection-establishment overhead without any I/O.
fn tcp_connect_cycles() {
    for _ in 0..CONNECT_ITERS {
        let stream = TcpStream::connect(TARGET_IP).expect("TCP connect failed");
        drop(stream);
    }
}

/// Issue an HTTP GET on a *new* connection for every iteration.
/// Uses `Connection: close` so the server closes after the response,
/// allowing a simple `read_to_end`.
fn http_get_new_connections() {
    let request = format!(
        "GET / HTTP/1.1\r\nHost: {HTTP_HOST}\r\nConnection: close\r\n\r\n"
    );

    for _ in 0..REQUEST_ITERS {
        let mut stream = TcpStream::connect(TARGET_IP).expect("TCP connect failed");
        stream.write_all(request.as_bytes()).expect("write request");

        let mut response = Vec::new();
        stream.read_to_end(&mut response).expect("read response");
        assert!(!response.is_empty(), "empty response from server");
    }
}

/// Issue HTTP GETs over a single persistent connection.
/// Requires proper HTTP response parsing so we know where one response
/// ends and the next begins.
fn http_get_keepalive() {
    // HTTP/1.1 defaults to keep-alive, so we omit the Connection header.
    let request = format!(
        "GET / HTTP/1.1\r\nHost: {HTTP_HOST}\r\n\r\n"
    );

    let mut stream = TcpStream::connect(TARGET_IP).expect("TCP connect failed");

    for _ in 0..KEEPALIVE_ITERS {
        stream.write_all(request.as_bytes()).expect("write request");
        stream.flush().expect("flush");

        let body_len = read_http_response(&mut stream);
        assert!(body_len > 0, "empty response body on keep-alive connection");
    }
}

/// Send an HTTP POST with a large deterministic body on each iteration.
/// Uses `Connection: close` per request for simplicity.
fn http_post() {
    let body = generate_body(POST_BODY_SIZE);
    let header = format!(
        "POST / HTTP/1.1\r\n\
         Host: {HTTP_HOST}\r\n\
         Content-Length: {POST_BODY_SIZE}\r\n\
         Content-Type: application/octet-stream\r\n\
         Connection: close\r\n\r\n"
    );

    for _ in 0..REQUEST_ITERS {
        let mut stream = TcpStream::connect(TARGET_IP).expect("TCP connect failed");
        stream.write_all(header.as_bytes()).expect("write header");
        stream.write_all(&body).expect("write body");

        let mut response = Vec::new();
        stream.read_to_end(&mut response).expect("read response");
        assert!(!response.is_empty(), "empty POST response");
    }
}

// ---------------------------------------------------------------------------
// HTTP response reader (for keep-alive connections)
// ---------------------------------------------------------------------------

/// Read a single HTTP response from a persistent connection.
///
/// Handles both `Content-Length` and `Transfer-Encoding: chunked` framing.
/// Returns the number of body bytes consumed.
fn read_http_response(stream: &mut TcpStream) -> usize {
    let mut buf = Vec::with_capacity(4096);
    let mut tmp = [0u8; 1024];

    // Accumulate data until the header/body separator `\r\n\r\n` appears.
    let header_end = loop {
        let n = stream.read(&mut tmp).expect("read failed");
        assert!(n > 0, "unexpected EOF while reading headers");
        buf.extend_from_slice(&tmp[..n]);
        if let Some(pos) = find_bytes(&buf, b"\r\n\r\n") {
            break pos + 4; // position right after the separator
        }
    };

    let headers = std::str::from_utf8(&buf[..header_end]).unwrap_or("");
    let headers_lower = headers.to_ascii_lowercase();

    if headers_lower.contains("transfer-encoding: chunked")
        || headers_lower.contains("transfer-encoding:chunked")
    {
        let leftover = buf[header_end..].to_vec();
        read_chunked_body(stream, &leftover)
    } else {
        let content_length = parse_content_length(headers);
        let already_read = buf.len() - header_end;
        let remaining = content_length.saturating_sub(already_read);
        if remaining > 0 {
            let mut rest = vec![0u8; remaining];
            stream.read_exact(&mut rest).expect("read remaining body");
        }
        content_length
    }
}

/// Parse `Content-Length` from raw HTTP headers. Returns 0 if absent.
fn parse_content_length(headers: &str) -> usize {
    headers
        .lines()
        .find(|l| l.to_ascii_lowercase().starts_with("content-length:"))
        .and_then(|l| l.split_once(':'))
        .and_then(|(_, v)| v.trim().parse::<usize>().ok())
        .unwrap_or(0)
}

/// Consume a chunked HTTP body. `initial` holds bytes already buffered past
/// the header section.  Returns total body bytes (excluding chunk framing).
fn read_chunked_body(stream: &mut TcpStream, initial: &[u8]) -> usize {
    let mut data = initial.to_vec();
    let mut total_body: usize = 0;

    loop {
        // Ensure we have a complete chunk-size line (terminated by \r\n).
        while find_bytes(&data, b"\r\n").is_none() {
            let mut tmp = [0u8; 512];
            let n = stream.read(&mut tmp).expect("read chunk size");
            assert!(n > 0, "unexpected EOF in chunked body");
            data.extend_from_slice(&tmp[..n]);
        }

        let crlf_pos = find_bytes(&data, b"\r\n").unwrap();
        let size_str = std::str::from_utf8(&data[..crlf_pos])
            .unwrap_or("0")
            .trim();
        let chunk_size = usize::from_str_radix(size_str, 16).unwrap_or(0);

        // Advance past the size line.
        data = data[crlf_pos + 2..].to_vec();

        if chunk_size == 0 {
            // Terminal chunk — done (trailers ignored).
            break;
        }

        total_body += chunk_size;

        // We need `chunk_size` bytes of data followed by `\r\n`.
        let need = chunk_size + 2;
        while data.len() < need {
            let mut tmp = [0u8; 1024];
            let n = stream.read(&mut tmp).expect("read chunk data");
            assert!(n > 0, "unexpected EOF in chunk data");
            data.extend_from_slice(&tmp[..n]);
        }

        // Advance past the chunk data and its trailing \r\n.
        data = data[need..].to_vec();
    }

    total_body
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Find the first occurrence of `needle` in `haystack`.
fn find_bytes(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack.windows(needle.len()).position(|w| w == needle)
}

/// Generate a deterministic byte buffer of the given length.
fn generate_body(len: usize) -> Vec<u8> {
    (0..len).map(|i| (i % 251) as u8).collect()
}
