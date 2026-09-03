# Static File Serving: `include_str!` for HTML vs `ServeDir` for JS/CSS

## The Architecture

A common Axum static-file setup uses two different strategies:

| File type | Serving method | Source |
|-----------|---------------|--------|
| `index.html` | `include_str!("../static/index.html")` | Embedded in binary at compile time |
| `app.js`, `app.css`, `*.png` | `tokio::fs::read("static/{path}")` | Served from disk at runtime |

This split is intentional: `include_str!` gives a single portable binary; `tokio::fs::read` lets JS/CSS be edited without rebuilding.

## The Trap

**`include_str!` bakes the HTML at compile time.** When you edit `static/index.html` (e.g. to bump `app.js?v=7` → `v=8`), the running binary still serves the OLD content.

**`app.js` is served from disk**, so `curl /static/app.js` shows the new version.

This creates a confusing state:
```
curl /                   → <script src="/static/app.js?v=7">   ← embedded binary
curl /static/app.js      → v=8 in the file                       ← served from disk
```

**Symptom**: User says "charts don't render" or "new feature not visible". You `curl /` and see old content. But `curl /static/app.js` shows new code. You're confused — the server is serving inconsistent versions.

## Why This Happened (Chart.js example)

The `index.html` was edited to add `<script src="/static/chart.umd.min.js">` and bump `v=8`. But the binary was never rebuilt with `cargo build`, so `curl /` still showed `v=7` and CDN links. Meanwhile `curl /static/app.js` showed `v=10` (a later edit). The HTML and JS versions were completely out of sync.

## The Fix

**Always rebuild after editing `index.html`:**

```bash
# 1. Touch a Rust source to force recompilation
python -c "open('src/lib.rs','a').write('\n')"

# 2. Kill the running server
taskkill /F /IM crm-quality-inspector.exe

# 3. Rebuild
cargo build

# 4. Restart
./target/debug/crm-quality-inspector
```

**Alternative (better for active development)**: Replace `include_str!` with `tower_http::services::ServeDir` for HTML too. Then all static files are served from disk and no rebuild is needed for HTML changes.

## Diagnostic

When user reports a feature isn't showing:

```bash
# 1. Check what HTML the server serves
curl -s http://localhost:3000/ | grep "app.js\|chart\|jquery"

# 2. Check what's on disk
grep "app.js\|chart\|jquery" static/index.html

# 3. If they differ → include_str! trap → rebuild required
# 4. If they match → check browser cache (Ctrl+Shift+R)
```

## Prevention

Add to your workflow habit: **"Edit HTML → rebuild binary → restart server."**
Only JS/CSS changes need no rebuild (they're served from disk).

In the CRM project specifically, `src/api.rs` line 49:
```rust
let html = include_str!("../static/index.html");
```

This is the ONLY place HTML is embedded. Any edit to `static/index.html` requires a rebuild.
