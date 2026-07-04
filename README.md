# TrapRS 🪤🦀

![Rust](https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white)
![Tokio](https://img.shields.io/badge/Tokio-async-blue?style=for-the-badge)
![Platform](https://img.shields.io/badge/Platform-Linux%20%7C%20Android%20(Termux)-green?style=for-the-badge)
![License](https://img.shields.io/badge/License-Educational-orange?style=for-the-badge)

TrapRS é um honeypot TCP desenvolvido em **Rust** com foco em detecção de reconhecimento e captura de credenciais. Simula simultaneamente servidores **SSH** (OpenSSH), **HTTP** (Apache) e **HTTPS** (Apache com TLS), registrando em log estruturado JSONL tudo que atacantes e scanners enviam. Dispara alertas em tempo real quando um IP atinge um threshold configurável, com integração via webhook para a **[Netwatch-API](https://github.com/LuizGrochevski/netwatch-api)**, dashboard web ao vivo via WebSocket, persistência de estatísticas em JSON e alertas via **Telegram**.

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
- 🔒 **Honeypot HTTPS** — certificado TLS autoassinado gerado em runtime via `rcgen`, captura tudo que o HTTP captura mais handshake TLS e clientes que falham no TLS
- ⚠️ **Alertas por threshold** — quando um IP dispara N eventos em X segundos, alerta vermelho no terminal em tempo real
- 🔗 **Múltiplos webhooks** — alertas enviados em paralelo para quantos endpoints quiser via `--webhook-url`
- 📱 **Alertas via Telegram** — receba notificações no celular em tempo real quando um ataque é detectado
- 📊 **Dashboard web em tempo real** — interface HTML/JS conectada via WebSocket, mostra feed de eventos ao vivo, top IPs, top paths e contadores por protocolo
- 💾 **Persistência de estatísticas** — salva top IPs, top paths e credenciais SSH mais tentadas em JSON a cada 10 eventos e ao encerrar
- 📋 **Log estruturado em JSONL** — um evento JSON por linha, fácil de processar com `jq`
- 🎨 **Output colorido em tempo real** no terminal
- ⚙️ **CLI totalmente configurável** — portas, banners falsos, log path, threshold, webhooks, Telegram, dashboard e stats
- 📱 Compatível com **Termux (Android/ARM)** e **Linux**

---

## 🧠 Arquitetura

```
Atacante/Scanner
    │
    ├── TCP :2222  → Honeypot SSH
    ├── TCP :8080  → Honeypot HTTP
    └── TCP :8443  → Honeypot HTTPS (TLS autoassinado)
    │
    ▼
Logger assíncrono (mpsc channel)
    │
    ├── logs/events.jsonl          (log JSONL)
    ├── logs/stats.json            (estatísticas persistidas)
    ├── Output colorido no terminal
    ├── ThresholdAlert
    │       ├── send_all() → POST /webhook/alert (múltiplos endpoints)
    │       └── Telegram Bot API → mensagem no celular
    └── broadcast (WebSocket)
            │
            ▼
    Dashboard web (ws://:9000)
```

---

## 🛠️ Tecnologias

| Tecnologia | Uso |
|---|---|
| Rust | Linguagem principal |
| Tokio | Runtime assíncrono (listeners paralelos) |
| tokio-rustls | TLS assíncrono para o honeypot HTTPS |
| tokio-tungstenite | Servidor WebSocket para o dashboard |
| rcgen | Geração de certificado autoassinado em runtime |
| reqwest | Cliente HTTP para webhooks e Telegram |
| serde / serde_json | Serialização dos eventos e estatísticas em JSON |
| chrono | Timestamps UTC precisos |
| clap | CLI com argumentos configuráveis |

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
# Básico (Termux)
./target/release/traprs --ssh-port 2222 --http-port 8080 --https-port 8443

# Completo: dashboard, alertas, webhooks, Telegram e stats
./target/release/traprs \
  --ssh-port 2222 \
  --http-port 8080 \
  --https-port 8443 \
  --dashboard-port 9000 \
  --alert-threshold 10 \
  --alert-window 60 \
  --webhook-url http://localhost:8000/webhook/alert \
  --webhook-url http://outro-servidor/webhook \
  --telegram-token SEU_TOKEN_DO_BOT \
  --telegram-chat-id SEU_CHAT_ID \
  --stats logs/stats.json

# Linux (portas reais, requer root)
sudo ./target/release/traprs --ssh-port 22 --http-port 80 --https-port 443
```

**Dashboard:** abra `dashboard/index.html` num servidor HTTP local:
```bash
cd dashboard && python3 -m http.server 7777
# Acesse http://localhost:7777
```

---

## 📱 Configurando alertas no Telegram

1. Abra o Telegram e busque por `@BotFather`
2. Mande `/newbot` e siga as instruções para criar seu bot
3. Copie o **token** gerado (formato: `123456789:AAHxxxxxxxxxxxxx`)
4. Mande qualquer mensagem para o bot que você criou
5. Acesse `https://api.telegram.org/botSEU_TOKEN/getUpdates` e copie o `chat.id`
6. Rode o TrapRS com `--telegram-token` e `--telegram-chat-id`

**Exemplo de alerta recebido no Telegram:**
```
🪤 TrapRS ALERTA

🔴 IP: 1.2.3.4
📊 Eventos: 10 em 60s
🌐 Protocolo: HTTP
```

---

## 📊 Exemplo de output

```
🪤 TrapRS iniciado!
   SSH       → porta 2222
   HTTP      → porta 8080
   HTTPS     → porta 8443
   Dashboard → ws://0.0.0.0:9000
   Log       → logs/events.jsonl
[SSH] Honeypot escutando em 0.0.0.0:2222
[HTTP] Honeypot escutando em 0.0.0.0:8080
[HTTPS] Honeypot escutando em 0.0.0.0:8443
[DASH] Dashboard WebSocket em ws://0.0.0.0:9000
[HTTP] 1.2.3.4 GET /wp-login.php
[HTTP] 1.2.3.4 GET /.env
[HTTP] 1.2.3.4 GET /admin
[⚠️  ALERTA] IP 1.2.3.4 disparou 3 eventos em 10s!
[WEBHOOK] Alerta enviado → http://localhost:8000/webhook/alert (status: 200 OK)
[TELEGRAM] Alerta enviado para chat 940237849
[SSH] 1.2.3.4 → AuthAttempt { username: "root", password: Some("123456"), method: "password" }
```

---

## 📋 Exemplo de log e estatísticas

**Evento HTTP (JSONL):**
```json
{
  "protocol": "HTTP",
  "timestamp": "2026-07-04T02:00:00.000Z",
  "src_ip": "1.2.3.4",
  "src_port": 46228,
  "method": "GET",
  "path": "/admin",
  "query": "",
  "http_version": "HTTP/1.1",
  "headers": { "user-agent": "masscan/1.0" },
  "body": "",
  "user_agent": "masscan/1.0",
  "protocol_tag": "HTTP"
}
```

**Estatísticas persistidas (stats.json):**
```json
{
  "total_events": 150,
  "ssh_events": 30,
  "http_events": 110,
  "https_events": 10,
  "alerts": 8,
  "top_ips": { "1.2.3.4": 95, "5.6.7.8": 55 },
  "top_paths": { "/admin": 40, "/wp-login.php": 35, "/.env": 20 },
  "top_credentials": { "root:123456": 12, "admin:admin": 8 }
}
```

**Processando com jq:**
```bash
# Top IPs
cat logs/events.jsonl | jq -r '.src_ip' | sort | uniq -c | sort -rn | head -10

# Credenciais SSH tentadas
cat logs/events.jsonl | jq 'select(.event.kind == "auth_attempt") | "\(.event.username):\(.event.password)"'

# Paths HTTP mais atacados
cat logs/events.jsonl | jq -r 'select(.protocol == "HTTP") | .path' | sort | uniq -c | sort -rn
```

---

## 🔗 Integração com Netwatch-API

```
TrapRS (honeypot detecta ataque)
    │
    ├── POST /webhook/alert (múltiplos endpoints em paralelo)
    └── Telegram Bot API (notificação no celular)
            │
            ▼
    Netwatch-API (loga e pode encadear com scan ou CVE lookup)
```

Payload do webhook:
```json
{
  "src_ip": "1.2.3.4",
  "event_count": 10,
  "window_secs": 60,
  "protocol": "HTTP"
}
```

---

## 🛣️ Roadmap

- [x] Honeypot SSH com captura de banner e payload
- [x] Honeypot HTTP com captura completa de request
- [x] Honeypot HTTPS com TLS autoassinado em runtime
- [x] Log estruturado JSONL com timestamp UTC
- [x] Output colorido em tempo real
- [x] CLI configurável (portas, banners, log path)
- [x] Alertas por threshold (N eventos em X segundos)
- [x] Múltiplos webhooks em paralelo
- [x] Dashboard web em tempo real via WebSocket
- [x] Persistência de estatísticas (top IPs, paths, credenciais)
- [x] Alertas via Telegram
- [x] Compatível com Termux e Linux

---

## 👨‍💻 Autor

**Luiz Felipe Grochevski** — [LinkedIn](https://www.linkedin.com/in/luiz-felipe-grochevski) | [GitHub](https://github.com/LuizGrochevski)

---

## ⚠️ Aviso

Este projeto é destinado exclusivamente para fins educacionais, laboratoriais e monitoramento de redes próprias ou autorizadas. Nunca use em redes ou sistemas sem autorização explícita.

