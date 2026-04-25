# Configuration Reference

ztnet-box reads `config.yml` on startup. All fields have defaults; the file is optional.

```bash
./ztnet-box --config /path/to/config.yml
```

## Full Example

See [`config.yml.example`](../config.yml.example) in the repository root — it is the authoritative reference and always up to date.

## Key Fields

### `server`

| Field | Default | Description |
|-------|---------|-------------|
| `host` | `127.0.0.1` | Bind address. Change to `0.0.0.0` to expose on the network (no auth — use a firewall) |
| `port` | `3000` | HTTP port |

### `zerotier.local`

| Field | Default | Description |
|-------|---------|-------------|
| `api_url` | `http://127.0.0.1:9993` | ZeroTier daemon API URL |
| `token_file` | `/var/lib/zerotier-one/authtoken.secret` | Path to the ZeroTier auth token |

### `zerotier.central`

| Field | Default | Description |
|-------|---------|-------------|
| `base_url` | `https://api.zerotier.com/api/v1` | Central API base URL (change for self-hosted) |
| `tokens` | `[]` | Managed via Settings → API Tokens in the UI |

### `metrics`

| Field | Default | Description |
|-------|---------|-------------|
| `enabled` | `true` | Enable Prometheus metrics collection |
| `prometheus_url` | `http://127.0.0.1:9993/metrics` | ZeroTier metrics endpoint (ZT 1.14+) |
| `poll_interval_seconds` | `15` | How often to scrape |
| `metricstoken_file` | `/var/lib/zerotier-one/metricstoken.secret` | Separate metrics auth token (ZT 1.16.1+) |
