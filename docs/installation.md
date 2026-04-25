# Installation

## Binary (Linux)

```bash
curl -fsSL https://github.com/CleoWixom/ztnet-box/releases/latest/download/ztnet-box-linux-x86_64 \
  -o ztnet-box && chmod +x ztnet-box
sudo ./ztnet-box
```

ZeroTier must be installed separately. If it is not present, the Dashboard shows an **Install ZeroTier** button that installs it automatically.

## Build from Source

```bash
git clone https://github.com/CleoWixom/ztnet-box
cd ztnet-box
cargo build --release
sudo ./target/release/ztnet-box
```

Requires Rust 1.75+ (`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`).

## systemd Service

```ini
# /etc/systemd/system/ztnet-box.service
[Unit]
Description=ZTNetwork Panel
After=network.target zerotier-one.service
Requires=zerotier-one.service

[Service]
ExecStart=/usr/local/bin/ztnet-box --config /etc/ztnet-box/config.yml
Restart=on-failure
RestartSec=5

[Install]
WantedBy=multi-user.target
```

```bash
sudo systemctl enable --now ztnet-box
```

> **Note:** `sudo` / root is required only when using Exit Node, L2 Bridge, or Physical Routing features. For controller-only or network management, a non-root user with read access to `/var/lib/zerotier-one/authtoken.secret` is sufficient.
