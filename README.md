# CRM Quality Inspector 🛡️

[![Rust](https://img.shields.io/badge/rust-1.88%2B-orange?logo=rust)](https://www.rust-lang.org)
[![Axum](https://img.shields.io/badge/axum-0.7-blue)](https://github.com/tokio-rs/axum)
[![PostgreSQL](https://img.shields.io/badge/postgresql-14%2B-blue?logo=postgresql)](https://www.postgresql.org)
[![License: MIT](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)

A modern, high-performance call center quality assurance (QA) platform built for **Persian/Farsi** contact centers in banking, insurance, and BPO industries. Features customizable rubrics, weighted scoring, CAPA workflow, real-time dashboard, AI-assisted coaching, and Docker-based one-line deployment.

---

## ✨ Features

### 🎯 Core
- **Multi-dimensional scoring** with weighted criteria (each can be marked "critical")
- **Smart rubric auto-selection** by agent department + customer product type
- **Critical fail detection** — flags interactions where any critical criterion < 60
- **CAPA workflow** — root cause + corrective action tracking for every issue
- **Hierarchical grading** — عالی / خوب / نیازمند بهبود / ضعیف
- **Bulk score loading** — single `/api/scores` endpoint for fast dashboard

### 📊 Analytics
- Real-time dashboard with 8 KPIs (agents, customers, interactions, coverage %, avg quality, open issues, critical fails, grade)
- Quality trend chart (Chart.js) — score progression over time
- Per-agent performance reports with score history
- Coverage tracking — % of interactions evaluated
- Bulk agent + interaction loading (parallel Promise.all) for sub-second dashboard

### 🛠️ Productivity
- **Full-text search** in interactions (subject + transcript + tags)
- **Lazy load** per tab + localStorage cache (60s TTL) for sub-second repeat loads
- **Server-side pagination** for all list endpoints (interactions, customers, agents, issues)
- **CSV export** with UTF-8 BOM (Excel Persian support)
- **Smart QA recommendations** — auto-suggest high-priority interactions

### 🎨 UX
- Modern dark RTL interface (Linear/Vercel-inspired)
- Persian/Farsi native support
- **Token expiry toast** — "نشست شما منقضی شده" + redirect to login
- Modal-based scoring with 7-slider rubric
- Filter interactions by channel, agent, status

### 🔐 Security
- **Bearer-token sessions** (24h TTL)
- **bcrypt password hashing**
- **Fail-fast on misconfiguration** — refuses to start if `DATABASE_URL` or `ADMIN_PASSWORD` are missing/weak
- **Password strength validation** (≥12 chars, rejects `admin`, `password`, `123456`, etc.)
- **Weak DB password detection** — rejects `ssdssd`, `admin1234`, placeholders
- **No insecure defaults** — every public image requires operator-provided secrets
- **CORS-enabled**, ready for SPA frontends
- **Swagger UI** at `/docs` (utoipa-generated)

---

## 🏗️ Architecture

```
┌─────────────────────────────────────────────────────┐
│              Browser (RTL, Dark Theme)             │
│   Vanilla JS + Chart.js + jQuery (locally bundled) │
└──────────────────┬──────────────────────────────────┘
                   │ HTTP/JSON (Bearer token)
                   ▼
┌─────────────────────────────────────────────────────┐
│         Axum 0.7 (async, tokio multi-thread)        │
│   ┌──────────┬─────────────┬────────────────────┐  │
│   │  api.rs  │ service.rs  │     auth.rs        │  │
│   │  routes  │ business    │  session + bcrypt  │  │
│   └──────────┴─────────────┴────────────────────┘  │
│         ┌─────────────┬──────────────┐             │
│         │  store.rs   │  models.rs   │             │
│         │  SQLx ops   │  Domain types│             │
│         └─────────────┴──────────────┘             │
└──────────────────┬──────────────────────────────────┘
                   │ SQLx (async, connection pool)
                   ▼
         ┌──────────────────┐
         │  PostgreSQL 14+  │
         │  (any host)      │
         └──────────────────┘
```

**Stack**:
- **Backend**: Rust 1.88, axum 0.7, tokio, sqlx (PostgreSQL), bcrypt, utoipa (Swagger)
- **Frontend**: Vanilla JS (no framework), Chart.js 4, jQuery 3 (all locally bundled for Iran CDN blocks)
- **Storage**: PostgreSQL 14+ (remote or local)
- **Static**: `include_str!`-embedded HTML + locally served JS/CSS

---

## 🚀 Quick Start

### Prerequisites
- **Rust 1.88+** ([install](https://rustup.rs))
- **PostgreSQL 14+** accessible (or use the bundled Docker Compose)
- **OpenSSL** (for `rand -base64` to generate strong passwords)

### 🐳 Option A: Docker (recommended)

This is the easiest path — one command brings up PostgreSQL + the app.

```bash
# 1. Clone
git clone https://github.com/Saeeddn/crm-quality-inspector.git
cd crm-quality-inspector

# 2. Create .env from template
cp .env.example .env

# 3. Generate strong passwords
echo "POSTGRES_PASSWORD=$(openssl rand -base64 24 | tr -d '/+=' | head -c 32)" >> .env.tmp
echo "ADMIN_PASSWORD=$(openssl rand -base64 18 | tr -d '/+=' | head -c 24)" >> .env.tmp
# Then manually merge POSTGRES_PASSWORD and ADMIN_PASSWORD into .env
# (compose reads POSTGRES_PASSWORD for the db service and ADMIN_PASSWORD for the app)

# 4. Build and start
docker compose up -d --build

# 5. Verify
docker compose ps                    # both containers should be "healthy"
docker compose logs app | tail -20   # should show "listening on http://0.0.0.0:3000"
curl http://localhost:3000/api/health
# → {"data":{"database":"connected","status":"ok"},"success":true}
```

Open http://localhost:3000 and log in with the `ADMIN_USERNAME` and `ADMIN_PASSWORD` you set in `.env`.

**⏱ First build takes ~5 minutes** (compiles Rust deps in `rust:1.88-bookworm`). Subsequent builds are fast (~30s with cache).

### 🛠️ Option B: Local Rust build (no Docker)

```bash
# 1. Clone
git clone https://github.com:Saeeddn/crm-quality-inspector.git
cd crm-quality-inspector

# 2. Set up PostgreSQL (or use a remote one)
#    - Create database: CREATE DATABASE crm_quality_inspector;
#    - Create user:     CREATE USER crm_quality WITH PASSWORD 'STRONG_RANDOM';

# 3. Configure
cp .env.example .env
# Edit .env and set DATABASE_URL + ADMIN_PASSWORD
export $(grep -v '^#' .env | xargs)

# 4. Build
cargo build --release

# 5. Run
./target/release/crm-quality-inspector
```

Open http://localhost:3000.

### ⚙️ Option C: Systemd service (production single-server)

```bash
# 1. Copy binary to /usr/local/bin
sudo cp target/release/crm-quality-inspector /usr/local/bin/
sudo chmod 755 /usr/local/bin/crm-quality-inspector

# 2. Create service user
sudo useradd -r -s /sbin/nologin crm

# 3. Create app directory
sudo mkdir -p /opt/crm-quality-inspector/static
sudo chown -R crm:crm /opt/crm-quality-inspector

# 4. Copy static assets
sudo cp -r static/* /opt/crm-quality-inspector/static/
sudo chown -R crm:crm /opt/crm-quality-inspector/static

# 5. Create .env (see Configuration below)
sudo nano /opt/crm-quality-inspector/.env
sudo chown crm:crm /opt/crm-quality-inspector/.env
sudo chmod 600 /opt/crm-quality-inspector/.env

# 6. Install systemd unit (see deploy/systemd/crm-quality-inspector.service)
sudo cp deploy/systemd/crm-quality-inspector.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable --now crm-quality-inspector

# 7. Verify
sudo systemctl status crm-quality-inspector
sudo journalctl -u crm-quality-inspector -f
```

---

## ⚙️ Configuration

All config is via environment variables or a `.env` file (use `.env.example` as a template).

| Variable | Required | Description | Default |
|----------|:--------:|-------------|---------|
| `DATABASE_URL` | ✅ | PostgreSQL connection string | **none** — refuses to start if missing |
| `ADMIN_USERNAME` | ✅ | Admin username (first run only) | **none** |
| `ADMIN_PASSWORD` | ✅ | Admin password (≥12 chars, no weak substrings) | **none** |
| `RUST_LOG` | optional | Log level: `info`, `debug`, `warn` | `info` |
| `SERVER_ADDR` | optional | Bind address | `0.0.0.0:3000` |
| `FORCE_SEED` | optional | `1` to wipe business data and re-seed demo (preserves users) | unset |

### Security checks (enforced at startup)

The app **refuses to start** if:

1. `DATABASE_URL` is missing, empty, or contains a placeholder (`PG_USER_REDACTED`, `ssdssd`, `admin1234`, `example`)
2. `ADMIN_PASSWORD` is shorter than 12 characters
3. `ADMIN_PASSWORD` contains weak substrings (`admin`, `password`, `123456`, `admin1234`, `letmein`, `qwerty`)

Example weak passwords that will be rejected:
```bash
# ❌ Rejected
ADMIN_PASSWORD=admin1234
ADMIN_PASSWORD=mypassword
ADMIN_PASSWORD=12345678

# ✅ Accepted
ADMIN_PASSWORD=$(openssl rand -base64 18)
```

### First-run vs subsequent runs

- **First run** (empty users table): admin is created from `ADMIN_USERNAME` + `ADMIN_PASSWORD`.
- **Subsequent runs**: the password is **always re-synced** from `ADMIN_PASSWORD` in `.env` (this is a deliberate choice for fresh deploys — `.env` is the single source of truth).
- **After the first login**, change the password from the **Users** tab in the UI. If you then restart the server, **your UI-set password will be overwritten** by the new `ADMIN_PASSWORD` env var. To make a UI-set password "stick" permanently, also update `ADMIN_PASSWORD` in `.env`.

---

## 📚 API Documentation (Swagger UI)

After starting the server:
- **Interactive docs**: <http://localhost:3000/swagger-ui> (or `/docs`)
- **OpenAPI JSON**: <http://localhost:3000/openapi.json>

---

## 🧪 Testing

```bash
# Unit tests (no DB required)
cargo test --lib

# Integration tests (require running PostgreSQL)
cargo test --test integration_test -- --test-threads=1
```

---

## 📡 API Quick Reference

### Public
| Method | Path | Description |
|--------|------|-------------|
| GET    | `/` | Web UI (static HTML) |
| GET    | `/static/*` | CSS/JS/assets |
| GET    | `/api/health` | Health check (no auth) |
| POST   | `/api/auth/login` | Login → Bearer token |
| POST   | `/api/auth/register` | Create non-admin user |
| GET    | `/swagger-ui` | Interactive API docs |
| GET    | `/openapi.json` | OpenAPI 3.0 spec |

### Protected (Bearer token)
| Method | Path | Description |
|--------|------|-------------|
| GET    | `/api/reports/dashboard` | Dashboard summary (counts + KPIs) |
| GET    | `/api/agents` | List agents (paginated) |
| POST   | `/api/agents` | Create agent |
| PATCH  | `/api/agents/{id}` | Update agent |
| GET    | `/api/customers` | List customers (paginated) |
| POST   | `/api/customers` | Create customer |
| GET    | `/api/interactions` | List interactions (paginated, filterable) |
| POST   | `/api/interactions` | Create interaction |
| GET    | `/api/interactions/{id}` | Get interaction detail |
| GET    | `/api/scores` | **Bulk** all scores (used by dashboard) |
| GET    | `/api/rubrics` | List rubrics |
| POST   | `/api/rubrics` | Create rubric |
| POST   | `/api/scoring/score` | Score an interaction |
| GET    | `/api/issues` | List issues (paginated) |
| POST   | `/api/issues` | Create issue |
| PATCH  | `/api/issues/{id}/status` | Update issue status (CAPA) |
| GET    | `/api/recommendations` | Auto-suggested QA queue |

All responses follow: `{ "success": true, "data": ... }` or `{ "success": false, "error": "..." }`.

---

## 📂 Project Structure

```
.
├── Cargo.toml
├── Cargo.lock
├── Dockerfile                  # Single-stage rust:1.88 build
├── docker-compose.yml          # db (postgres:16-alpine) + app
├── .env.example                # Template (NO real secrets)
├── .dockerignore
├── README.md
├── PLAN.md                     # Business plan (Persian)
├── src/
│   ├── main.rs                 # Bootstrap (fail-fast config checks + serve)
│   ├── lib.rs                  # AppState + router + seed_defaults (password validation)
│   ├── api.rs                  # axum route handlers
│   ├── service.rs              # Business logic
│   ├── auth.rs                 # bcrypt + session store + middleware
│   ├── store.rs                # SQLx CRUD with bulk fetches
│   ├── models.rs               # Domain types
│   ├── error.rs                # AppError + JSON error responses
│   └── openapi.rs              # Swagger UI (utoipa)
├── static/
│   ├── index.html
│   ├── app.css
│   ├── app.js
│   ├── chart.js                # 205KB (locally bundled)
│   ├── jquery.js               # 87KB (locally bundled)
│   ├── swagger-ui.css          # 186KB (locally bundled)
│   └── swagger-ui-bundle.js    # 1.5MB (locally bundled)
└── tests/
    ├── unit_test.rs
    └── integration_test.rs
```

---

## 🔄 Branches

- **`master`** — stable production-ready code
- **`dev`** — active development, gets all merged features first
- **`v1.0`** — frozen snapshot of the first public release

---

## 🎯 Use Cases

### Banking Call Center QA
Score every customer call against a custom rubric:
- **Compliance**: identity verification, account number privacy
- **Empathy**: greeting, active listening, closing
- **Resolution**: first-call resolution, follow-up commitment
- **Critical flags**: any compliance miss → automatic issue creation

### Insurance Claims
Evaluate agent interactions on:
- **Accuracy** of policy information
- **Empathy** in claims handling
- **Process adherence** to underwriting rules

### BPO Quality Management
Use CAPA workflow to track systemic issues:
- Open issue → root cause analysis → corrective action → close
- Track agent improvement over time with scoring history
- Export reports for client delivery

---

## 🛡️ Security Checklist for Production Deploy

Before exposing this app to the public internet:

- [ ] Generated strong `POSTGRES_PASSWORD` with `openssl rand -base64 24`
- [ ] Generated strong `ADMIN_PASSWORD` with `openssl rand -base64 18`
- [ ] Set `RUST_LOG=info` (NOT `debug` — leaks request details)
- [ ] Placed `.env` outside the web root (e.g. `/opt/crm-quality-inspector/.env`)
- [ ] `chmod 600 .env` (only owner can read)
- [ ] Fronted by a reverse proxy (nginx/Caddy) with HTTPS
- [ ] Firewall: only 80/443 open to the public; 5432 restricted to internal network
- [ ] Database backups scheduled (e.g. daily `pg_dump` to off-site)
- [ ] Logged-in as admin → changed the default password from the Users tab
- [ ] Removed or rotated any test/seed data before going live

---

## 🔮 Roadmap

- [ ] **Customer Risk Score** — auto-rank customers by churn probability (next release)
- [ ] AI-assisted scoring using LLM (suggest scores based on transcript)
- [ ] Real-time sentiment analysis (WebSocket stream)
- [ ] Multi-tenant support (organization isolation)
- [ ] Audit log + compliance trail
- [ ] Mobile-responsive UI for supervisors
- [ ] Integration with telephony (Asterisk AMI, FreeSWITCH)
- [ ] Persian speech-to-text (Vosk integration)
- [ ] Grafana metrics export

---

## 🤝 Contributing

Contributions welcome! Please:
1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Write tests for your changes
4. Commit (`git commit -m 'Add amazing feature'`)
5. Push and open a Pull Request against `dev`

---

## 📜 License

MIT — see [LICENSE](LICENSE).

---

## 🙏 Acknowledgments

Built with:
- [axum](https://github.com/tokio-rs/axum) — web framework
- [tokio](https://tokio.rs) — async runtime
- [sqlx](https://github.com/launchbadge/sqlx) — async PostgreSQL
- [bcrypt](https://github.com/Keats/rust-bcrypt) — password hashing
- [utoipa](https://github.com/juhaku/utoipa) — OpenAPI/Swagger generation
- [Chart.js](https://www.chartjs.org) — trend visualization

---

**Made with ❤️ in Rust for Persian-speaking contact centers**
