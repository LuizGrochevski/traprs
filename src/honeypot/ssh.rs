use crate::logger::LogSender;
use crate::models::{HoneypotEvent, SshEvent, SshEventKind};
use chrono::Utc;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;

pub async fn run(port: u16, banner: String, tx: LogSender) {
    let addr = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(&addr).await.expect("Falha ao bind SSH");
    eprintln!("\x1b[33m[SSH]\x1b[0m Honeypot escutando em {}", addr);

    loop {
        match listener.accept().await {
            Ok((stream, peer)) => {
                let tx = tx.clone();
                let banner = banner.clone();
                tokio::spawn(async move {
                    handle_ssh(stream, peer.ip().to_string(), peer.port(), banner, tx).await;
                });
            }
            Err(e) => eprintln!("[SSH] Erro ao aceitar conexão: {}", e),
        }
    }
}

async fn handle_ssh(
    stream: tokio::net::TcpStream,
    src_ip: String,
    src_port: u16,
    banner: String,
    tx: LogSender,
) {
    let (reader, mut writer) = stream.into_split();
    let mut buf_reader = BufReader::new(reader);

    // Loga conexão inicial
    let _ = tx.send(HoneypotEvent::SSH(SshEvent {
        timestamp: Utc::now(),
        src_ip: src_ip.clone(),
        src_port,
        event: SshEventKind::Connect,
        client_banner: None,
    }));

    // Envia banner SSH
    let banner_line = format!("{}\r\n", banner);
    if writer.write_all(banner_line.as_bytes()).await.is_err() {
        return;
    }

    // Lê banner do cliente
    let mut client_banner = String::new();
    if buf_reader.read_line(&mut client_banner).await.is_err() {
        return;
    }
    let client_banner = client_banner.trim().to_string();

    let _ = tx.send(HoneypotEvent::SSH(SshEvent {
        timestamp: Utc::now(),
        src_ip: src_ip.clone(),
        src_port,
        event: SshEventKind::Connect,
        client_banner: Some(client_banner.clone()),
    }));

    // Lê dados brutos tentando extrair credenciais do protocolo SSH
    // O honeypot não implementa SSH real — apenas captura o tráfego inicial
    // e tenta extrair strings legíveis (usuário/senha de clientes simples ou scanners)
    let mut buffer = vec![0u8; 4096];
    loop {
        match tokio::time::timeout(
            tokio::time::Duration::from_secs(10),
            buf_reader.read(&mut buffer),
        )
        .await
        {
            Ok(Ok(0)) | Err(_) => break,
            Ok(Ok(n)) => {
                let raw = &buffer[..n];
                // Tenta extrair strings legíveis do payload bruto
                let printable: String = raw
                    .iter()
                    .map(|&b| if b.is_ascii_graphic() || b == b' ' { b as char } else { '·' })
                    .collect();

                // Tenta identificar tentativa de autenticação por password
                // Muitos scanners e brute-forcers enviam usuário e senha em texto claro
                // nos primeiros bytes antes do handshake completo
                let username = extrair_campo(raw, "user");
                let password = extrair_campo(raw, "password")
                    .or_else(|| extrair_campo(raw, "pass"));

                if username.is_some() || password.is_some() {
                    let _ = tx.send(HoneypotEvent::SSH(SshEvent {
                        timestamp: Utc::now(),
                        src_ip: src_ip.clone(),
                        src_port,
                        event: SshEventKind::AuthAttempt {
                            username: username.unwrap_or_else(|| extrair_ascii(raw)),
                            password,
                            method: "password".to_string(),
                        },
                        client_banner: Some(client_banner.clone()),
                    }));
                } else if printable.chars().filter(|&c| c != '·').count() > 8 {
                    // Loga o payload bruto como tentativa genérica
                    let _ = tx.send(HoneypotEvent::SSH(SshEvent {
                        timestamp: Utc::now(),
                        src_ip: src_ip.clone(),
                        src_port,
                        event: SshEventKind::AuthAttempt {
                            username: extrair_ascii(raw),
                            password: None,
                            method: "unknown".to_string(),
                        },
                        client_banner: Some(client_banner.clone()),
                    }));
                }
            }
            Ok(Err(_)) => break,
        }
    }

    let _ = tx.send(HoneypotEvent::SSH(SshEvent {
        timestamp: Utc::now(),
        src_ip: src_ip.clone(),
        src_port,
        event: SshEventKind::Disconnect,
        client_banner: None,
    }));
}

/// Extrai campo por nome de um buffer binário (ex: "user\x00\x00root")
fn extrair_campo(buf: &[u8], campo: &str) -> Option<String> {
    let campo_bytes = campo.as_bytes();
    buf.windows(campo_bytes.len())
        .position(|w| w == campo_bytes)
        .and_then(|pos| {
            let resto = &buf[pos + campo_bytes.len()..];
            // Pula bytes nulos/de controle
            let start = resto.iter().position(|&b| b.is_ascii_graphic())?;
            let val: String = resto[start..]
                .iter()
                .take_while(|&&b| b.is_ascii_graphic() && b != 0)
                .map(|&b| b as char)
                .collect();
            if val.is_empty() { None } else { Some(val) }
        })
}

/// Extrai a primeira string ASCII legível de um buffer binário
fn extrair_ascii(buf: &[u8]) -> String {
    let s: String = buf
        .iter()
        .filter(|&&b| b.is_ascii_graphic() || b == b' ')
        .map(|&b| b as char)
        .collect();
    s.chars().take(64).collect()
}
