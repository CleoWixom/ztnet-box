# TCP Relay (Pylon)

A TCP relay for ZeroTier nodes behind firewalls that block UDP traffic.

## How It Works

ZeroTier normally uses UDP for peer-to-peer connections. When UDP is blocked, nodes show `RELAY` status and traffic is routed through ZeroTier's public roots. A **Pylon** relay server provides a private, self-hosted alternative relay accessible over TCP (port 443 by default).

Traffic remains end-to-end encrypted — the relay cannot read it.

## When to Use

- Nodes consistently show `RELAY` in the peer list
- Behind corporate firewalls that block UDP
- On networks where only TCP port 443 is allowed

## Setup

### Option A: Deploy Pylon via UI

1. Open **Gateway → TCP Relay**
2. Fill in the **Deploy Pylon** section with your VPS SSH details
3. Click **Deploy** — ztnet-box installs and starts Pylon over SSH

### Option B: Manual Pylon Setup

```bash
# On your VPS
curl -fsSL https://github.com/nickolastone/pylon/releases/latest/download/pylon-linux-x86_64 \
  -o pylon && chmod +x pylon
./pylon --port 443
```

Then in ztnet-box → TCP Relay → Local Configuration:
- Set **TCP Fallback Relay** to `<VPS_IP>/443`
- Save

### Force TCP (debugging only)

**Force TCP Relay** routes all ZeroTier traffic through TCP. Only enable temporarily for testing — it significantly degrades performance and latency.
