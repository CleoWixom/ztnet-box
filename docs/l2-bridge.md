# L2 Bridge

Bridge a ZeroTier interface and a physical network interface at Layer 2, making ZeroTier peers appear directly on the physical LAN.

## How it works

```
Physical LAN ──[eth0]──┐
                        ├──[br0 Linux bridge]
ZeroTier ──[zt*]───────┘
```

ZeroTier peers receive addresses from the physical LAN's DHCP server and are visible to all physical hosts as regular network devices — no routing, no NAT.

ZTNetwork Panel runs:
```bash
ip link add br0 type bridge
ip link set eth0 master br0
ip link set zt7nnig26 master br0
ip link set br0 up
```

Writes a `systemd-networkd` `.network` unit for persistence across reboots.

## Requirements

| Requirement | Notes |
|---|---|
| **OS** | Linux only |
| **Privileges** | root (`sudo`) |
| **Packages** | `iproute2` (standard on all distros) |
| **Persistence** | `systemd-networkd` (optional but recommended) |
| **ZeroTier Central** | **Bridging must be enabled** for this member |

## Setup

1. Open **L2 Bridge** in the sidebar
2. Select the **ZeroTier interface** (e.g. `zt7nnig26`)
3. Select the **physical interface** to bridge (e.g. `eth0`, `enp3s0`)
4. Click **Enable**

**Critical:** In ZeroTier Central → your network → member settings for this node, enable **Allow Ethernet Bridging**. Without this, the bridge will create a loop.

## Diagnostics

```bash
# Check bridge is up
ip link show br0
bridge link show

# Check MAC address table
bridge fdb show

# Check spanning tree
brctl showstp br0

# Check systemd-networkd unit
systemctl status systemd-networkd
cat /etc/systemd/network/ztnet-box-bridge.network
```

## Troubleshooting

| Symptom | Likely cause | Fix |
|---|---|---|
| No connectivity after bridge | Bridging not enabled in ZT Central | Enable "Allow Ethernet Bridging" for this member |
| Bridge up but no DHCP | Physical interface has address on br0 instead | Move IP from `eth0` to `br0` |
| Reboot loses bridge | systemd-networkd not running | `systemctl enable systemd-networkd` |
| `ip: Cannot find device` | Interface name wrong | `ip link` to list all interfaces |

## Notes

- The physical interface (`eth0`) should **not** have an IP address — the bridge (`br0`) holds the IP
- If you lose SSH access after enabling, log into the console and `ip link delete br0` to revert
- L2 bridging works best on a dedicated physical NIC; avoid bridging the same NIC used for management
