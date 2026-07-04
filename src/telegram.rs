const TELEGRAM_API: &str = "https://api.telegram.org";

pub async fn send_alert(token: &str, chat_id: &str, src_ip: &str, event_count: usize, window_secs: u64, protocol: &str) {
    let text = format!(
        "🪤 *TrapRS ALERTA*\n\n\
        🔴 IP: `{}`\n\
        📊 Eventos: `{}` em `{}s`\n\
        🌐 Protocolo: `{}`",
        src_ip, event_count, window_secs, protocol
    );

    let url = format!("{}/bot{}/sendMessage", TELEGRAM_API, token);
    let client = reqwest::Client::new();

    match client
        .post(&url)
        .json(&serde_json::json!({
            "chat_id": chat_id,
            "text": text,
            "parse_mode": "Markdown"
        }))
        .send()
        .await
    {
        Ok(resp) if resp.status().is_success() => {
            eprintln!("\x1b[32m[TELEGRAM]\x1b[0m Alerta enviado para chat {}", chat_id);
        }
        Ok(resp) => {
            eprintln!("\x1b[31m[TELEGRAM]\x1b[0m Falha: status {}", resp.status());
        }
        Err(e) => {
            eprintln!("\x1b[31m[TELEGRAM]\x1b[0m Erro: {}", e);
        }
    }
}
