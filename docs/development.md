# Development

## Prerequisites

- Rust 1.75+ — install via [rustup](https://rustup.rs)
- ZeroTier daemon running locally (for integration tests)
- Node.js (only if modifying the frontend build pipeline — not required for normal dev)

## Build

```bash
cargo build          # debug
cargo build --release
```

The frontend (HTML/CSS/JS) is compiled into the binary by `build.rs`. Changing any file under `www/src/` and rebuilding picks up changes automatically.

## Run

```bash
sudo ./target/debug/ztnet-box
# → http://127.0.0.1:3000
```

## Tests

```bash
# Fast unit + API smoke tests (no ZeroTier required)
cargo test

# Integration tests against real ZeroTier daemon
sudo ZT_RUNNING=1 cargo test --test api_local

# Integration tests against ZeroTier Central
ZT_CENTRAL_TOKEN=<your_token> cargo test --test api_central

# Use a specific test network for join/leave tests
ZT_TEST_NETWORK=8056c2e21c000001 sudo cargo test --test api_local local_network
```

## CI / Workflows

| Workflow | Trigger | Purpose |
|----------|---------|---------|
| `check-and-lint.yml` | push/PR | `cargo fmt`, `clippy`, `cargo test` |
| `release.yml` | tag `v*` | build release binary, create GitHub release |
| `screenshots.yml` | manual | capture UI screenshots with Playwright |
| `manual-testing.yml` | manual | run ztnet-box with public tunnel (cloudflared/localtunnel) |

## Project Structure

```
src/
  server/         # Axum HTTP server, routes, handlers
  zerotier/       # ZeroTier local daemon + Central API clients
  config/         # Config schema, token store, persistence
  exitnode/       # Exit node platform detection + firewall rules
  bridge/         # L2 bridge systemd-networkd integration
  physnet/        # Physical routing (ip rule/route management)
  relay/          # TCP relay (Pylon) management

www/src/
  html/           # shell.html — SPA shell
  css/            # variables, layout, components, pages
  js/
    pages/        # One module per page (dashboard, networks, …)
    components/   # Shared components (log-panel, modal, toast)
    api.js        # fetch wrapper with timeout
    router.js     # Hash-based SPA router
    state.js      # Simple key-value app state + Utils
    z-init.js     # App bootstrap (routes, sidebar, status poll)

tests/
  api_health.rs   # Smoke tests (no ZeroTier required)
  api_local.rs    # Integration tests (real ZT daemon)
  api_central.rs  # Integration tests (real Central API)
```
