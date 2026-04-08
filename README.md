# ztnet-box

Local web UI for managing ZeroTier: join/leave networks, control the built-in network controller, manage members, view metrics, configure Exit Node and root servers — all from a single self-contained binary.

## Requirements

- **ZeroTier One** daemon (`zerotier-one`) running on the host — or let ztnet-box install it automatically via the Settings UI
- **Root / Administrator** privileges for Exit Node features (iptables/nftables manipulation)
- Linux, macOS, or Windows (Exit Node is Linux-only)

## Installation

Download the binary for your platform from [Releases](https://github.com/CleoWixom/ztnet-box/releases):

| Platform | Archive |
|---|---|
| Linux x86_64 | `ztnet-box-x86_64-unknown-linux-gnu.tar.gz` |
| Linux ARM64 | `ztnet-box-aarch64-unknown-linux-gnu.tar.gz` |
| macOS Intel | `ztnet-box-x86_64-apple-darwin.tar.gz` |
| macOS Apple Silicon | `ztnet-box-aarch64-apple-darwin.tar.gz` |
| Windows | `ztnet-box-x86_64-pc-windows-msvc.zip` |

Or build from source:

```bash
cargo build --release
```

## Running

```bash
cp config.yml.example config.yml
# Edit config.yml as needed
./ztnet-box
# Open http://127.0.0.1:3000
```

## Configuration

All settings can be changed via the **Settings** page in the UI. The config file (`config.yml`) is loaded from the first existing path among:

1. `./config.yml`
2. `~/.config/ztnet-box/config.yml`
3. `/etc/ztnet-box/config.yml`

| Parameter | ENV override | Default | Description |
|---|---|---|---|
| `server.host` | `ZT_SERVER_HOST` | `127.0.0.1` | Bind address |
| `server.port` | `ZT_SERVER_PORT` | `3000` | HTTP port |
| `zerotier.local.api_url` | `ZT_LOCAL_API_URL` | `http://127.0.0.1:9993` | ZeroTier One service URL |
| `zerotier.local.token_file` | `ZT_LOCAL_TOKEN_FILE` | `/var/lib/zerotier-one/authtoken.secret` | Local API auth token file |
| `zerotier.central.base_url` | `ZT_CENTRAL_BASE_URL` | `https://api.zerotier.com/api/v1` | ZeroTier Central base URL |
| `metrics.enabled` | — | `false` | Enable Prometheus metrics polling |
| `metrics.prometheus_url` | — | `http://127.0.0.1:9000/metrics` | Prometheus endpoint |
| `metrics.poll_interval_seconds` | — | `15` | Poll interval |
| `exitnode.nftables_preferred` | — | `true` | Prefer nftables over iptables |

## Security Model

ztnet-box has **no authentication**. This is intentional — the server binds to `127.0.0.1` by default, making it accessible only from the local machine. Security is enforced at the network level.

If you change `server.host` to a non-loopback address, ztnet-box will log a warning at startup. In that case, ensure access is restricted via firewall rules or a reverse proxy with authentication.

No cookies are set, so CSRF is not applicable. All API responses include `X-Content-Type-Options`, `X-Frame-Options`, `Content-Security-Policy`, and `Referrer-Policy` headers. Request bodies are limited to 64 KB.

## Central API Tokens

To manage networks via ZeroTier Central:

1. Go to **Settings → API Tokens**
2. Paste your token from [my.zerotier.com](https://my.zerotier.com)
3. ztnet-box validates the token and detects your rate limit (Free: 20 req/s, Paid: 100 req/s)
4. The first token added becomes active automatically

Multiple tokens are supported — use **Activate** to switch between accounts.

## Exit Node

Exit Node routes all traffic from ZeroTier peers through this machine to the internet (full-tunnel VPN).

Requirements:
- Linux only
- Must run as root
- `iptables` or `nftables` installed

Enable via **Exit Node** in the UI: select the ZeroTier network interface and the WAN interface, then click **Enable**. Disable restores the original firewall state.

## Changelog

See [CHANGELOG.md](./CHANGELOG.md).
