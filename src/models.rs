use chrono::{DateTime, Utc};
use serde::Serialize;
use std::collections::HashMap;

#[derive(Serialize, Debug, Clone)]
#[serde(tag = "protocol")]
pub enum HoneypotEvent {
    SSH(SshEvent),
    HTTP(HttpEvent),
}

#[derive(Serialize, Debug, Clone)]
pub struct SshEvent {
    pub timestamp: DateTime<Utc>,
    pub src_ip: String,
    pub src_port: u16,
    pub event: SshEventKind,
    pub client_banner: Option<String>,
}

#[derive(Serialize, Debug, Clone)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SshEventKind {
    Connect,
    AuthAttempt {
        username: String,
        password: Option<String>,
        method: String,
    },
    Disconnect,
}

#[derive(Serialize, Debug, Clone)]
pub struct HttpEvent {
    pub timestamp: DateTime<Utc>,
    pub src_ip: String,
    pub src_port: u16,
    pub method: String,
    pub path: String,
    pub query: String,
    pub http_version: String,
    pub headers: HashMap<String, String>,
    pub body: String,
    pub user_agent: String,
}
