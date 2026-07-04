use crate::alert::ThresholdAlert;
use crate::dashboard::BroadcastSender;
use crate::models::HoneypotEvent;
use crate::stats;
use crate::webhook::{send_all, AlertPayload};
use std::path::PathBuf;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc;

pub type LogSender = mpsc::UnboundedSender<HoneypotEvent>;

pub async fn run_logger(
    log_path: PathBuf,
    stats_path: PathBuf,
    mut rx: mpsc::UnboundedReceiver<HoneypotEvent>,
    threshold: usize,
    window_secs: u64,
    webhook_urls: Vec<String>,
    broadcast_tx: BroadcastSender,
) {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .await
        .expect("Não foi possível abrir o arquivo de log");

    let mut alerter = ThresholdAlert::new(threshold, window_secs);
    let mut st = stats::load(&stats_path).await;
    let mut events_since_save = 0u64;

    while let Some(event) = rx.recv().await {
        let json = serde_json::to_string(&event).unwrap_or_default();
        let mut line = json.clone();
        line.push('\n');
        if let Err(e) = file.write_all(line.as_bytes()).await {
            eprintln!("Erro ao escrever log: {}", e);
        }
        let _ = file.flush().await;
        let _ = broadcast_tx.send(json);

        let (src_ip, protocol) = match &event {
            HoneypotEvent::SSH(e) => {
                eprintln!("\x1b[33m[SSH]\x1b[0m {} → {:?}", e.src_ip, e.event);
                let (user, pass) = match &e.event {
                    crate::models::SshEventKind::AuthAttempt { username, password, .. } => {
                        (Some(username.as_str()), password.as_deref())
                    }
                    _ => (None, None),
                };
                st.record_ssh(&e.src_ip, user, pass);
                (e.src_ip.clone(), "SSH".to_string())
            }
            HoneypotEvent::HTTP(e) => {
                let tag = &e.protocol_tag;
                let color = if tag == "HTTPS" { "\x1b[35m" } else { "\x1b[36m" };
                eprintln!("{}[{}]\x1b[0m {} {} {}", color, tag, e.src_ip, e.method, e.path);
                st.record_http(&e.src_ip, &e.path, tag);
                (e.src_ip.clone(), tag.clone())
            }
        };

        events_since_save += 1;
        if events_since_save >= 10 {
            stats::save(&stats_path, &st).await;
            events_since_save = 0;
        }

        if alerter.record(&src_ip) {
            let count = alerter.count(&src_ip);
            eprintln!(
                "\x1b[31m[⚠️  ALERTA]\x1b[0m IP {} disparou {} eventos em {}s!",
                src_ip, count, window_secs
            );
            st.record_alert();

            if !webhook_urls.is_empty() {
                let payload = AlertPayload {
                    src_ip: src_ip.clone(),
                    event_count: count,
                    window_secs,
                    protocol: protocol.clone(),
                };
                let urls = webhook_urls.clone();
                tokio::spawn(async move {
                    send_all(&urls, payload).await;
                });
            }
        }
    }

    // Salva stats ao encerrar
    stats::save(&stats_path, &st).await;
}
