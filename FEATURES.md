# PaaS - Planned Features

This file tracks upcoming features to implement in the PaaS platform.
Focused on JS/TS ecosystem (Next.js, Express, Vite, etc.)

---

## ✅ Completed Features
- Deploy apps (Node, Go, Python, Rust)
- Stop apps (kills entire process tree)
- Redeploy apps
- Live process status check
- Log streaming with `--follow`
- PID tracking
- Port conflict detection
- Auto-cleanup stale records on startup
- Environment variables support (`[env]` in paas.toml)
- Auto-import env vars from `.env` file on `paas init`
- Auto-restart on crash (max 3 retries, then marks as CRASHED)
- Detect actual running port from app stdout

---

## 🔜 Next Features (Priority Order)

### 1. 📋 Log Retention / Cleanup
**Why:** Logs grow forever in DB. After a week of running apps you'll have millions of rows.
**What:**
- Keep only last 1000 log entries per app
- Delete logs older than 7 days
- Run cleanup on paasd startup and every 24h
**Files:** `paasd/src/repository/log_repo.rs`, `paasd/src/main.rs`

---

### 2. 🔑 API Key Authentication
**Why:** Anyone on the network can deploy/stop apps right now.
**What:**
- Generate API key on `paas init` and store in `paas.toml`
- CLI sends key as `X-API-Key` header on every request
- paasd validates the key before processing any request
- Keys stored in a `api_keys` table in DB
**Files:** `paas/src/commands/*.rs`, `paasd/src/handlers/`, new middleware

---

### 3. 🏥 Health Checks
**Why:** Know if your app is actually responding, not just running.
**What:**
- paasd pings `http://localhost:<port>/health` every 30s
- If it fails 3 times consecutively, mark as `UNHEALTHY` and restart
- `paas status` shows health check result
- Configurable health check path in `paas.toml`: `health_check = "/api/health"`
**Files:** `paasd/src/main.rs` (background task), `shared/src/lib.rs` (new status)

---

### 4. 🌍 Remote Deployment
**Why:** Currently paasd and agent must run on the same machine as the user.
**What:**
- `paas.toml` has a `server` field pointing to remote paasd URL
- CLI talks to remote paasd instead of localhost
- Example: `server = "http://my-vps.com:8080"`
**Files:** All CLI commands (replace hardcoded `127.0.0.1:8080`)

---

### 5. 📊 Resource Limits (Advanced)
**Why:** Prevent one app from eating all CPU/RAM.
**What:**
- Set memory/CPU limits per app in `paas.toml`
- Agent enforces limits using OS-level mechanisms
- `paas status` shows resource usage
**Files:** `agent/src/main.rs`, `shared/src/lib.rs`

---

### 6. 🔄 Zero-downtime Redeploy (Advanced)
**Why:** Current redeploy kills the app then starts a new one (brief downtime).
**What:**
- Start new process on a temp port
- Wait for it to be healthy
- Switch traffic to new process
- Kill old process
**Files:** `agent/src/main.rs`, `paasd/src/handlers/app_handlers.rs`

---

## 💡 Nice-to-Have
- `paas list` - list all deployed apps across projects
- `paas logs --tail N` - show last N lines
- `paas open` - open app in browser automatically
- Colorized log output (stdout = white, stderr = red)
- `paas ps` - show all running processes with CPU/RAM usage