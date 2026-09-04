# CRM Quality Inspector 🛡️

[![Rust](https://img.shields.io/badge/rust-1.78%2B-orange?logo=rust)](https://www.rust-lang.org)
[![Axum](https://img.shields.io/badge/axum-0.7-blue)](https://github.com/tokio-rs/axum)
[![Redis](https://img.shields.io/badge/redis-6%2B-red?logo=redis)](https://redis.io)
[![License: MIT](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)

A modern, high-performance call center quality assurance (QA) platform built for **Persian/Farsi** contact centers in banking, insurance, and BPO industries. Features customizable rubrics, weighted scoring, CAPA (Corrective and Preventive Action) workflow, real-time dashboard, and AI-assisted coaching.

---

## ✨ Features

### 🎯 Core
- **Multi-dimensional scoring** with weighted criteria (each criterion can be marked "critical")
- **Smart rubric auto-selection** based on agent department and customer product type
- **Critical fail detection** — automatically flags interactions where any critical criterion scores below 60
- **CAPA workflow** — root cause analysis + corrective action tracking for every issue
- **Hierarchical grading** — عالی / خوب / نیازمند بهبود / ضعیف based on overall score

### 📊 Analytics
- Real-time dashboard with 8 KPIs (agents, customers, interactions, coverage %, average quality, open issues, critical fails, quality grade)
- Quality trend chart (Chart.js) showing score progression over time
- Per-agent performance reports with score history
- Coverage tracking: % of interactions evaluated

### 🛠️ Productivity
- **Full-text search** in interactions (subject + transcript + tags)
- **Lazy load** per tab + localStorage cache (60s TTL) for sub-second repeat loads
- **CSV export** of all interactions with scores (UTF-8 BOM for Excel Persian support)
- **Smart QA recommendations** — auto-suggest high-priority interactions to evaluate based on customer segment, channel, and agent performance

### 🎨 UX
- Modern dark RTL interface (Linear/Vercel-inspired)
- Persian/Farsi native support
- Custom Q logo with gradient favicon
- Modal-based scoring with 7-slider rubric
- Filter interactions by channel, agent, status

### 🔐 Security
- Session-based auth (Bearer token, 24h TTL)
- Password hashing with SHA-256 (production should use argon2)
- CORS-enabled, ready for SPA frontends
- Middleware-protected routes

---

## 🏗️ Architecture

```
┌─────────────────────────────────────────────────────┐
│              Browser (RTL, Dark Theme)             │
│   Vanilla JS + Chart.js (CDN) + localStorage cache  │
└──────────────────┬──────────────────────────────────┘
                   │ HTTP/JSON (Bearer token)
                   ▼
┌─────────────────────────────────────────────────────┐
│         Axum 0.7 (async, tokio multi-thread)        │
│   ┌──────────┬─────────────┬────────────────────┐  │
│   │  api.rs  │ service.rs  │     auth.rs        │  │
│   │  routes  │ business    │  session + middleware│ │
│   └──────────┴─────────────┴────────────────────┘  │
│         ┌─────────────┬──────────────┐             │
│         │  store.rs   │  models.rs   │             │
│         │  Redis ops  │  Domain types│             │
│         └─────────────┴──────────────┘             │
└──────────────────┬──────────────────────────────────┘
                   │ redis-rs (async, multiplexed)
                   ▼
         ┌──────────────────┐
         │  Redis 6+        │
         │  (any host)      │
         └──────────────────┘
```

**Stack**:
- **Backend**: Rust 1.78+, axum 0.7, tokio, tower-http, redis (async), serde, sha2, chrono
- **Frontend**: Vanilla JS (no framework), Chart.js 4.4 via CDN
- **Storage**: Redis 6+ (remote or local)
- **Test**: 6 unit tests + 3 integration tests (with `--test-threads=1`)

---

## 🚀 Quick Start

### Prerequisites
- Rust 1.78 or newer ([install](https://rustup.rs))
- PostgreSQL 14+ accessible at `127.0.0.1:5432` (or set `DATABASE_URL` env var)

### Run

```bash
git clone git@github.com:Saeeddn/crm-quality-inspector.git
cd crm-quality-inspector
cp .env.example .env
# Edit .env and set DATABASE_URL + ADMIN_PASSWORD
cargo build --release
./target/release/crm-quality-inspector
```

Open http://localhost:3000 in your browser.

**First-run login** is created from `ADMIN_USERNAME` / `ADMIN_PASSWORD` in `.env`.
On every subsequent start, env vars are ignored — use the **Users** tab in the UI to change passwords.

⚠️ **No default password.** If `ADMIN_USERNAME` or `ADMIN_PASSWORD` is missing, the server refuses to start with a clear error.

### 🐳 Docker (one-line deploy)

```bash
cp .env.example .env
# Edit .env: set POSTGRES_PASSWORD and ADMIN_PASSWORD
docker compose up -d --build
# Wait ~30s for the first build, then open http://localhost:3000
```

This brings up PostgreSQL + the app in one command. Data persists in the `crm_qi_pgdata` named volume.

**Production single-server (no Docker):**
```bash
cargo build --release
# Set env vars in your shell or systemd unit
DATABASE_URL=postgres://user:pass@127.0.0.1:5432/crm_qi \
ADMIN_USERNAME=admin \
ADMIN_PASSWORD='your-strong-pass' \
./target/release/crm-quality-inspector
```

### Configuration

All config is via environment variables (or a `.env` file you create from `.env.example`):
- `DATABASE_URL` — PostgreSQL connection string (default: localhost with placeholder password)
- `ADMIN_USERNAME` — Admin username (default: `admin`)
- `ADMIN_PASSWORD` — Admin password (default: `ADMIN_PASS_REDACTED` — **insecure, override in prod!**)

### 📚 API Documentation (Swagger UI)

After starting the server, open:
- Interactive docs: <http://localhost:3000/swagger-ui> (or `/docs`)
- OpenAPI JSON spec: <http://localhost:3000/openapi.json>

---

## 🧪 Testing

```bash
# Unit tests (no Redis required)
cargo test --test unit_test

# Integration tests (require Redis)
cargo test --test integration_test -- --test-threads=1
```

The integration test suite covers the full QA workflow:
- User authentication (admin auto-create on first login)
- Agent / customer / interaction CRUD
- Rubric creation and scoring end-to-end
- Issue workflow with CAPA

---

## 📡 API

### Public
| Method | Path                  | Description                |
|--------|-----------------------|----------------------------|
| GET    | `/`                   | Web UI (static HTML)       |
| GET    | `/static/*`           | CSS/JS/assets              |
| POST   | `/api/auth/login`     | Login, returns Bearer token |
| POST   | `/api/auth/register`  | Create non-admin user      |
| GET    | `/api/health`         | Health check               |

### Protected (Bearer token required)
| Method | Path                            | Description                  |
|--------|---------------------------------|------------------------------|
| GET    | `/api/agents`                   | List all agents              |
| POST   | `/api/agents`                   | Create agent                 |
| PATCH  | `/api/agents/{id}`              | Update agent (active flag)   |
| GET    | `/api/customers`                | List customers               |
| POST   | `/api/customers`                | Create customer              |
| GET    | `/api/interactions`             | List (filter: agent, channel, date) |
| POST   | `/api/interactions`             | Create interaction           |
| GET    | `/api/interactions/{id}`        | Get interaction detail       |
| GET    | `/api/rubrics`                  | List evaluation rubrics      |
| POST   | `/api/rubrics`                  | Create rubric                |
| POST   | `/api/scoring/score`            | Score an interaction         |
| GET    | `/api/scoring/{id}`             | Get score for interaction    |
| GET    | `/api/issues`                   | List issues (filter: status, severity) |
| POST   | `/api/issues`                   | Create issue                 |
| PATCH  | `/api/issues/{id}/status`       | Update issue status (CAPA)   |
| GET    | `/api/reports/dashboard`        | Dashboard summary            |
| GET    | `/api/reports/agent/{id}`       | Per-agent report             |
| GET    | `/api/recommendations`          | Auto-suggested QA queue      |

All responses follow: `{ "success": true, "data": ... }` or `{ "success": false, "error": "..." }`.

---

## 📂 Project Structure

```
.
├── Cargo.toml
├── PLAN.md                  # Business plan (Persian)
├── README.md                # This file
├── src/
│   ├── main.rs              # Bootstrap (tracing + serve)
│   ├── lib.rs               # AppState + router assembly
│   ├── api.rs               # axum route handlers
│   ├── service.rs           # Business logic (login, scoring, reports)
│   ├── auth.rs              # Session store + middleware
│   ├── store.rs             # Redis CRUD with MGET batch fetches
│   ├── models.rs            # Domain types (Agent, Interaction, Score, ...)
│   └── error.rs             # AppError + JSON error responses
├── static/
│   ├── index.html           # Single-page app shell
│   ├── app.css              # Dark RTL theme
│   ├── app.js               # Frontend logic (lazy load, cache, Chart.js)
│   └── favicon.svg          # Q logo (gradient)
└── tests/
    ├── unit_test.rs         # Unit tests (no external deps)
    └── integration_test.rs  # E2E tests against Redis
```

---

## 🎯 Use Cases

### 1. Banking Call Center QA
Score every customer call against a custom rubric:
- **Compliance**: identity verification, account number privacy
- **Empathy**: greeting, active listening, closing
- **Resolution**: first-call resolution, follow-up commitment
- **Critical flags**: any compliance miss → automatic issue creation

### 2. Insurance Claims
Evaluate agent interactions on:
- **Accuracy** of policy information
- **Empathy** in claims handling
- **Process adherence** to underwriting rules

### 3. BPO Quality Management
Use CAPA workflow to track systemic issues:
- Open issue → root cause analysis → corrective action → close
- Track agent improvement over time with scoring history
- Export reports for client delivery

---

## 🔮 Roadmap

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
5. Push and open a Pull Request

---

## 📜 License

MIT — see [LICENSE](LICENSE).

---

## 🙏 Acknowledgments

Built with:
- [axum](https://github.com/tokio-rs/axum) — web framework
- [tokio](https://tokio.rs) — async runtime
- [redis-rs](https://github.com/redis-rs/redis-rs) — Redis client
- [Chart.js](https://www.chartjs.org) — trend visualization

---

**Made with ❤️ in Rust for Persian-speaking contact centers**
