use crate::logger::LogSender;
use crate::models::{HoneypotEvent, HttpEvent};
use chrono::Utc;
use std::collections::HashMap;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;

pub async fn run(port: u16, server_banner: String, tx: LogSender) {
    let addr = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(&addr).await.expect("Falha ao bind HTTP");
    eprintln!("\x1b[36m[HTTP]\x1b[0m Honeypot escutando em {}", addr);

    loop {
        match listener.accept().await {
            Ok((stream, peer)) => {
                let tx = tx.clone();
                let banner = server_banner.clone();
                tokio::spawn(async move {
                    handle_connection(stream, peer.ip().to_string(), peer.port(), banner, tx).await;
                });
            }
            Err(e) => eprintln!("[HTTP] Erro ao aceitar conexão: {}", e),
        }
    }
}

async fn handle_connection(
    stream: tokio::net::TcpStream,
    src_ip: String,
    src_port: u16,
    server_banner: String,
    tx: LogSender,
) {
    let (reader, mut writer) = stream.into_split();
    let mut buf_reader = BufReader::new(reader);

    // Lê a request line
    let mut request_line = String::new();
    if buf_reader.read_line(&mut request_line).await.is_err() {
        return;
    }
    let request_line = request_line.trim().to_string();
    let parts: Vec<&str> = request_line.splitn(3, ' ').collect();
    if parts.len() < 3 {
        return;
    }

    let method = parts[0].to_string();
    let raw_path = parts[1].to_string();
    let http_version = parts[2].to_string();

    let (path, query) = match raw_path.split_once('?') {
        Some((p, q)) => (p.to_string(), q.to_string()),
        None => (raw_path.clone(), String::new()),
    };

    // Lê os headers
    let mut headers: HashMap<String, String> = HashMap::new();
    let mut content_length = 0usize;
    loop {
        let mut line = String::new();
        if buf_reader.read_line(&mut line).await.is_err() {
            break;
        }
        let line = line.trim().to_string();
        if line.is_empty() {
            break;
        }
        if let Some((k, v)) = line.split_once(':') {
            let key = k.trim().to_lowercase();
            let val = v.trim().to_string();
            if key == "content-length" {
                content_length = val.parse().unwrap_or(0);
            }
            headers.insert(key, val);
        }
    }

    let user_agent = headers.get("user-agent").cloned().unwrap_or_default();

    // Lê o body (até 4KB)
    let body = if content_length > 0 {
        let to_read = content_length.min(4096);
        let mut body_buf = vec![0u8; to_read];
        let _ = buf_reader.read_exact(&mut body_buf).await;
        String::from_utf8_lossy(&body_buf).to_string()
    } else {
        String::new()
    };

    // Loga o evento
    let event = HoneypotEvent::HTTP(HttpEvent {
        timestamp: Utc::now(),
        src_ip: src_ip.clone(),
        src_port,
        method: method.clone(),
        path: path.clone(),
        query: query.clone(),
        http_version,
        headers,
        body,
        user_agent,
    });
    let _ = tx.send(event);

    // Responde com página falsa convincente
    let body_html = r#"<!DOCTYPE html>
<html><head><title>Apache2 Ubuntu Default Page</title></head>
<body><h1>It works!</h1>
<p>This is the default welcome page used to test the correct operation of the Apache2 server after installation.</p>
</body></html>"#;

    let response = format!(
        "HTTP/1.1 200 OK\r\nServer: {}\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        server_banner,
        body_html.len(),
        body_html
    );

    let _ = writer.write_all(response.as_bytes()).await;
}
