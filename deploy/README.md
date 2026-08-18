# 🚀 Directory: `deploy` (Production & VPS Deployment)

Konfigurasi containerisasi dan service daemon untuk menjalankan sistem trading autonomous di server Linux VPS (24/7).

## Berkas Tersedia:
- `Dockerfile`: Multi-stage build image Docker untuk binary Rust (`signal-daemon`, `api-server`, `scraper-worker`).
- `docker-compose.yml`: Komposisi service (TimescaleDB, Redis, Signal Daemon, API Server, UI Web Server).
- `systemd/`: Template unit file systemd untuk menjalankan binary tanpa Docker.

## Menjalankan dengan Docker Compose
```bash
docker compose -f deploy/docker-compose.yml up -d
```
