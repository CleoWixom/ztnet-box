# Exit Node

Route all ZeroTier peer traffic through this machine — full-tunnel VPN gateway.

## How it works

```
ZT peer ──[zerotier interface zt*]──► MASQUERADE ──[WAN eth0]──► Internet
```

ZTNetwork Panel applies:
- `iptables -t nat -A POSTROUTING -o <wan> -j MASQUERADE`
- `iptables -A FORWARD -i <zt> -o <wan> -j ACCEPT` (and reverse)
- `sysctl net.ipv4.ip_forward=1`

Rules are written to `/etc/iptables/rules.v4` (or nftables equivalent) for reboot persistence.

## Requirements

| Requirement | Notes |
|---|---|
| **OS** | Linux only |
| **Privileges** | root (`sudo`) |
| **Firewall** | `iptables ≥ 1.8` or `nftables ≥ 0.9` |
| **Kernel** | `ip_forward` enabled (done automatically) |
| **ZeroTier** | Network with `allowDefault=1` and `allowGlobal=1` on client nodes |

## Setup

1. Open **Exit Node** in the sidebar
2. Select the **ZeroTier interface** (e.g. `zt7nnig26`) — the `zt*` interface, NOT the network ID
3. Select the **WAN interface** (e.g. `eth0`, `enp3s0`)
4. Optionally enable **IPv6** with an NDP proxy prefix
5. Click **Enable**

On client nodes, set in ZeroTier Central → network settings:
```
allowDefault = 1
allowGlobal  = 1
```

## IPv6 via NDP Proxy

For native IPv6 (no NAT), ZTNetwork Panel installs and configures [ndppd](https://github.com/DanielAdolfsson/ndppd). Supports apt, dnf, and pacman.

## Diagnostics

```bash
# Verify from a client — should show the exit node's IP
curl https://ipinfo.io/ip

# Check iptables rules on the exit node
sudo iptables -t nat -L POSTROUTING -v -n

# Check ip_forward
cat /proc/sys/net/ipv4/ip_forward   # should be 1

# Check ZeroTier interface
ip addr show | grep zt
```

## Troubleshooting

| Symptom | Likely cause | Fix |
|---|---|---|
| Client traffic not routed | `allowDefault`/`allowGlobal` not set | Enable in ZeroTier Central for the network |
| `iptables: No chain/target/match` | iptables version mismatch | Install `iptables-legacy` or switch to nftables in config |
| IPv6 not working | ndppd not installed | Click "Install ndppd" on the Exit Node page |
| Rules lost after reboot | iptables-persistent not installed | Run: `sudo apt install iptables-persistent` |
