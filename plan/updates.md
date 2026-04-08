# ZeroBox — Update Plan

> Основан на изучении документации ZeroTier:
> - [Client Configuration](https://docs.zerotier.com/config/)
> - [VPN Exit Node](https://docs.zerotier.com/exitnode/)
> - [Physical Network Routing](https://docs.zerotier.com/route-between-phys-and-virt/)
> - [Layer 2 Bridge](https://docs.zerotier.com/bridging/)
> - [TCP Relay](https://docs.zerotier.com/relay/)
>
> Аудит кода выполнен: `src/exitnode/`, `src/zerotier/local/`, `src/server/handlers/`.

---

## Аудит соответствия: текущий код vs документация

### Что реализовано ✅
- IPv4 ip_forward через `fs::write("/proc/sys/net/ipv4/ip_forward")`
- MASQUERADE через iptables и nftables
- FORWARD rules (ZT→WAN, ESTABLISHED,RELATED)
- Определение backend: nftables/iptables
- Базовая отдача интерфейсов через getifaddrs

### Чего не хватает ❌
| Фича | Документация | Статус |
|---|---|---|
| `rp_filter=2` для Linux клиентов | [exitnode#rp_filter](https://docs.zerotier.com/exitnode/#a-linux-gotcha-rp_filter) | ❌ отсутствует |
| IPv6 ip6tables rules | [exitnode#ipv6](https://docs.zerotier.com/exitnode/#ipv6-optional) | ❌ отсутствует |
| allowGlobal + allowDefault для IPv6 | [exitnode#allowglobal](https://docs.zerotier.com/exitnode/#allowglobal-and-allowdefault) | ❌ UI не уведомляет |
| NDP Proxy (ndppd) для IPv6 gateway | [exitnode#ndp](https://docs.zerotier.com/exitnode/#set-up-gateway-ndp-proxying-not-always-needed) | ❌ отсутствует |
| IPv6 Security (ip6tables stateful) | [exitnode#ipv6-security](https://docs.zerotier.com/exitnode/#ipv6-security) | ❌ отсутствует |
| iptables-persistent / netfilter-persistent | [exitnode](https://docs.zerotier.com/exitnode/) | ❌ правила не персистируются |
| local.conf read/write (ZT settings) | [config](https://docs.zerotier.com/config/) | ❌ отсутствует |
| `<network>.local.conf` read/write | [config#network-specific](https://docs.zerotier.com/config/) | ❌ отсутствует |
| Physical Network Routing | [route-between-phys-and-virt](https://docs.zerotier.com/route-between-phys-and-virt/) | ❌ новый раздел |
| Layer 2 Bridge (systemd-networkd) | [bridging](https://docs.zerotier.com/bridging/) | ❌ новый раздел |
| TCP Relay (pylon) | [relay](https://docs.zerotier.com/relay/) | ❌ новый раздел |

---

## 1. MENU > ZeroTier Settings (local.conf) — новый пункт

**Источник:** [docs.zerotier.com/config/](https://docs.zerotier.com/config/)

### Backend: новые типы и API

```rust
// src/zerotier/local_config/mod.rs
pub struct LocalConf {
    pub physical: HashMap<String, PhysicalPathConfig>,  // CIDR → blacklist
    pub virtual_:  HashMap<String, VirtualNodeConfig>,  // node_id → try/blacklist
    pub settings:  LocalSettings,
}

pub struct LocalSettings {
    pub primary_port:             Option<u16>,
    pub secondary_port:           Option<u16>,
    pub tertiary_port:            Option<u16>,
    pub port_mapping_enabled:     Option<bool>,
    pub force_tcp_relay:          Option<bool>,
    pub interface_prefix_blacklist: Option<Vec<String>>,
    pub allow_management_from:    Option<Vec<String>>,
    pub allow_tcp_fallback_relay: Option<bool>,
    pub bind:                     Option<Vec<String>>,
    pub tcp_fallback_relay:       Option<String>,  // "ip/port"
}
```

Расположение файла по платформам:
- Linux: `/var/lib/zerotier-one/local.conf`
- macOS: `/Library/Application Support/ZeroTier/One/local.conf`
- Windows: `C:\ProgramData\ZeroTier\One\local.conf`

### REST API (новые эндпоинты)
```
GET  /api/local/config          → LocalConf (читает local.conf)
PUT  /api/local/config          → сохраняет local.conf, перезапускает ZT если нужно

GET  /api/local/networks/:id/localconf   → NetworkLocalConf (allowManaged/Global/Default/DNS)
PUT  /api/local/networks/:id/localconf   → пишет <network-id>.local.conf
```

### Проверки конфликтов
- `force_tcp_relay: true` + `port_mapping_enabled: true` → WARN "forceTcpRelay делает portMapping бессмысленным"
- `primary_port == secondary_port` → ERROR "порты должны различаться"
- `interface_prefix_blacklist` содержит `zt` → WARN "блокируете ZT-интерфейсы"
- `allow_management_from` с не-loopback адресом → WARN "Management API открыт публично"

### UI: новая страница Settings > ZeroTier Node
- **Primary/Secondary/Tertiary port** inputs (0 = auto)
- **Toggle:** portMappingEnabled (UPnP/NAT-PMP)
- **Toggle:** allowTcpFallbackRelay
- **Input:** bind IP addresses (array)
- **Interface Blacklist:** add/remove prefixes
- **Toggle:** forceTcpRelay (с предупреждением о производительности)
- **Physical path blacklist:** таблица CIDR → blacklist toggle

---

## 2. MENU > Exit Node — доработка (критические пробелы)

**Источник:** [docs.zerotier.com/exitnode/](https://docs.zerotier.com/exitnode/)

### 2.1 rp_filter (Linux Gotcha)

```rust
// src/exitnode/rules.rs — добавить
pub fn check_rp_filter() -> bool {
    std::fs::read_to_string("/proc/sys/net/ipv4/conf/all/rp_filter")
        .map(|s| s.trim() == "2" || s.trim() == "0")
        .unwrap_or(false)
}

pub fn fix_rp_filter(persistent: bool) -> Result<(), RulesError> {
    // Временно
    std::fs::write("/proc/sys/net/ipv4/conf/all/rp_filter", "2\n")?;
    // Постоянно: дописать в /etc/sysctl.conf
    if persistent {
        append_sysctl("net.ipv4.conf.all.rp_filter", "2")?;
    }
    Ok(())
}
```

В UI: в шаге Deps добавить строку "rp_filter = 2 (required for clients)" с ✅/❌ и кнопкой "Fix".

> **Примечание из документации:** rp_filter нужен только на **клиентских** нодах с allowDefault=1, **не** на самом gateway.

### 2.2 IPv6 Exit Node

```rust
pub struct ExitNodeRules {
    pub zt_iface:   String,
    pub wan_iface:  String,
    pub backend:    FirewallBackend,
    pub enable_ipv6: bool,          // новое поле
    pub ipv6_prefix: Option<String>, // e.g. "2001:19f0:6001:01a6::/64"
}

// В apply() при enable_ipv6 = true:
fn apply_ipv6_forwarding(&self) -> Result<(), RulesError> {
    // ip6tables stateful firewall (рекомендованный вариант из доки):
    // :FORWARD DROP
    // -A FORWARD -i zt+ -s $prefix -j ACCEPT
    // -A FORWARD -m state --state ESTABLISHED,RELATED -j ACCEPT
    run_ip6tables(&["-A", "FORWARD", "-i", &self.zt_iface,
        "-s", prefix, "-j", "ACCEPT"])?;
    run_ip6tables(&["-A", "FORWARD", "-m", "state",
        "--state", "ESTABLISHED,RELATED", "-j", "ACCEPT"])?;
}
```

В UI: дополнительный шаг "IPv6 (optional)":
- Toggle: Enable IPv6 forwarding
- Input: IPv6 prefix (/64)
- Checklist: `allowGlobal` + `allowDefault` должны быть включены на клиентах

### 2.3 NDP Proxy (ndppd) — опционально

```rust
// src/exitnode/ndp.rs
pub fn check_ndppd() -> bool { which::which("ndppd").is_ok() }

pub fn install_ndppd() -> Result<(), String> { /* apt/dnf install ndppd */ }

pub fn configure_ndppd(prefix: &str, iface: &str) -> Result<(), String> {
    // Генерировать /etc/ndppd.conf:
    // rule $prefix/80 { iface $wan_iface }
}
```

В UI: показывать только если IPv6 включён. Кнопка "Install & Configure ndppd".

### 2.4 iptables-persistent (правила не теряются после reboot)

```rust
// После apply() вызывать:
pub fn persist_rules(backend: FirewallBackend) -> Result<(), RulesError> {
    match backend {
        FirewallBackend::Iptables => {
            // Проверить наличие iptables-persistent / netfilter-persistent
            if which::which("netfilter-persistent").is_ok() {
                run(&["netfilter-persistent", "save"])
            } else if which::which("iptables-save").is_ok() {
                // Записать в /etc/iptables/rules.v4
                let rules = run_capture(&["iptables-save"])?;
                std::fs::write("/etc/iptables/rules.v4", rules)?;
                Ok(())
            } else {
                Err(RulesError::Command("iptables-persistent not available".into()))
            }
        }
        FirewallBackend::Nftables => {
            run(&["nft", "list", "ruleset"]).and_then(|_| {
                run(&["systemctl", "enable", "nftables"])
            })
        }
    }
}
```

### 2.5 allowGlobal / allowDefault — проверка конфликтов

При включении Exit Node проверять через `/api/local/networks/:id/localconf`:
- `allowDefault` должен быть `1` → WARNING если нет
- Для IPv6: `allowGlobal` тоже должен быть `1`

UI показывает конфликт баннером: "⚠️ allowDefault is disabled on this network. Exit node won't route traffic."

### 2.6 FreeBSD unsupported

В `platform.rs` добавить: FreeBSD → `UnsupportedPlatform` с объяснением из доки.

---

## 3. MENU > Physical Network Routing — новый раздел

**Источник:** [docs.zerotier.com/route-between-phys-and-virt/](https://docs.zerotier.com/route-between-phys-and-virt/)

### Концепция
NAT/Masquerade между ZeroTier и физической LAN. Не требует доступа к роутеру. Linux-only.

### Backend: новый модуль `src/physnet/`

```rust
// src/physnet/mod.rs
pub struct PhysNetConfig {
    pub zt_iface:   String,    // e.g. zt7nnig26
    pub phy_iface:  String,    // e.g. eth0
    pub phy_subnet: String,    // e.g. 192.168.100.0/24 (маршрут /23 в ZT)
    pub zt_addr:    String,    // IP этой ноды в ZT сети (Gateway address)
    pub network_id: String,
}

pub struct PhysNetState {
    pub enabled:    bool,
    pub config:     Option<PhysNetConfig>,
    pub applied_at: Option<DateTime<Utc>>,
}
```

```rust
// Применение:
pub fn apply(cfg: &PhysNetConfig) -> Result<(), PhysNetError> {
    enable_ip_forward()?;
    // iptables rules (из документации):
    run_iptables(&["-t", "nat", "-A", "POSTROUTING", "-o", &cfg.phy_iface, "-j", "MASQUERADE"])?;
    run_iptables(&["-A", "FORWARD", "-i", &cfg.phy_iface, "-o", &cfg.zt_iface,
        "-m", "state", "--state", "RELATED,ESTABLISHED", "-j", "ACCEPT"])?;
    run_iptables(&["-A", "FORWARD", "-i", &cfg.zt_iface, "-o", &cfg.phy_iface, "-j", "ACCEPT"])?;
    persist_iptables()?;
    Ok(())
}
```

### Проверки конфликтов
- Exit Node уже включён → WARN "Конфликт: и Exit Node и Physical Routing включены одновременно — iptables правила могут конфликтовать"
- Layer 2 Bridge включён → ERROR "Нельзя использовать одновременно с L2 Bridge"
- `phy_subnet` пересекается с ZT подсетью → WARN "Подсети пересекаются, могут быть петли маршрутизации"

### REST API
```
GET  /api/physnet/platform      → { supported, reason }
GET  /api/physnet/deps          → { iptables, is_root, missing }
GET  /api/physnet/status        → PhysNetState
POST /api/physnet/enable        → body: PhysNetConfig
POST /api/physnet/disable
```

### UI: страница "Physical Network Routing"
1. **Step 1** — Platform check (Linux-only)
2. **Step 2** — Deps check (iptables, root)
3. **Step 3** — Configuration:
   - ZT Network dropdown (из connected networks)
   - Physical interface dropdown (из /proc/net/dev, не ZT)
   - Physical subnet input (CIDR), автоподстановка /23 для managed route
   - ZT IP адрес этой ноды (автозаполнение из ZT network)
   - **Banner:** "Add managed route in ZeroTier Central: `$PHY_SUB (via /23)` → `$ZT_ADDR`"
4. Toggle Enable/Disable

---

## 4. MENU > Layer 2 Bridge — новый раздел

**Источник:** [docs.zerotier.com/bridging/](https://docs.zerotier.com/bridging/)

### Концепция
Соединяет ZeroTier и физический Ethernet на L2 через Linux bridge (`br0`). Использует `systemd-networkd`. Devices on physical LAN получают ZT-адреса и наоборот.

### Backend: новый модуль `src/bridge/`

```rust
pub struct BridgeConfig {
    pub zt_iface:     String,       // zt...
    pub phy_iface:    String,       // eth0 / enp...
    pub bridge_iface: String,       // br0 (default)
    pub bridge_addr:  Option<String>, // статический IP или DHCP
    pub gateway:      Option<String>,
    pub dns:          Option<Vec<String>>,
    pub zt_pool_start: String,      // ZT Auto-Assign start
    pub zt_pool_end:   String,      // ZT Auto-Assign end
    pub zt_route:      String,      // Managed Route (/23)
    pub network_id:    String,
}
```

```rust
// Применение через systemd-networkd:
pub fn apply(cfg: &BridgeConfig) -> Result<(), BridgeError> {
    // 1. zerotier-cli set $NETWORK_ID allowManaged=0
    // 2. Записать /etc/systemd/network/{br0.netdev, 25-br0.network, 25-br0-zt.network, 25-br0-en.network}
    // 3. systemctl restart systemd-networkd
    write_netdev_file(&cfg.bridge_iface)?;
    write_bridge_network_file(cfg)?;
    write_zt_bridge_member(cfg)?;
    write_eth_bridge_member(cfg)?;
    run_cmd("systemctl", &["restart", "systemd-networkd"])?;
    Ok(())
}
```

### Проверки зависимостей
```rust
pub struct BridgeDeps {
    pub systemd_networkd:  bool,   // which systemctl && systemd-networkd status
    pub systemd_resolved:  bool,
    pub is_root:           bool,
    pub dhcpcd_conflict:   bool,   // dpkg -l dhcpcd5 | grep -q "^ii"
    pub ifupdown_conflict: bool,   // dpkg -l ifupdown | grep -q "^ii"
    pub missing:           Vec<String>,
}

// Установка: sudo apt install systemd-resolved
// Удаление конфликтов: apt remove dhcpcd5 fake-hwclock ifupdown isc-dhcp-client openresolv
```

### Обнаружение конфликтов
- `dhcpcd` / `ifupdown` установлены → WARN с инструкцией по удалению
- Exit Node активен → ERROR "Нельзя одновременно с Exit Node"
- Physical Network Routing активен → ERROR
- `phy_iface` == `zt_iface` → ERROR
- Raspbian < Bookworm и `systemd-resolved` не установлен → WARN (требует ручной установки)

### REST API
```
GET  /api/bridge/platform       → { supported, os }
GET  /api/bridge/deps           → BridgeDeps
POST /api/bridge/deps/install   → установить systemd-resolved, убрать конфликты
GET  /api/bridge/status         → BridgeState
POST /api/bridge/enable         → body: BridgeConfig
POST /api/bridge/disable        → удалить systemd-network файлы, перезапустить
GET  /api/bridge/interfaces     → физические интерфейсы (без ZT)
```

### UI: страница "Layer 2 Bridge"
1. **Step 1** — Platform (Linux + systemd-networkd)
2. **Step 2** — Deps (systemd-networkd ✅, systemd-resolved ✅, dhcpcd ❌ → удалить)
3. **Step 3** — Configuration:
   - ZT Network dropdown
   - Physical interface dropdown
   - Bridge interface name (default: br0)
   - Bridge IP (static input или DHCP toggle)
   - Gateway IP
   - ZT Auto-Assign range (start/end)
   - Managed Route (CIDR, /23 рекомендуется)
4. **Banner:** "⚠️ This will modify systemd-networkd configuration and restart networking. You may lose SSH access temporarily."
5. **Checklist в ZeroTier Central** (инструкция):
   - Enable "Allow Bridging" на member
   - Enable "Do Not Auto Assign" на member

---

## 5. MENU > TCP Relay — новый раздел

**Источник:** [docs.zerotier.com/relay/](https://docs.zerotier.com/relay/)

### Концепция
Когда UDP/NAT-hole-punching заблокирован корпоративным файрволом — использовать pylon relay через HTTPS/443. Поддерживает локальный self-hosted relay.

### 5.1 Конфигурация local ZT node для использования relay

```rust
// Добавить в LocalSettings:
pub tcp_fallback_relay: Option<String>,  // "ip/port"
pub force_tcp_relay:    Option<bool>,    // для тестирования
```

UI в **Settings > ZeroTier Node**:
- Input: TCP Relay address (`ip/port`, e.g. `1.2.3.4/443`)
- Toggle: Force TCP Relay (для тестирования)
- Status: текущий режим (DIRECT / RELAY / TUNNELED) из `/api/local/status`

### 5.2 Remote TCP Relay Setup (SSH)

Новый модуль для remote deployment через SSH:

```rust
// src/relay/ssh_deploy.rs
pub struct RelayDeployConfig {
    pub host:        String,         // IP или hostname сервера
    pub port:        u16,            // SSH port (default 22)
    pub auth:        SshAuth,        // root+password или key
    pub tcp_port:    u16,            // default 443
    pub udp_port:    u16,            // default 9993
}

pub enum SshAuth {
    Password { user: String, password: String },
    Key      { user: String, key_path: PathBuf },
}
```

Шаги деплоя (через SSH команды):
1. Проверить наличие Docker на удалённом хосте
2. Установить Docker если отсутствует
3. Остановить UFW если запущен (конфликт с Docker)
4. Запустить pylon container:
   ```
   docker run --init -p 443:443 -p 9993:9993/udp zerotier/pylon:latest reflect
   ```
5. Настроить автозапуск (systemd unit или docker restart policy)
6. Проверить доступность: curl tcp://$HOST:$TCP_PORT

### Проверки конфликтов и зависимостей
- UFW активен → WARN "UFW конфликтует с Docker. Будет остановлен перед запуском pylon."
- Порт 443 занят (nginx/apache) → ERROR "Порт 443 уже используется"
- Docker не установлен → кнопка "Install Docker"
- `forceTcpRelay: true` без настроенного relay → WARN "Без relay адреса трафик пойдёт через официальные серверы ZeroTier (медленно)"

### REST API
```
GET  /api/relay/status          → { local_config: LocalRelayConf, remote: RelayStatus }
PUT  /api/relay/local           → обновить tcp_fallback_relay и force_tcp_relay в local.conf
POST /api/relay/deploy          → body: RelayDeployConfig — SSH деплой pylon
GET  /api/relay/verify          → проверить доступность настроенного relay
DELETE /api/relay/remote        → остановить удалённый relay (SSH)
```

### UI: страница "TCP Relay"
- **Tabs:** Local Config | Deploy Relay
- **Local Config:**
  - Input: Relay address (`ip/port`)
  - Toggle: Force TCP Relay (с предупреждением)
  - Status badge: DIRECT / RELAY / TUNNELED
  - Кнопка "Verify Connection"
- **Deploy Relay (SSH):**
  - Input: Server hostname/IP
  - Input: SSH port
  - Radio: Auth (Password / Key)
  - Input: username, password / key path
  - Input: TCP port (443), UDP port (9993)
  - Кнопка "Deploy" → прогресс-лог
  - После деплоя: автозаполнение "Local Config" relay address

---

## 6. Sidebar Bottom > Log Panel — новый компонент

### Backend

```rust
// src/server/log_collector.rs
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tracing_subscriber::Layer;

pub struct LogEntry {
    pub timestamp: DateTime<Utc>,
    pub level:     LogLevel,
    pub message:   String,
    pub target:    String,   // module path
}

pub enum LogLevel { Debug, Info, Warn, Error }

pub struct LogCollector {
    buffer:   Arc<RwLock<VecDeque<LogEntry>>>,  // последние 500 записей
    tx:       broadcast::Sender<LogEntry>,        // для SSE стриминга
    min_level: Arc<RwLock<LogLevel>>,
}
```

Интегрировать как кастомный `tracing::Layer` — перехватывать все события и добавлять в буфер.

### REST API
```
GET  /api/logs                  → последние N записей (query: ?level=warn&limit=100)
GET  /api/logs/stream           → SSE (Server-Sent Events) — live поток
PUT  /api/logs/level            → body: { level: "debug"|"info"|"warn"|"error"|"all" }
DELETE /api/logs                → очистить буфер
```

### Frontend: Log Panel (нижний sidebar)

```javascript
// Компонент: нижний сворачиваемый панель
const LogPanel = (() => {
    let _visible = false;
    let _level = 'info';
    let _es = null;  // EventSource для SSE

    // HTML структура добавляется в body под #app
    // [▲ Logs · 12 entries · WARN] [INFO] [WARN] [ERROR] [DEBUG] [ALL] [Clear] [×]
    // <div id="log-list">...</div>
})();
```

UI элементы:
- **Bottom bar** (всегда видна): `[▲ Logs] · N entries · текущий уровень`
- Клик на bar → expand/collapse с анимацией (default: скрыт)
- **Toolbar в раскрытой панели:** Level selector (INFO | WARN | ERROR | DEBUG | ALL) + Clear + Stop/Start
- **Log list:** timestamp | level badge | target | message
- **Stop/Start:** при Stop — EventSource отключается, буфер не обновляется
- **Auto-scroll** to bottom при новых записях (если не скроллил вверх)
- Высота панели: 30% от viewport, resize-able

---

## 7. Workflow > Package Build Pipeline

### Новые workflow: `.github/workflows/packages.yml`

**Триггер:** push тега `v[0-9]+.[0-9]+.[0-9]+` (вместе с `release.yml`)

**Матрица сборки пакетов:**

#### .deb (Debian/Ubuntu — apt)
```yaml
- os: ubuntu-latest
  target: x86_64-unknown-linux-gnu
  pkg: deb
```
Инструмент: `cargo-deb` или `fpm`
```
ztnet-box_0.4.0_amd64.deb
```
Зависимости пакета: `zerotier-one (>= 1.10), iptables | nftables`
Postinst скрипт: создаёт systemd unit, копирует config.yml.example

#### .rpm (RHEL/Fedora — dnf/yum)
```yaml
- os: ubuntu-latest
  target: x86_64-unknown-linux-gnu
  pkg: rpm
```
Инструмент: `fpm`
```
ztnet-box-0.4.0-1.x86_64.rpm
```
Зависимости: `zerotier-one >= 1.10, iptables или nftables`

#### .pkg.tar.zst (Arch Linux — pacman)
```yaml
- os: ubuntu-latest
  pkg: pacman
```
Инструмент: `fpm` или `makepkg`
```
ztnet-box-0.4.0-1-x86_64.pkg.tar.zst
```
PKGBUILD автогенерация из Cargo.toml метаданных

#### Homebrew Formula (macOS — brew)
```yaml
- os: macos-latest
  pkg: brew
```
Создаёт `Formula/ztnet-box.rb`:
```ruby
class ZtnetBox < Formula
  desc "Local web UI for ZeroTier"
  url "https://github.com/CleoWixom/ztnet-box/releases/download/v0.4.0/..."
  sha256 "..."
  depends_on "zerotier"
end
```
Пуш в отдельный tap-репозиторий `homebrew-ztnet-box`

#### .msi (Windows)
```yaml
- os: windows-latest
  pkg: msi
```
Инструмент: WiX Toolset через GitHub Action
Включает: бинарник, config.yml.example, README
Создаёт Windows Service при установке

### Проверка зависимостей в пакетах
- Pre-install скрипты проверяют наличие ZeroTier One
- Post-install: `systemctl enable --now ztnet-box` (Linux)
- Uninstall: `systemctl stop ztnet-box && systemctl disable ztnet-box`

### Артефакты из `packages.yml`
```
ztnet-box_0.4.0_amd64.deb
ztnet-box_0.4.0_arm64.deb
ztnet-box-0.4.0-1.x86_64.rpm
ztnet-box-0.4.0-1.aarch64.rpm
ztnet-box-0.4.0-1-x86_64.pkg.tar.zst
ztnet-box-0.4.0.msi
ztnet-box-0.4.0.rb (Homebrew formula)
```

---

## 8. Workflow > WebUI Screenshots (manual trigger)

### `.github/workflows/screenshots.yml`

**Триггер:** `workflow_dispatch` (только ручной запуск)

```yaml
name: WebUI Screenshots
on:
  workflow_dispatch:
    inputs:
      zerotier_token:
        description: 'ZeroTier API token (optional, for realistic data)'
        required: false

jobs:
  screenshot:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Build binary
        run: cargo build --release

      - name: Start ztnet-box
        run: |
          ./target/release/ztnet-box &
          sleep 2  # wait for startup

      - name: Install Playwright
        run: npx playwright install chromium

      - name: Take screenshots (Desktop)
        run: |
          npx playwright screenshot \
            --browser chromium \
            --full-page \
            --viewport "1440x900" \
            http://localhost:3000/#/dashboard \
            docs/screenshots/dashboard-desktop.png
          # Repeat for each page...

      - name: Take screenshots (Mobile)
        run: |
          npx playwright screenshot \
            --browser chromium \
            --full-page \
            --viewport "390x844" \  # iPhone 14
            http://localhost:3000/#/dashboard \
            docs/screenshots/dashboard-mobile.png

      - name: Upload screenshots
        uses: actions/upload-artifact@v4
        with:
          name: webui-screenshots
          path: docs/screenshots/

      - name: Create PR with screenshots
        if: github.event_name == 'workflow_dispatch'
        uses: peter-evans/create-pull-request@v6
        with:
          title: "docs: update WebUI screenshots"
          branch: "docs/update-screenshots"
          commit-message: "docs: update WebUI screenshots"
```

**Страницы для скриншотов:**
- `#/dashboard` → dashboard-desktop.png / dashboard-mobile.png
- `#/networks` → networks-desktop.png
- `#/controllers/networks` → controllers-desktop.png
- `#/exitnode` → exitnode-desktop.png
- `#/settings/tokens` → settings-tokens-desktop.png

Артефакты помещаются в `docs/screenshots/` и используются в README.md.

---

## Приоритеты реализации

| Приоритет | Задача | Сложность | Зависимости |
|---|---|---|---|
| 🔴 High | rp_filter fix в Exit Node | Low | — |
| 🔴 High | iptables-persistent/persist rules | Low | — |
| 🔴 High | allowDefault/allowGlobal conflict check | Low | local.conf API |
| 🟡 Medium | ZeroTier local.conf R/W API | Medium | — |
| 🟡 Medium | Physical Network Routing | Medium | exitnode модуль |
| 🟡 Medium | Log Panel (frontend + backend) | Medium | — |
| 🟡 Medium | IPv6 ip6tables for Exit Node | Medium | — |
| 🟢 Low | Layer 2 Bridge | High | systemd-networkd |
| 🟢 Low | TCP Relay + SSH deploy | High | SSH client crate |
| 🟢 Low | NDP Proxy (ndppd) | Medium | ndppd package |
| 🟢 Low | Package workflows (deb/rpm/pkg/msi) | Medium | CI/CD |
| 🟢 Low | Screenshots workflow | Low | Playwright |

---

## Ветки реализации (план)

```
main
 ├── feat/update-exitnode-rp-filter      # rp_filter + persist rules + IPv6
 ├── feat/local-conf-api                 # ZT local.conf + network.local.conf R/W
 ├── feat/physnet-routing                # Physical Network Routing
 ├── feat/l2-bridge                      # Layer 2 Bridge
 ├── feat/tcp-relay                      # TCP Relay + SSH deploy
 ├── feat/log-panel                      # Log Panel sidebar
 ├── feat/package-workflows              # .deb/.rpm/.pkg/.msi workflows
 └── feat/screenshot-workflow            # WebUI screenshots workflow
```
