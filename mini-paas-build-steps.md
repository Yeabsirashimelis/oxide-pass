# Mini PaaS in Rust — Step-by-Step Build Guide

This document is an **opinionated, practical roadmap** to build a **Mini PaaS (Platform as a Service)** in Rust.
Follow the steps **in order**. Do NOT skip ahead.

---

## What You Are Building (One Sentence)

A local system that **runs, monitors, restarts, and exposes web apps for you**, similar to a tiny Heroku — written entirely in Rust.

---

## Core Rule (IMPORTANT)

> Start **local, single-node, process-based**.  
> No Docker. No Kubernetes. No clusters.

---

## Tech Stack (Locked)

- Language: **Rust (stable)**
- Async runtime: **Tokio**
- HTTP server: **Actix Web**
- CLI: **clap**
- Database: **SQLite** (via sqlx)
- Logging: **tracing**
- Serialization: **serde**
- Process management: **tokio::process::Command**

---

## System Components (V1)

You will build **three programs**:

1. `paasd` — Control Plane (API server)
2. `agent` — Node Agent (runs apps)
3. `paas` — CLI tool

All in **one Cargo workspace**.

---

## Repo Structure (Create First)

```
mini-paas/
├─ crates/
│  ├─ paasd/        # Control plane (Actix)
│  ├─ agent/        # Node agent (process runner)
│  ├─ paas/         # CLI
│  └─ shared/       # Shared types
├─ Cargo.toml
└─ README.md
```

---

## STEP 1 — Control Plane (Minimal)

### Goal
Accept app registrations and track their state.

### Tasks
- Create Actix Web server
- Create SQLite database
- Define App model:
  - id
  - name
  - command
  - status (pending, running, failed, stopped)
  - port

### Endpoints
- POST /apps
- GET /apps
- GET /apps/{id}
- PATCH /apps/{id}

### Output
You can register apps and see them via curl or HTTP client.

---

## STEP 2 — CLI Tool

### Goal
User can deploy apps via CLI.

### Tasks
- Create `paas` CLI with clap
- Commands:
  - `deploy <command>`
  - `status`
- CLI sends HTTP requests to `paasd`

### Output
Running:
```
paas deploy "./my-app"
```
creates an app entry in the control plane.

---

## STEP 3 — Node Agent (Core System Part)

### Goal
Run apps as OS processes and track them.

### Tasks
- Agent polls control plane for apps with status = pending
- Agent spawns process using tokio::process::Command
- Capture:
  - PID
  - stdout
  - stderr
- Update app status to running

### Output
App process actually runs on your machine.

---

## STEP 4 — App Lifecycle Management

### Goal
Keep apps alive.

### Tasks
- Detect process exit
- Restart on crash
- Handle SIGINT / SIGTERM
- Track exit codes
- Status transitions:
  - pending → running → failed → restarting

### Output
If app crashes, it restarts automatically.

---

## STEP 5 — Port Management & Routing

### Goal
Expose apps over HTTP.

### Tasks
- Agent assigns free ports
- Store port in control plane
- Implement simple reverse proxy:
  - /apps/{name} → localhost:{port}

### Output
You can open the deployed app in a browser.

---

## STEP 6 — Logs

### Goal
View app logs via CLI.

### Tasks
- Stream stdout/stderr
- Store logs in memory or file
- API endpoint to fetch logs
- CLI command:
  - `paas logs <app>`

---

## FIRST MAJOR MILESTONE

At this point you should be able to:
- Deploy a web app
- See it running
- Restart it on crash
- Access it via browser
- View logs

This is already a **serious system**.

---

## What NOT To Do Early

- Docker
- Kubernetes
- Multi-node scheduling
- Auth systems
- Web dashboards

Add these only AFTER V1 works.

---

## Mental Model

- Control Plane = Brain
- Agent = Hands
- CLI = Mouth
- App = Child process

---

## Stretch Goals (Later)

- Docker support
- Multi-node scheduling
- Resource limits (CPU / memory)
- Metrics (Prometheus)
- Web dashboard
- Zero-downtime restarts

---

## Final Advice

Build **slowly**, test each step, and never move on until the current step works fully.

This project is about **depth, not speed**.
