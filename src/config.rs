use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug, Clone)]
#[command(
    name = "traprs",
    about = "TrapRS 🪤 — Honeypot TCP em Rust (SSH + HTTP)",
    version = "0.1.0"
)]
pub struct Config {
    /// Porta do honeypot SSH
    #[arg(long, default_value = "2222")]
    pub ssh_port: u16,

    /// Porta do honeypot HTTP
    #[arg(long, default_value = "8080")]
    pub http_port: u16,

    /// Arquivo de log (JSONL)
    #[arg(long, default_value = "logs/events.jsonl")]
    pub log: PathBuf,

    /// Banner SSH falso apresentado aos clientes
    #[arg(long, default_value = "SSH-2.0-OpenSSH_8.9p1 Ubuntu-3ubuntu0.6")]
    pub ssh_banner: String,

    /// Banner HTTP falso (header Server)
    #[arg(long, default_value = "Apache/2.4.7 (Ubuntu)")]
    pub http_server: String,
}
