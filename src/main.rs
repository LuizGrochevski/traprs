mod config;
mod honeypot;
mod logger;
mod models;

use clap::Parser;
use config::Config;
use std::fs;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() {
    let cfg = Config::parse();

    // Cria diretório de log se não existir
    if let Some(parent) = cfg.log.parent() {
        fs::create_dir_all(parent).ok();
    }

    println!("🪤 TrapRS iniciado!");
    println!("   SSH  → porta {}", cfg.ssh_port);
    println!("   HTTP → porta {}", cfg.http_port);
    println!("   Log  → {}", cfg.log.display());

    let (tx, rx) = mpsc::unbounded_channel();

    // Logger assíncrono
    let log_path = cfg.log.clone();
    tokio::spawn(async move {
        logger::run_logger(log_path, rx).await;
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

    // Mantém o processo vivo
    tokio::signal::ctrl_c().await.expect("Falha ao escutar Ctrl+C");
    println!("\n🛑 TrapRS encerrado.");
}
