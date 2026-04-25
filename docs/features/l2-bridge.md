# L2 Bridge

Connect a ZeroTier network to a physical LAN at Layer 2. Physical LAN devices become directly reachable from ZeroTier without needing ZeroTier installed.

## How It Works

ztnet-box creates a Linux bridge (`br0` by default) that joins the ZeroTier virtual interface and a physical NIC. Traffic is forwarded at Layer 2 (MAC level), so ARP, DHCP, and broadcast work transparently.

## Requirements

- Linux with `systemd-networkd` and `iproute2`
- No conflicting network managers (`dhcpcd`, `ifupdown`) on the bridge interface
- Root / sudo
- A separate physical NIC (or VLAN) to bridge onto

## Setup

1. Resolve dependency conflicts shown in the UI
2. Select ZeroTier interface and physical interface
3. Set bridge interface name (default: `br0`)
4. Optionally set static IP/gateway for the bridge
5. Click **Enable Bridge**
6. In your controller: enable **Allow Bridging** for this member

## Controller Setting

The member must have **Active Bridge** enabled in the controller. Without this, the ZeroTier network will not forward bridged traffic.
