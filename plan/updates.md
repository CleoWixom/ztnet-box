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
| IPv6 ip6tables for Exit Node | 🟡 Medium | ✅ Реализовано | v0.6.5 |
| Physical Network Routing | 🟡 Medium | ✅ Реализовано | v0.6.4 |
| Log Panel (frontend + backend) | 🟡 Medium | ✅ Реализовано | v0.7.0 |
| local.conf UI (Settings > ZeroTier Node) | 🟡 Medium | ✅ Реализовано | v0.7.3 |
| Layer 2 Bridge | 🟢 Low | ✅ Реализовано | v0.7.1 |
| TCP Relay + SSH deploy | 🟢 Low | ✅ Реализовано | v0.7.2 |
| NDP Proxy (ndppd) | 🟢 Low | ⏳ Следующая | — |
| Package workflows (deb/rpm/pkg/msi) | 🟢 Low | ⏳ Следующая | — |
| Screenshots workflow | 🟢 Low | ⏳ Следующая | — |

---

## 1. ✅ Exit Node — доработки (РЕАЛИЗОВАНО v0.6.3 + v0.6.5)

- `rp_filter=2` — `ExitNodeRules::check_rp_filter()`, `fix_rp_filter()`, запись в `/proc/sys/net/ipv4/conf/all/rp_filter` + append в `/etc/sysctl.conf`
- `persist_rules()` — iptables: netfilter-persistent / iptables-save → `/etc/iptables/rules.v4`; nftables: `/etc/nftables.conf` + systemctl enable
- `rp_filter_ok` и `persist_available` в `DepsStatus`
- FORWARD chain добавлен в nftables ruleset
- `allowDefault=true + allowManaged=false` → warning в ответе API
- IPv6: `enable_ipv6: bool` + `ipv6_prefix: Option<String>` в `ExitNodeRules`, ip6tables stateful rules, ip6_forward sysctl persist (см. §4)
- FreeBSD и прочие ОС → `UnsupportedPlatform` через `#[cfg(not(any(linux, macos, windows)))]`

### Остаётся
- [ ] NDP Proxy (ndppd) detection/install/configure (см. §11)

---

## 2. ✅ ZeroTier local.conf R/W API (РЕАЛИЗОВАНО v0.6.3 + v0.7.3)

- `src/zerotier/local_config/mod.rs` — `LocalConf`, `LocalSettings`, `NetworkLocalConf`
- `read(path)` / `write(path, conf)` / `read_network(id)` / `write_network(id, conf)`
- `local_conf_path()` — определяет путь по платформе (Linux/macOS/Windows)
- `validate_settings()` — возвращает `Vec<ValidationWarning>`
- `GET/PUT /api/local/config`
- `GET/PUT /api/local/networks/:id/localconf`
- **UI**: `www/src/js/pages/settings-ztnode.js` — Settings > ZeroTier Node:
  - ports (primary, secondary), portMapping toggle
  - forceTcpRelay, allowTcpFallbackRelay, tcpFallbackRelay endpoint
  - bind addresses (textarea)
  - interfacePrefixBlacklist (textarea)
  - allowManagementFrom (textarea)
  - предупреждения из `validate_settings` отображаются после сохранения

---

## 3. ✅ Input Validation (РЕАЛИЗОВАНО v0.6.x)

- `src/server/validate.rs` — `network_id()`, `node_id()`, `world_id()`, `ip_addr()`, `cidr()`
- Применено во всех handlers
- 12 unit-тестов

---

## 4. ✅ IPv6 для Exit Node (РЕАЛИЗОВАНО v0.6.5)

- `enable_ipv6: bool` + `ipv6_prefix: Option<String>` в `ExitNodeRules` + `.with_ipv6()` builder
- `enable_ipv6_forward()` — `/proc/sys/net/ipv6/conf/all/forwarding` + sysctl.conf persist
- `apply_ipv6_forwarding()` — ip6tables stateful: FORWARD + nat POSTROUTING MASQUERADE
- `remove_ipv6_rules()` — откат ip6tables правил
- `ip6tables: Option<PathBuf>` + `ipv6_forward_enabled: bool` в `DepsStatus`
- `enable_ipv6` + `ipv6_prefix` в `EnableRequest` и `ExitNodeState`
- Frontend: ip6tables в deps checklist, checkbox Enable IPv6, поле IPv6 Prefix

---

## 5. ✅ Physical Network Routing (РЕАЛИЗОВАНО v0.6.4)

