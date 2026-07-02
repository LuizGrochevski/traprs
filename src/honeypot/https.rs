use crate::logger::LogSender;
use crate::models::{HoneypotEvent, HttpEvent};
use chrono::Utc;
use rcgen::generate_simple_self_signed;
use rustls_pki_types::{CertificateDer, PrivateKeyDer};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;
use tokio_rustls::rustls::ServerConfig;
use tokio_rustls::TlsAcceptor;

pub async fn run(port: u16, server_banner: String, tx: LogSender) {
    // Gera certificado autoassinado falso em runtime
    let subject_alt_names = vec!["localhost".to_string(), "127.0.0.1".to_string()];
    let cert = generate_simple_self_signed(subject_alt_names)
        .expect("Falha ao gerar certificado");

    let cert_der = CertificateDer::from(cert.cert.der().to_vec());
    let key_der = PrivateKeyDer::Pkcs8(cert.key_pair.serialize_der().into());

    let config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(vec![cert_der], key_der)
        .expect("Falha ao configurar TLS");

    let acceptor = TlsAcceptor::from(Arc::new(config));
    let addr = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(&addr).await.expect("Falha ao bind HTTPS");
    eprintln!("\x1b[35m[HTTPS]\x1b[0m Honeypot escutando em {}", addr);

    loop {
        match listener.accept().await {
            Ok((stream, peer)) => {
                let acceptor = acceptor.clone();
                let tx = tx.clone();
                let banner = server_banner.clone();
                let src_ip = peer.ip().to_string();
                let src_port = peer.port();

                tokio::spawn(async move {
                    match acceptor.accept(stream).await {
                        Ok(tls_stream) => {
                            handle_https(tls_stream, src_ip, src_port, banner, tx).await;
                        }
                        Err(_) => {
                            // Cliente não completou o handshake TLS — ainda loga a tentativa
                            let _ = tx.send(HoneypotEvent::HTTP(HttpEvent {
                                timestamp: Utc::now(),
                                src_ip,
                                src_port,
                                method: "TLS_HANDSHAKE_FAILED".to_string(),
                                path: String::new(),
                                query: String::new(),
                                http_version: String::new(),
                                headers: HashMap::new(),
                                body: String::new(),
                                user_agent: String::new(),
                                protocol_tag: "HTTPS".to_string(),
                            }));
                        }
                    }
                });
            }
            Err(e) => eprintln!("[HTTPS] Erro ao aceitar conexão: {}", e),
        }
    }
}

async fn handle_https(
    stream: tokio_rustls::server::TlsStream<tokio::net::TcpStream>,
    src_ip: String,
    src_port: u16,
    server_banner: String,
    tx: LogSender,
) {
    let (reader, mut writer) = tokio::io::split(stream);
    let mut buf_reader = BufReader::new(reader);

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

    let body = if content_length > 0 {
        let to_read = content_length.min(4096);
        let mut body_buf = vec![0u8; to_read];
        let _ = buf_reader.read_exact(&mut body_buf).await;
        String::from_utf8_lossy(&body_buf).to_string()
    } else {
        String::new()
    };

    let _ = tx.send(HoneypotEvent::HTTP(HttpEvent {
        timestamp: Utc::now(),
        src_ip,
        src_port,
        method,
        path,
        query,
        http_version,
        headers,
        body,
        user_agent,
        protocol_tag: "HTTPS".to_string(),
    }));

    let body_html = r#"<!DOCTYPE html>
<html><head><title>Apache2 Ubuntu Default Page</title></head>
<body><h1>It works!</h1></body></html>"#;

    let response = format!(
        "HTTP/1.1 200 OK\r\nServer: {}\r\nContent-Type: text/html\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        server_banner,
        body_html.len(),
        body_html
    );
    let _ = writer.write_all(response.as_bytes()).await;
}
