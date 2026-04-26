# ztnet-box

**Self-hosted web UI for ZeroTier — single Rust binary, zero dependencies.**

```bash
./ztnet-box          # → open http://127.0.0.1:3000
```

Manage your ZeroTier node, networks, and built-in controller entirely from a browser.
No cloud account required. No database, no Node.js, no Docker.

---

## Features

| | Feature | Description |
|-|---------|-------------|
| 🖥️ | **Dashboard** | Node status · peers · traffic metrics · one-click join |
| 🌐 | **Networks** | Join/leave ZeroTier Central and local controller networks |
| 🎛️ | **Controller** | Create networks, manage members, authorize devices |
| 🚀 | **Exit Node** | Full-tunnel VPN gateway (nftables/iptables, NDP proxy for IPv6) |
| 🌉 | **L2 Bridge** | Bridge ZeroTier to a physical LAN at Layer 2 |
| 🔁 | **TCP Relay** | Pylon relay deployment for UDP-blocked environments |
| 📊 | **Metrics** | Prometheus-backed traffic/latency charts |
| 📋 | **Log Panel** | Live log streaming with level filter |

## Quick Start

```bash
# Download latest release
curl -fsSL https://github.com/CleoWixom/ztnet-box/releases/latest/download/ztnet-box-linux-x86_64 \
  -o ztnet-box && chmod +x ztnet-box

# Run (ZeroTier must already be installed and running)
sudo ./ztnet-box
# → open http://127.0.0.1:3000
```

> **Note:** `sudo` is required when Exit Node, L2 Bridge, or TCP Relay features are used
> (firewall rule management needs root). For read-only / controller use, sudo is not needed.

## Documentation

| Guide | Description |
|-------|-------------|
| [Installation](docs/installation.md) | Binary, Docker, systemd service |
| [Configuration](docs/configuration.md) | `config.yml` reference |
| [Features](docs/features/) | Per-feature setup guides |
| [Development](docs/development.md) | Build from source, contribute |

## Configuration

Copy the example and adjust:

```bash
cp config.yml.example config.yml
./ztnet-box --config config.yml
```

See [docs/configuration.md](docs/configuration.md) for all options.

## License

[GPL-3.0](LICENSE)
