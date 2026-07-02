use crate::models::HoneypotEvent;
use std::path::PathBuf;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc;

pub type LogSender = mpsc::UnboundedSender<HoneypotEvent>;

pub async fn run_logger(log_path: PathBuf, mut rx: mpsc::UnboundedReceiver<HoneypotEvent>) {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .await
        .expect("Não foi possível abrir o arquivo de log");

    while let Some(event) = rx.recv().await {
        let mut line = serde_json::to_string(&event).unwrap_or_default();
        line.push('\n');
        if let Err(e) = file.write_all(line.as_bytes()).await {
            eprintln!("Erro ao escrever log: {}", e);
        }
        let _ = file.flush().await;

        // Também imprime no terminal colorido
        match &event {
            HoneypotEvent::SSH(e) => {
                eprintln!("\x1b[33m[SSH]\x1b[0m {} → {:?}", e.src_ip, e.event);
            }
            HoneypotEvent::HTTP(e) => {
                eprintln!(
                    "\x1b[36m[HTTP]\x1b[0m {} {} {}",
                    e.src_ip, e.method, e.path
                );
            }
        }
    }
}
