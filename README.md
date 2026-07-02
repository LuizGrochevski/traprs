# TrapRS 🪤🦀

![Rust](https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white)
![Tokio](https://img.shields.io/badge/Tokio-async-blue?style=for-the-badge)
![Platform](https://img.shields.io/badge/Platform-Linux%20%7C%20Android%20(Termux)-green?style=for-the-badge)
![License](https://img.shields.io/badge/License-Educational-orange?style=for-the-badge)

TrapRS é um honeypot TCP desenvolvido em **Rust** com foco em detecção de reconhecimento e captura de credenciais. Simula simultaneamente um servidor **SSH** (OpenSSH) e um servidor **HTTP** (Apache), registrando em log estruturado JSONL tudo que atacantes e scanners enviam — banners, headers, queries, payloads e tentativas de autenticação.

Complementa o pipeline de auditoria formado por **[Sentinel-RS](https://github.com/LuizGrochevski/Sentinel-RS)** e **[Netwatch-API](https://github.com/LuizGrochevski/netwatch-api)** — enquanto o Sentinel-RS detecta serviços na rede, o TrapRS detecta quem está tentando escanear ou invadir você.

---

## 🚀 Funcionalidades

- 🎭 **Honeypot SSH** — finge ser OpenSSH, captura:
  - IP e porta de origem + timestamp UTC
  - Banner do cliente SSH
  - Payload bruto do handshake (algoritmos, tentativas de auth)
  - Tentativas de autenticação (usuário, senha, método)
  - Evento de desconexão
- 🌐 **Honeypot HTTP** — finge ser Apache/nginx, captura:
  - Método, path, query string completa
  - Todos os headers (User-Agent, Authorization, Referer, etc)
  - Body da requisição (até 4KB)
  - Versão do protocolo HTTP
- 📋 **Log estruturado em JSONL** — um evento JSON por linha, fácil de processar com `jq` ou ingestão em SIEM
- 🎨 **Output colorido em tempo real** no terminal
- ⚙️ **CLI configurável** — portas, banners falsos e path do log via argumentos
- 📱 Compatível com **Termux (Android/ARM)** e **Linux**

---

## 🧠 Arquitetura

```
Atacante/Scanner
    │
    ├── TCP :2222 → Honeypot SSH
    │       ├── Captura banner do cliente
    │       ├── Lê payload bruto do handshake
    │       └── Extrai tentativas de auth
    │
    └── TCP :8080 → Honeypot HTTP
            ├── Parseia request line, headers e body
            └── Responde com página Apache falsa convincente
    │
    ▼
Logger assíncrono (mpsc channel)
    │
    ▼
logs/events.jsonl (JSONL estruturado)
```

---

## 🛠️ Tecnologias

| Tecnologia | Uso |
|---|---|
| Rust | Linguagem principal |
| Tokio | Runtime assíncrono (listeners paralelos) |
| serde / serde_json | Serialização dos eventos em JSON |
| chrono | Timestamps UTC precisos |
| clap | CLI com argumentos configuráveis |
| tracing | Structured logging interno |

---

## 📦 Instalação

### Linux
```bash
git clone https://github.com/LuizGrochevski/traprs.git
cd traprs
cargo build --release
```

### Termux (Android/ARM)
```bash
pkg install rust clang make git
git clone https://github.com/LuizGrochevski/traprs.git
cd traprs
ANDROID_API_LEVEL=24 cargo build --release
```

---

## 📄 Uso

```bash
# Termux (portas não privilegiadas)
./target/release/traprs --ssh-port 2222 --http-port 8080

# Linux (portas reais, requer root)
sudo ./target/release/traprs --ssh-port 22 --http-port 80

# Customizando banners e log
./target/release/traprs \
  --ssh-port 2222 \
  --http-port 8080 \
  --ssh-banner "SSH-2.0-OpenSSH_7.4p1 Debian-10+deb9u7" \
  --http-server "nginx/1.14.0 (Ubuntu)" \
  --log /var/log/traprs/events.jsonl
```

---

## 📊 Exemplo de log (JSONL)

**Evento HTTP:**
```json
{
  "protocol": "HTTP",
  "timestamp": "2026-07-02T00:35:51.684Z",
  "src_ip": "1.2.3.4",
  "src_port": 46228,
  "method": "GET",
  "path": "/admin",
  "query": "user=teste",
  "http_version": "HTTP/1.1",
  "headers": {
    "host": "127.0.0.1:8080",
    "user-agent": "curl/8.20.0",
    "accept": "*/*"
  },
  "body": "",
  "user_agent": "curl/8.20.0"
}
```

**Evento SSH:**
```json
{
  "protocol": "SSH",
  "timestamp": "2026-07-02T00:35:55.419Z",
  "src_ip": "1.2.3.4",
  "src_port": 57768,
  "event": {
    "kind": "auth_attempt",
    "username": "root",
    "password": "123456",
    "method": "password"
  },
  "client_banner": "SSH-2.0-OpenSSH_10.3"
}
```

**Processando com jq:**
```bash
# Ver só tentativas de autenticação SSH
cat logs/events.jsonl | jq 'select(.protocol == "SSH" and .event.kind == "auth_attempt")'

# Ver só requisições HTTP suspeitas
cat logs/events.jsonl | jq 'select(.protocol == "HTTP" and (.path | contains("/admin") or contains("/wp-login")))'

# Top IPs por número de eventos
cat logs/events.jsonl | jq -r '.src_ip' | sort | uniq -c | sort -rn | head -10
```

---

## 🛣️ Roadmap

- [x] Honeypot SSH com captura de banner e payload
- [x] Honeypot HTTP com captura completa de request
- [x] Log estruturado JSONL com timestamp UTC
- [x] Output colorido em tempo real
- [x] CLI configurável (portas, banners, log path)
- [x] Compatível com Termux e Linux
- [ ] Suporte a HTTPS (TLS falso para capturar clientes que tentam HTTPS)
- [ ] Integração com Netwatch-API (enviar eventos via webhook)
- [ ] Dashboard de eventos em tempo real
- [ ] Alertas por threshold (ex: N tentativas do mesmo IP em X segundos)

---

## 👨‍💻 Autor

**Luiz Felipe Grochevski** — [LinkedIn](https://www.linkedin.com/in/luiz-felipe-grochevski) | [GitHub](https://github.com/LuizGrochevski)

---

## ⚠️ Aviso

Este projeto é destinado exclusivamente para fins educacionais, laboratoriais e monitoramento de redes próprias ou autorizadas. Nunca use em redes ou sistemas sem autorização explícita.

