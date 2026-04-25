# Exit Node

Route all ZeroTier peer traffic through this machine — a self-hosted full-tunnel VPN gateway.

## How It Works

1. Members that set **Allow Default Route** on a shared network send all internet traffic to this node
2. ztnet-box configures `nftables` (or `iptables`) MASQUERADE rules and enables IP forwarding
3. Traffic exits through the WAN interface of this machine

## Requirements

- Linux only
- `nftables` or `iptables` installed
- Running as root (`sudo`)
- A dedicated WAN interface with internet access

## Setup

1. Open **Gateway → Exit Node** in the UI
2. Install missing dependencies if shown
3. Select ZeroTier interface (`zt…`), WAN interface, and optionally the network
4. Click **Enable Exit Node**
5. In your controller: set **Allow Default Route** for the network
6. On member devices: enable **Default Route** for the network

## IPv6 / NDP Proxy

Enable **NDP Proxy** (ndppd) to provide real IPv6 routing without MASQUERADE. Requires `ndppd` installed and a publicly routable `/64` prefix delegated to this machine.