- `src/physnet/` — `PhysNetConfig`, `PhysNetState`, `conflicts`, `deps`, `rules`
- iptables NAT MASQUERADE + FORWARD правила per ZT docs
- Conflict check: exitnode WARNING, bridge ERROR, subnet overlap WARNING
- `GET/POST /api/physnet/{platform,deps,status,enable,disable}`
- Frontend: `www/src/js/pages/physnet.js` — deps checklist, interface selects, managed route hint, ZT Central instructions

---

## 6. ✅ Log Panel (РЕАЛИЗОВАНО v0.7.0)

- `src/server/log_collector.rs` — `LogCollector` ring buffer (500) + broadcast (256), `CollectorLayer` (tracing::Layer)
- `GET /api/logs`, `GET /api/logs/stream` (SSE), `GET/PUT /api/logs/level`, `DELETE /api/logs`
- `CollectorLayer` встроен в `tracing_subscriber::registry()` в `main.rs`
- Frontend: `www/src/js/log-panel.js` — нижний sidebar, SSE stream, фильтр, уровни, цветовая подсветка

---

## 7. ✅ Layer 2 Bridge (РЕАЛИЗОВАНО v0.7.1)

- `src/bridge/` — `BridgeConfig`, `BridgeState`, `deps`, `platform`, `rules`
- `rules::apply()`: `ip link` bridge + enslave zt+phy + systemd-networkd `.netdev`/`.network`
- `rules::remove()`: detach + `ip link del` + удаление unit-файлов
- Conflict check: physnet → ERROR
- `GET/POST /api/bridge/{platform,deps,deps/install,status,enable,disable}`
- Frontend: `www/src/js/pages/bridge.js` — deps checklist, config form, ZT Central instructions

---

## 8. ✅ TCP Relay (РЕАЛИЗОВАНО v0.7.2)

- `src/relay/` — `SshClient` (системный ssh/sshpass), `deploy()`, `remove()`, `verify()`
- `GET /api/relay/status`, `PUT /api/relay/local`, `POST /api/relay/deploy`, `GET /api/relay/verify`, `DELETE /api/relay/remote`
- Auto-update `local.conf` tcp_fallback_relay после deploy/remove
- Frontend: `www/src/js/pages/relay.js` — local config form, SSH deploy form, remote status card

---

## 9. ⏳ NDP Proxy (ndppd)

Нужен для native IPv6 Exit Node без NAT.

- Detect `ndppd` / `ndp-proxy` binary
- Install via apt/dnf/pacman
- Generate `/etc/ndppd.conf` для zt+ интерфейса
- `systemctl enable --now ndppd`
- REST: `GET /api/exitnode/ndp/status`, `POST /api/exitnode/ndp/install`, `POST /api/exitnode/ndp/enable`, `POST /api/exitnode/ndp/disable`

---

## 10. ⏳ Package Workflows

**Файл:** `.github/workflows/packages.yml`
**Триггер:** push тега `v*.*.*`

- `.deb` (amd64, arm64) — cargo-deb + postinst systemd unit
- `.rpm` (x86_64, aarch64) — fpm
- `.pkg.tar.zst` (Arch) — fpm / makepkg
- `.msi` (Windows) — WiX Toolset
- Homebrew formula `ztnet-box.rb`

Зависимости пакетов: `zerotier-one >= 1.10, iptables | nftables`

---

## 11. ⏳ Screenshots Workflow

**Файл:** `.github/workflows/screenshots.yml`
**Триггер:** `workflow_dispatch`

Инструмент: Playwright (chromium)
Viewports: desktop (1440×900) + mobile (390×844 / iPhone 14)
Страницы: dashboard, networks, exitnode, bridge, relay, settings/ztnode
Результат: PR с обновлёнными `docs/screenshots/*.png`

---

## Ветки реализации

```
main (v0.7.3)
 ├── feat/exitnode-ipv6          ✅ IPv6 ip6tables + ip6_forward
 ├── feat/physnet-routing        ✅ Physical Network Routing
 ├── feat/log-panel              ✅ Log Panel sidebar
 ├── feat/l2-bridge              ✅ Layer 2 Bridge
 ├── feat/tcp-relay              ✅ TCP Relay + SSH deploy
 ├── feat/localconf-ui           ✅ Settings > ZeroTier Node UI
 ├── feat/package-workflows      ⏳ .deb/.rpm/.pkg/.msi
 └── feat/screenshot-workflow    ⏳ WebUI screenshots
```
