# Configuration Reference

`config.yml` is loaded from the **first existing** path:

1. `$ZTNET_BOX_CONFIG` env var
2. `./config.yml`
3. `~/.config/ztnet-box/config.yml`
4. `/etc/ztnet-box/config.yml`

All settings are editable through **Settings → Global** in the UI.

## Full reference

```yaml
server:
  host: "127.0.0.1"   # bind address — keep loopback unless behind a reverse proxy
  port: 3000

zerotier:
  local:
    api_url: "http://127.0.0.1:9993"
    token_file: "/var/lib/zerotier-one/authtoken.secret"
  central:
    base_url: "https://api.zerotier.com/api/v1"
    tokens: []           # managed via Settings → API Tokens
    active_token_id: ""  # set automatically when a token is added

metrics:
  enabled: false         # opt-in — enable if ZeroTier ≥ 1.14 is running
  prometheus_url: "http://127.0.0.1:9993/metrics"
  poll_interval_seconds: 15
  metricstoken_file: "/var/lib/zerotier-one/metricstoken.secret"

exitnode:
  nftables_preferred: true   # true = nftables, false = iptables
```

## Environment variable overrides

| Variable | Config key | Default |
|---|---|---|
| `ZT_SERVER_HOST` | `server.host` | `127.0.0.1` |
| `ZT_SERVER_PORT` | `server.port` | `3000` |
| `ZT_LOCAL_API_URL` | `zerotier.local.api_url` | `http://127.0.0.1:9993` |
| `ZT_LOCAL_TOKEN_FILE` | `zerotier.local.token_file` | `/var/lib/zerotier-one/authtoken.secret` |
| `ZT_CENTRAL_BASE_URL` | `zerotier.central.base_url` | `https://api.zerotier.com/api/v1` |
| `ZTNET_SKIP_DEPS` | — | Bypass startup ZeroTier dependency check (useful in CI) |
| `ZTNET_STATE_FILE` | — | Override path for `state.json` |
