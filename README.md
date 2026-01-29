# qrlink - URL Shortener & Branded QR Code Generator

A self-hosted URL shortener written in Rust 🦀 designed for on-premise environments.

## Features

- **URL Shortening**: Create short links with random, URL-safe codes (7 characters of [Base56](https://en.wikipedia.org/wiki/Binary-to-text_encoding#Examples))
- **Time-to-Live (TTL)**: Optional expiration with presets (1 week, 1 month, 1 year, never)
- **QR Code Generation**: Automatic QR codes with corporate branding
- **SQLite Database**: Zero-dependency, single-file persistence

## Quick Start

### Using Docker Compose

```bash
docker compose up -d
```

### Using Cargo

```bash
cargo run
```

The defaults work out of the box:
- Database: `sqlite:data/shortener.db`
- Base URL: `http://localhost:8080`
- QR Logo: `assets/logo.svg`

## API Documentation

Full API usage examples are in [api-usage.md](docs/api-usage.md).

## Configuration

All configuration is via environment variables:

| Variable | Default | Description |
|----------|---------|-------------|
| `DATABASE_URL` | `sqlite:data/shortener.db` | SQLite database path |
| `BASE_URL` | `http://localhost:8080` | Public base URL for short links |
| `HOST` | `0.0.0.0` | Server bind address |
| `PORT` | `8080` | Server port |
| `QR_BRANDING_LOGO` | `assets/logo.svg` | Path to logo for QR codes (PNG/SVG) |
| `QR_SIZE` | `512` | QR code size in pixels |

## Deployment

### Custom logo in container

- Mount your logo into the container and set `QR_BRANDING_LOGO` to that path.
- Example: mount `/opt/logos/logo.svg` to `/app/assets/logo.svg` and set `QR_BRANDING_LOGO=/app/assets/logo.svg`.

### Real public URL

- Set `BASE_URL` to your public domain, e.g. `https://qrlink.domain.com`.
- Put the container behind a reverse proxy (Nginx, Traefik, Caddy) and point DNS to it.

### Backups

- The Compose file includes an **optional** backup helper for bare Docker setups.
- If you're deploying on Kubernetes or any platform with managed backups, you can ignore it.

## Architecture

```
src/
├── main.rs           # Application entry point
├── config.rs         # Environment configuration
├── domain.rs         # Business logic (Link, TTL)
├── service.rs        # Use cases (LinkService, QrService)
├── repository.rs     # Database access (SQLite)
├── http.rs           # REST API handlers and routing
├── qr.rs             # QR code generation with branding
└── error.rs          # Error types
```

## License

MIT