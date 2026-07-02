use serde::Serialize;

#[derive(Serialize)]
pub struct AlertPayload {
    pub src_ip: String,
    pub event_count: usize,
    pub window_secs: u64,
    pub protocol: String,
}

pub async fn send_alert(url: &str, payload: AlertPayload) {
    let client = reqwest::Client::new();
    match client.post(url).json(&payload).send().await {
        Ok(resp) => {
            eprintln!(
                "\x1b[32m[WEBHOOK]\x1b[0m Alerta enviado → {} (status: {})",
                url,
                resp.status()
            );
        }
        Err(e) => {
            eprintln!("\x1b[31m[WEBHOOK]\x1b[0m Falha ao enviar alerta: {}", e);
        }
    }
}
