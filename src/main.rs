mod alert;
mod config;
mod honeypot;
mod logger;
mod models;
mod webhook;

use clap::Parser;
use config::Config;
use std::fs;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() {
    // Garante que ring é o provider de crypto para tokio-rustls e reqwest
    let _ = tokio_rustls::rustls::crypto::ring::default_provider().install_default();
    let cfg = Config::parse();

    // Cria diretório de log se não existir
    if let Some(parent) = cfg.log.parent() {
        fs::create_dir_all(parent).ok();
    }

    println!("🪤 TrapRS iniciado!");
    println!("   SSH   → porta {}", cfg.ssh_port);
    println!("   HTTP  → porta {}", cfg.http_port);
    println!("   HTTPS → porta {}", cfg.https_port);
    println!("   Log   → {}", cfg.log.display());

    let (tx, rx) = mpsc::unbounded_channel();

    // Logger assíncrono
    let log_path = cfg.log.clone();
    let threshold = cfg.alert_threshold;
    let window = cfg.alert_window;
    let webhook_url = cfg.webhook_url.clone();
    tokio::spawn(async move {
        logger::run_logger(log_path, rx, threshold, window, webhook_url).await;
    });

    // Honeypots em paralelo
    let ssh_tx = tx.clone();
    let ssh_banner = cfg.ssh_banner.clone();
    let ssh_port = cfg.ssh_port;
    tokio::spawn(async move {
        honeypot::ssh::run(ssh_port, ssh_banner, ssh_tx).await;
    });

    let http_tx = tx.clone();
    let http_banner = cfg.http_server.clone();
    let http_port = cfg.http_port;
    tokio::spawn(async move {
        honeypot::http::run(http_port, http_banner, http_tx).await;
    });

    let https_tx = tx.clone();
    let https_banner = cfg.http_server.clone();
    let https_port = cfg.https_port;
    tokio::spawn(async move {
        honeypot::https::run(https_port, https_banner, https_tx).await;
    });

    // Mantém o processo vivo
    tokio::signal::ctrl_c().await.expect("Falha ao escutar Ctrl+C");
    println!("\n🛑 TrapRS encerrado.");
}
