use crate::models::HoneypotEvent;
use futures_util::SinkExt;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::broadcast;
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::Message;

pub type BroadcastSender = broadcast::Sender<String>;

pub async fn run(port: u16, tx: BroadcastSender) {
    let addr = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(&addr)
        .await
        .expect("Falha ao bind dashboard WebSocket");
    eprintln!(
        "\x1b[32m[DASH]\x1b[0m Dashboard WebSocket em ws://{}",
        addr
    );

    loop {
        match listener.accept().await {
            Ok((stream, peer)) => {
                let mut rx = tx.subscribe();
                eprintln!("\x1b[32m[DASH]\x1b[0m Cliente conectado: {}", peer);

                tokio::spawn(async move {
                    match accept_async(stream).await {
                        Ok(ws) => {
                            let (mut write, _read) = futures_util::StreamExt::split(ws);
                            while let Ok(msg) = rx.recv().await {
                                if write.send(Message::Text(msg.into())).await.is_err() {
                                    break;
                                }
                            }
                        }
                        Err(e) => eprintln!("[DASH] Erro no handshake WS: {}", e),
                    }
                });
            }
            Err(e) => eprintln!("[DASH] Erro ao aceitar: {}", e),
        }
    }
}

pub fn serialize_event(event: &HoneypotEvent) -> String {
    serde_json::to_string(event).unwrap_or_default()
}
