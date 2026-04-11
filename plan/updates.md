# ZeroBox — Update Plan

> Основан на изучении документации ZeroTier:
> - [Client Configuration](https://docs.zerotier.com/config/)
> - [VPN Exit Node](https://docs.zerotier.com/exitnode/)
> - [Physical Network Routing](https://docs.zerotier.com/route-between-phys-and-virt/)
> - [Layer 2 Bridge](https://docs.zerotier.com/bridging/)
> - [TCP Relay](https://docs.zerotier.com/relay/)

---

## Статус реализации

| Задача | Приоритет | Статус | Версия |
|---|---|---|---|
| rp_filter fix + persist in sysctl.conf | 🔴 High | ✅ Реализовано | v0.6.3 |
| iptables-persistent / persist rules | 🔴 High | ✅ Реализовано | v0.6.3 |
| allowDefault/allowGlobal conflict check | 🔴 High | ✅ Реализовано | v0.6.3 |
| ZeroTier local.conf R/W API | 🟡 Medium | ✅ Реализовано | v0.6.3 |
| Input validation (network_id, node_id, CIDR) | 🟡 Medium | ✅ Реализовано | v0.6.3 |
| Security headers (CSP img-src data:, Referrer-Policy) | 🟡 Medium | ✅ Реализовано | v0.6.x |
| Body limit 64KB | 🟡 Medium | ✅ Реализовано | v0.6.x |
| Public bind warning | 🟡 Medium | ✅ Реализовано | v0.6.x |
| IPv6 ip6tables for Exit Node | 🟡 Medium | ⏳ В работе | — |
| Physical Network Routing | 🟡 Medium | ✅ Реализовано | v0.6.4 |
| IPv6 ip6tables for Exit Node | 🟡 Medium | ✅ Реализовано | v0.6.5 |
| Log Panel (frontend + backend) | 🟡 Medium | ⏳ Следующая | — |
| Layer 2 Bridge | 🟢 Low | ⏳ Следующая | — |
| TCP Relay + SSH deploy | 🟢 Low | ⏳ Следующая | — |
| NDP Proxy (ndppd) | 🟢 Low | ⏳ Следующая | — |
| Package workflows (deb/rpm/pkg/msi) | 🟢 Low | ⏳ Следующая | — |
| Screenshots workflow | 🟢 Low | ⏳ Следующая | — |

---

## 1. ✅ Exit Node — доработки (РЕАЛИЗОВАНО v0.6.3)

### Реализовано
- `rp_filter=2` — `ExitNodeRules::check_rp_filter()`, `fix_rp_filter()`, запись в `/proc/sys/net/ipv4/conf/all/rp_filter` + append в `/etc/sysctl.conf`
- `persist_rules()` — iptables: netfilter-persistent / iptables-save → `/etc/iptables/rules.v4`; nftables: `/etc/nftables.conf` + systemctl enable
- `rp_filter_ok` и `persist_available` в `DepsStatus`
- FORWARD chain добавлен в nftables ruleset (был пропущен)
- `allowDefault=true + allowManaged=false` → warning в ответе API

### Ещё нужно (IPv6)
- [ ] IPv6 ip6tables stateful firewall rules для Exit Node gateway
- [ ] `enable_ipv6: bool` + `ipv6_prefix: Option<String>` в `ExitNodeRules`
- [ ] NDP Proxy (ndppd) detection/install/configure
- [ ] allowGlobal + allowDefault обязательны для IPv6 — предупреждение в UI
- [ ] FreeBSD → `UnsupportedPlatform` в platform.rs

---

## 2. ✅ ZeroTier local.conf R/W API (РЕАЛИЗОВАНО v0.6.3)

### Реализовано
- `src/zerotier/local_config/mod.rs` — `LocalConf`, `LocalSettings`, `NetworkLocalConf`
- `read()` / `write()` / `read_network()` / `write_network()`
- `local_conf_path()` — определяет путь по платформе (Linux/macOS/Windows)
- `validate_settings()` — возвращает `Vec<ValidationWarning>`:
  - `forceTcpRelay + portMappingEnabled` → предупреждение
  - `primaryPort == secondaryPort` → предупреждение
  - `interfacePrefixBlacklist` содержит `zt` → предупреждение
  - `allowManagementFrom` с публичным IP → предупреждение
- `GET/PUT /api/local/config`
- `GET/PUT /api/local/networks/:id/localconf`

### UI (ещё не реализовано)
- [ ] Страница Settings > ZeroTier Node в frontend
- [ ] Форма: ports, portMapping, forceTcpRelay, bind, interfaceBlacklist, allowManagementFrom
- [ ] Отображение предупреждений из validate_settings

---

## 3. ✅ Input Validation (РЕАЛИЗОВАНО v0.6.x)

- `src/server/validate.rs` — `network_id()`, `node_id()`, `world_id()`, `ip_addr()`, `cidr()`
- Применено в `local_config` handler
- 12 unit-тестов

---

## 4. ✅ IPv6 для Exit Node (РЕАЛИЗОВАНО v0.6.5)

### Реализовано
- `enable_ipv6: bool` + `ipv6_prefix: Option<String>` в `ExitNodeRules`
- Builder `.with_ipv6(enable, prefix)` — backward-compatible
- `enable_ipv6_forward()` — пишет `1` в `/proc/sys/net/ipv6/conf/all/forwarding`, sysctl.conf persist
- `apply_ipv6_forwarding()` — ip6tables stateful rules:
  - `FORWARD -i zt+ [-s $prefix] -j ACCEPT`
  - `FORWARD -m state --state ESTABLISHED,RELATED -j ACCEPT`
  - `nat POSTROUTING -o $WAN -j MASQUERADE`
- `remove_ipv6_rules()` — откат всех ip6tables правил (errors ignored)
- `ip6tables: Option<PathBuf>` + `ipv6_forward_enabled: bool` в `DepsStatus`
- `enable_ipv6` + `ipv6_prefix` в `EnableRequest` (handler с валидацией CIDR)
- `enable_ipv6` + `ipv6_prefix` в `ExitNodeState`
- Предупреждения: missing `network_id` с IPv6, IPv6 NAT notice
- Frontend: ip6tables в deps checklist, checkbox Enable IPv6, поле IPv6 Prefix, статус в Status card
- 3 новых integration теста: deps IPv6 fields, invalid prefix 422, status IPv6 fields
- 3 новых unit теста в rules.rs: `with_ipv6_builder`, `with_ipv6_no_prefix`, `ipv6_forward_disabled_by_default`

---

## 5. Physical Network Routing ⏳

**Ветка:** `feat/physnet-routing`  
**Источник:** https://docs.zerotier.com/route-between-phys-and-virt/

### Backend: `src/physnet/`

```rust
pub struct PhysNetConfig {
    pub zt_iface:   String,     // zt...
    pub phy_iface:  String,     // eth0
    pub phy_subnet: String,     // 192.168.100.0/24
    pub zt_addr:    String,     // ZT IP этой ноды
    pub network_id: String,
}

pub struct PhysNetState {
    pub enabled:   bool,
    pub config:    Option<PhysNetConfig>,
    pub applied_at: Option<DateTime<Utc>>,
}
```

Правила (из документации):
```
iptables -t nat -A POSTROUTING -o $PHY_IFACE -j MASQUERADE
iptables -A FORWARD -i $PHY_IFACE -o $ZT_IFACE -m state --state RELATED,ESTABLISHED -j ACCEPT
iptables -A FORWARD -i $ZT_IFACE -o $PHY_IFACE -j ACCEPT
```

Проверки конфликтов:
- Exit Node активен → WARN
- L2 Bridge активен → ERROR
- `phy_subnet` пересекается с ZT подсетью → WARN

REST API:
```
GET  /api/physnet/platform
GET  /api/physnet/deps
GET  /api/physnet/status
POST /api/physnet/enable   body: PhysNetConfig
POST /api/physnet/disable
```

---

## 6. Log Panel ⏳

**Ветка:** `feat/log-panel`

### Backend: `src/server/log_collector.rs`

```rust
pub struct LogEntry {
    pub timestamp: DateTime<Utc>,
    pub level:     LogLevel,
    pub message:   String,
    pub target:    String,
}

pub enum LogLevel { Debug, Info, Warn, Error }

pub struct LogCollector {
    buffer:    Arc<RwLock<VecDeque<LogEntry>>>,
    tx:        broadcast::Sender<LogEntry>,
    min_level: Arc<RwLock<LogLevel>>,
}
```

Реализовать как кастомный `tracing::Layer`.

REST API:
```
GET  /api/logs              ?level=warn&limit=100
GET  /api/logs/stream       SSE — live поток
PUT  /api/logs/level        body: { level: "debug"|"info"|"warn"|"error"|"all" }
DELETE /api/logs            очистить буфер
```

### Frontend: нижний sidebar

```javascript
const LogPanel = (() => {
    // Bottom bar: [▲ Logs · N entries · LEVEL] [buttons]
    // Auto-collapse by default
    // SSE streaming with EventSource
    // Level selector, Clear, Stop/Start
})();
```

---

## 7. Layer 2 Bridge ⏳

**Ветка:** `feat/l2-bridge`  
**Источник:** https://docs.zerotier.com/bridging/

### Backend: `src/bridge/`

```rust
pub struct BridgeConfig {
    pub zt_iface:     String,
    pub phy_iface:    String,
    pub bridge_iface: String,       // br0
    pub bridge_addr:  Option<String>,
    pub gateway:      Option<String>,
    pub network_id:   String,
}

pub struct BridgeDeps {
    pub systemd_networkd: bool,
    pub systemd_resolved: bool,
    pub is_root:          bool,
    pub dhcpcd_conflict:  bool,
    pub ifupdown_conflict: bool,
    pub missing:          Vec<String>,
}
```

Применение: запись systemd-networkd файлов + restart  
Удаление конфликтов: apt remove dhcpcd5 ifupdown isc-dhcp-client

REST API:
```
GET  /api/bridge/platform
GET  /api/bridge/deps
POST /api/bridge/deps/install
GET  /api/bridge/status
POST /api/bridge/enable
POST /api/bridge/disable
```

---

## 8. TCP Relay ⏳

**Ветка:** `feat/tcp-relay`  
**Источник:** https://docs.zerotier.com/relay/

### Функциональность

Local config через `local.conf`:
- `tcpFallbackRelay: "ip/port"` 
- `forceTcpRelay: bool`

SSH remote deploy (pylon docker container):
- Авторизация: root/password | key
- Установка Docker если нет
- Остановка UFW (конфликтует с Docker)
- `docker run zerotier/pylon:latest reflect`

REST API:
```
GET  /api/relay/status
PUT  /api/relay/local         обновить local.conf
POST /api/relay/deploy        body: RelayDeployConfig (SSH)
GET  /api/relay/verify        проверить доступность
DELETE /api/relay/remote      остановить remote relay
```

---

## 9. Package Workflows ⏳

**Файл:** `.github/workflows/packages.yml`  
**Триггер:** push тега `v*.*.*`

Форматы:
- `.deb` (amd64, arm64) — cargo-deb + postinst systemd unit
- `.rpm` (x86_64, aarch64) — fpm
- `.pkg.tar.zst` (Arch) — fpm / makepkg
- `.msi` (Windows) — WiX Toolset
- Homebrew formula `ztnet-box.rb`

Зависимости пакетов: `zerotier-one >= 1.10, iptables | nftables`

---

## 10. Screenshots Workflow ⏳

**Файл:** `.github/workflows/screenshots.yml`  
**Триггер:** `workflow_dispatch`

Инструмент: Playwright (chromium)  
Viewports: desktop (1440×900) + mobile (390×844 / iPhone 14)  
Страницы: dashboard, networks, controllers, exitnode, settings/tokens  
Результат: PR с обновлёнными `docs/screenshots/*.png`

---

## Ветки реализации

```
main (v0.6.5)
 ├── feat/exitnode-ipv6          ✅ IPv6 ip6tables + ip6_forward
 ├── feat/log-panel              ⏳ Log Panel sidebar
 ├── feat/l2-bridge              ⏳ Layer 2 Bridge
 ├── feat/tcp-relay              ⏳ TCP Relay + SSH deploy
 ├── feat/package-workflows      ⏳ .deb/.rpm/.pkg/.msi
 └── feat/screenshot-workflow    ⏳ WebUI screenshots
```
