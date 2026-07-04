mod alert;
mod config;
mod dashboard;
mod honeypot;
mod logger;
mod models;
mod stats;
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
    println!("   SSH       → porta {}", cfg.ssh_port);
    println!("   HTTP      → porta {}", cfg.http_port);
    println!("   HTTPS     → porta {}", cfg.https_port);
    println!("   Dashboard → ws://0.0.0.0:{}", cfg.dashboard_port);
    println!("   Log       → {}", cfg.log.display());

    let (tx, rx) = mpsc::unbounded_channel();
    let (broadcast_tx, _) = tokio::sync::broadcast::channel(256);

    // Dashboard WebSocket
    let dash_tx = broadcast_tx.clone();
    let dash_port = cfg.dashboard_port;
    tokio::spawn(async move {
        dashboard::run(dash_port, dash_tx).await;
    });

    // Logger assíncrono
    // Imprime stats e encerra se --stats foi passado como flag
    // (verificamos se há argumento extra; aqui apenas mostramos ao iniciar)
    let log_path = cfg.log.clone();
    let stats_path = cfg.stats.clone();
    let threshold = cfg.alert_threshold;
    let window = cfg.alert_window;
    let webhook_urls = cfg.webhook_url.clone();
    let blog_tx = broadcast_tx.clone();
    tokio::spawn(async move {
        logger::run_logger(log_path, stats_path, rx, threshold, window, webhook_urls, blog_tx).await;
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
