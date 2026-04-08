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

### Чего не хватало — исправлено ✅
| Фича | Документация | Статус |
|---|---|---|
| `rp_filter=2` для Linux клиентов | [exitnode#rp_filter](https://docs.zerotier.com/exitnode/#a-linux-gotcha-rp_filter) | ✅ реализовано |
| iptables-persistent / netfilter-persistent | [exitnode](https://docs.zerotier.com/exitnode/) | ✅ реализовано |
| allowGlobal + allowDefault conflict check | [exitnode#allowglobal](https://docs.zerotier.com/exitnode/#allowglobal-and-allowdefault) | ✅ реализовано |
| local.conf read/write (ZT settings) | [config](https://docs.zerotier.com/config/) | ✅ реализовано |
| `<network>.local.conf` read/write | [config#network-specific](https://docs.zerotier.com/config/) | ✅ реализовано |
| FORWARD chain в nftables ruleset | [exitnode](https://docs.zerotier.com/exitnode/) | ✅ реализовано |
| DepsStatus: rp_filter_ok + persist_available | — | ✅ реализовано |

### Чего не хватает ❌
| Фича | Документация | Статус |
|---|---|---|
| IPv6 ip6tables rules | [exitnode#ipv6](https://docs.zerotier.com/exitnode/#ipv6-optional) | ❌ отсутствует |
| NDP Proxy (ndppd) для IPv6 gateway | [exitnode#ndp](https://docs.zerotier.com/exitnode/#set-up-gateway-ndp-proxying-not-always-needed) | ❌ отсутствует |
| IPv6 Security (ip6tables stateful) | [exitnode#ipv6-security](https://docs.zerotier.com/exitnode/#ipv6-security) | ❌ отсутствует |
| Physical Network Routing | [route-between-phys-and-virt](https://docs.zerotier.com/route-between-phys-and-virt/) | ❌ новый раздел |
| Layer 2 Bridge (systemd-networkd) | [bridging](https://docs.zerotier.com/bridging/) | ❌ новый раздел |
| TCP Relay (pylon) | [relay](https://docs.zerotier.com/relay/) | ❌ новый раздел |
| Log Panel (frontend + backend) | — | ❌ не реализовано |

---

## Приоритеты реализации

| Приоритет | Задача | Сложность | Статус |
|---|---|---|---|
| 🔴 High | rp_filter fix в Exit Node | Low | ✅ |
| 🔴 High | iptables-persistent/persist rules | Low | ✅ |
| 🔴 High | allowDefault/allowGlobal conflict check | Low | ✅ |
| 🔴 High | ZeroTier local.conf R/W API | Medium | ✅ |
| 🟡 Medium | Physical Network Routing | Medium | ❌ |
| 🟡 Medium | Log Panel (frontend + backend) | Medium | ❌ |
| 🟡 Medium | IPv6 ip6tables for Exit Node | Medium | ❌ |
| 🟢 Low | Layer 2 Bridge | High | ❌ |
| 🟢 Low | TCP Relay + SSH deploy | High | ❌ |
| 🟢 Low | NDP Proxy (ndppd) | Medium | ❌ |
| 🟢 Low | Package workflows (deb/rpm/pkg/msi) | Medium | ❌ |
| 🟢 Low | Screenshots workflow | Low | ❌ |



---

## 1. ZeroTier Settings (local.conf) ✅ реализовано

- `src/zerotier/local_config/mod.rs` — `LocalConf`, `NetworkLocalConf`, `LocalSettings`
- `GET/PUT /api/local/config` — читает/пишет `/var/lib/zerotier-one/local.conf`
- `GET/PUT /api/local/networks/:id/localconf` — читает/пишет `<id>.local.conf`
- Валидация конфликтов: `forceTcpRelay+portMapping`, совпадающие порты, `zt*` в blacklist, публичный `allowManagementFrom`
- Conflict check при `enable` exit node: предупреждение если `allowDefault` не установлен

---

## 2. Exit Node — доработка ✅ реализовано

- `rp_filter=2` в `rules.rs`: `check_rp_filter()`, `fix_rp_filter()`, запись в `/etc/sysctl.conf`
- `persist_rules()`: `netfilter-persistent save` → `iptables-save` → `/etc/iptables/rules.v4` (iptables); `nft list ruleset` → `/etc/nftables.conf` + `systemctl enable nftables` (nftables)
- FORWARD chain добавлен в nftables ruleset
- `DepsStatus`: новые поля `rp_filter_ok`, `persist_available`
- Exit Node enable: `allowDefault`/`allowGlobal` conflict check + warnings в response

---

## 3. Physical Network Routing ❌ не реализовано

*(описание из плана ниже)*

### Backend: новый модуль `src/physnet/`
[см. оригинальный план — раздел 3]

---

## 4. Layer 2 Bridge ❌ не реализовано

[см. оригинальный план — раздел 4]

---

## 5. TCP Relay ❌ не реализовано

[см. оригинальный план — раздел 5]

---

## 6. Log Panel ❌ не реализовано

[см. оригинальный план — раздел 6]

---

## 7. Package Build Pipeline ❌ не реализовано

[см. оригинальный план — раздел 7]

---

## 8. WebUI Screenshots ❌ не реализовано

[см. оригинальный план — раздел 8]

---

## Ветки реализации

```
main
 ├── feat/update-exitnode-rp-filter      ✅ merged
 ├── feat/local-conf-api                 ✅ merged
 ├── feat/physnet-routing                ❌ не начато
 ├── feat/l2-bridge                      ❌ не начато
 ├── feat/tcp-relay                      ❌ не начато
 ├── feat/log-panel                      ❌ не начато
 ├── feat/package-workflows              ❌ не начато
 └── feat/screenshot-workflow            ❌ не начато
```
