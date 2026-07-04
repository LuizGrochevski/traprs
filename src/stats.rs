use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs;

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Stats {
    pub total_events: u64,
    pub ssh_events: u64,
    pub http_events: u64,
    pub https_events: u64,
    pub alerts: u64,
    pub top_ips: HashMap<String, u64>,
    pub top_paths: HashMap<String, u64>,
    pub top_credentials: HashMap<String, u64>,
}

impl Stats {
    pub fn record_http(&mut self, ip: &str, path: &str, protocol_tag: &str) {
        self.total_events += 1;
        if protocol_tag == "HTTPS" {
            self.https_events += 1;
        } else {
            self.http_events += 1;
        }
        *self.top_ips.entry(ip.to_string()).or_default() += 1;
        if !path.is_empty() {
            *self.top_paths.entry(path.to_string()).or_default() += 1;
        }
    }

    pub fn record_ssh(&mut self, ip: &str, username: Option<&str>, password: Option<&str>) {
        self.total_events += 1;
        self.ssh_events += 1;
        *self.top_ips.entry(ip.to_string()).or_default() += 1;
        if let Some(user) = username {
            let pass = password.unwrap_or("?");
            let key = format!("{}:{}", user, pass);
            *self.top_credentials.entry(key).or_default() += 1;
        }
    }

    pub fn record_alert(&mut self) {
        self.alerts += 1;
    }

    pub fn top_n<V: Ord + Copy>(map: &HashMap<String, V>, n: usize) -> Vec<(String, V)> {
        let mut v: Vec<_> = map.iter().map(|(k, v)| (k.clone(), *v)).collect();
        v.sort_by(|a, b| b.1.cmp(&a.1));
        v.truncate(n);
        v
    }

    pub fn print_report(&self) {
        println!("\n📊 TrapRS — Relatório de Estatísticas");
        println!("─────────────────────────────────────");
        println!("  Total de eventos : {}", self.total_events);
        println!("  SSH              : {}", self.ssh_events);
        println!("  HTTP             : {}", self.http_events);
        println!("  HTTPS            : {}", self.https_events);
        println!("  Alertas          : {}", self.alerts);

        println!("\n🔝 Top IPs:");
        for (ip, n) in Self::top_n(&self.top_ips, 10) {
            println!("  {:5}  {}", n, ip);
        }

        println!("\n🔝 Top Paths:");
        for (path, n) in Self::top_n(&self.top_paths, 10) {
            println!("  {:5}  {}", n, path);
        }

        println!("\n🔑 Top Credenciais SSH:");
        for (cred, n) in Self::top_n(&self.top_credentials, 10) {
            println!("  {:5}  {}", n, cred);
        }
    }
}

pub async fn load(path: &PathBuf) -> Stats {
    match fs::read_to_string(path).await {
        Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
        Err(_) => Stats::default(),
    }
}

pub async fn save(path: &PathBuf, stats: &Stats) {
    let json = serde_json::to_string_pretty(stats).unwrap_or_default();
    let _ = fs::write(path, json).await;
}
