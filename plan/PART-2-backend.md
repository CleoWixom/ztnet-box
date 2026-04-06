# PART 2 — Backend Modules

> Ветки: `feat/part2-zt-local-api` → `feat/part2-zt-central-api` → `feat/part2-token-store` → `feat/part2-metrics` → `feat/part2-exitnode`

---

## feat/part2-zt-local-api ✅ merged #5

**Цель:** типизированный HTTP клиент к ZeroTier One Service API (`http://127.0.0.1:9993`).

### Задачи

**`src/zerotier/local/types.rs`** — все DTO:
- [x] `NodeStatus { address, public_identity, world_id, cluster_node_id, clock, online, tcp_fallback_active, relay_policy, version }`
- [x] `NetworkMembership { id, name, status, type_, mac, mtu, dhcp, bridge, broadcast_enabled, port_error, netconf_revision, assigned_addresses, routes, dns, allow_managed, allow_global, allow_default, allow_dns }`
- [x] `NetworkMembershipUpdate` (все поля Optional)
- [x] `PeerInfo { address, version, role, latency, last_unicast_frame, last_multicast_frame, paths }`
- [x] `PeerPath { address, last_send, last_receive, active, expired, preferred, trusted_path_id }`
- [x] `ControllerNetwork { id, name, private, creation_time, routes, ip_assignment_pools, v4_assign_mode, v6_assign_mode, mtu, multicast_limit, enable_broadcast, dns }`
- [x] `ControllerMember { node_id, authorized, active_bridge, ip_assignments, name, description, no_auto_assign_ips, sso_exempt, revision, last_modified_time, capabilities, tags }`
- [x] `ControllerMemberUpdate` (все поля Optional)
- [x] `Route { target, via }` / `Dns { domain, servers }` / `IpRange` / `V4AssignMode` / `V6AssignMode`
- [x] `Moon`, `MoonRoot`, `OrbitRequest`
- [ ] `MemberDetails { address, version, physical_address, last_seen, client_version }` — не реализовано

**`src/zerotier/local/client.rs`**:
- [x] `ZtLocalClient::new(api_url, token) -> Self`
- [x] `ZtLocalClient::from_config(cfg: &LocalConfig) -> Result<Self>`
- [x] `request<T>(&self, method, path, body?) -> Result<T, ApiError>`
- [x] `node_status()`, `networks()`, `network()`, `join_network()`, `leave_network()`
- [x] `peers()`, `peer()`
- [x] `controller_networks()`, `controller_network()`, `create_controller_network()`, `update_controller_network()`, `delete_controller_network()`
- [x] `network_members()`, `network_member()`, `update_member()`, `delete_member()`
- [x] `moons()`, `orbit_moon()`, `deorbit_moon()`
- [ ] `controller_status()` — не реализован
- [ ] `generate_network_id()` — не реализован (заменён inline генерацией в create)

### REST API
- [x] `GET  /api/local/status`
- [x] `GET  /api/local/networks`
- [x] `GET  /api/local/networks/:id`
- [x] `POST /api/local/networks/:id`
- [x] `DELETE /api/local/networks/:id`
- [x] `GET  /api/local/peers`
- [x] `GET  /api/local/peers/:node_id`
- [x] `GET  /api/local/controller/networks`
- [x] `POST /api/local/controller/networks`
- [x] `GET  /api/local/controller/networks/:id`
- [x] `PUT  /api/local/controller/networks/:id`
- [x] `DELETE /api/local/controller/networks/:id`
- [x] `GET  /api/local/controller/networks/:id/members`
- [x] `GET  /api/local/controller/networks/:id/members/:node_id`
- [x] `PUT  /api/local/controller/networks/:id/members/:node_id`
- [x] `DELETE /api/local/controller/networks/:id/members/:node_id`
- [x] `GET  /api/local/moons`
- [x] `POST /api/local/moons/:world_id`
- [x] `DELETE /api/local/moons/:world_id`

### Критерии готовности
- [x] Все основные методы покрыты handler'ами
- [x] Ошибки ZT → `ApiError::ZtLocal` с сохранением кода
- [x] `leave_network` возвращает 204

---

## feat/part2-zt-central-api ⏳ следующая

**Цель:** типизированный клиент ZeroTier Central API с поддержкой нескольких токенов.

### Задачи

**`src/zerotier/central/types.rs`** — DTO Central API:
- [ ] `CentralNetwork { id, config: CentralNetworkConfig, description, rules_source, permissions, owner_id, created_at, updated_at }`
- [ ] `CentralNetworkConfig { name, private, routes, ip_assignment_pools, v4_assign_mode, v6_assign_mode, mtu, multicast_limit, enable_broadcast, dns }`
- [ ] `CentralMember { node_id, name, description, authorized, active_bridge, no_auto_assign_ips, ip_assignments, capabilities, tags, network_id, last_online, physical_address, client_version, protocol_version, supports_rules_engine, sso_exempt, identity }`
- [ ] `CentralMemberUpdate` — все поля опциональны
- [ ] `CentralUser { id, display_name, email, sms_number }`
- [ ] `AccountStatus { id, display_name, email, auth, under_limit, plan_type }`
- [ ] `ApiTokenRecord { id, token_name, created_at, last_used }`

**`src/zerotier/central/client.rs`** — переписать стаб:
- [ ] `ZtCentralClient::new(base_url, token, rate_limit) -> Self`
- [ ] Rate limit enforcement (tokio::time::Interval, Free=20/s, Paid=100/s)
- [ ] `networks()`, `create_network()`, `network()`, `update_network()`, `delete_network()`
- [ ] `network_members()`, `network_member()`, `update_member()`, `delete_member()`
- [ ] `user()`, `account_status()`
- [ ] `create_api_token()`, `delete_api_token()`, `random_token()`
- [ ] При 401 → `ApiError::ZtCentral { code: "AUTH_FAILED" }`

**`src/server/handlers/central.rs`** — новый файл:
- [ ] Все REST хэндлеры для `/api/central/*`

### REST API
- [ ] `GET  /api/central/networks`
- [ ] `POST /api/central/networks`
- [ ] `GET  /api/central/networks/:id`
- [ ] `PUT  /api/central/networks/:id`
- [ ] `DELETE /api/central/networks/:id`
- [ ] `GET  /api/central/networks/:id/members`
- [ ] `GET  /api/central/networks/:id/members/:node_id`
- [ ] `PUT  /api/central/networks/:id/members/:node_id`
- [ ] `DELETE /api/central/networks/:id/members/:node_id`
- [ ] `GET  /api/central/user`
- [ ] `GET  /api/central/status`

### Критерии готовности
- [ ] Rate limiter не позволяет превысить лимит
- [ ] При 401 от Central API → `ApiError::ZtCentral { code: "AUTH_FAILED" }`

---

## feat/part2-token-store ⏳

**Цель:** управление набором ZeroTier Central API токенов — CRUD, валидация, активный токен.

### Задачи

**`src/zerotier/central/token_store.rs`** — переписать стаб:
- [ ] `CentralClientPool` (заменяет `TokenStore`): `HashMap<id, ZtCentralClient>` + `active_id` + `base_url`
- [ ] `from_config(cfg: &CentralConfig) -> Self`
- [ ] `active_client() -> Result<&ZtCentralClient>`
- [ ] `add_token()`, `remove_token()`, `set_active()`, `validate_token()`
- [ ] При добавлении: автоопределение `rate_limit` через `account_status()`
- [ ] После изменений: `Config::save()` для персистентности
- [ ] `mask_token(token: &str) -> String`

**`src/server/handlers/tokens.rs`** — новый файл:

### REST API
- [ ] `GET  /api/settings/tokens`
- [ ] `POST /api/settings/tokens`
- [ ] `PUT  /api/settings/tokens/:id`
- [ ] `DELETE /api/settings/tokens/:id`
- [ ] `POST /api/settings/tokens/:id/activate`
- [ ] `POST /api/settings/tokens/validate`

### Критерии готовности
- [ ] Токены персистируются в `config.yml`
- [ ] Реальный токен никогда не покидает backend
- [ ] Валидация работает без side-effects

---

## feat/part2-metrics ⏳

**Цель:** периодический сбор метрик из Prometheus endpoint, кэш, JSON API.

### Задачи

**`src/metrics/parser.rs`** — переписать стаб:
- [ ] `MetricSample { name, labels: HashMap<String,String>, value: f64, timestamp: Option<i64> }`
- [ ] `parse(input: &str) -> Vec<MetricSample>` — без panic, errors skip + log
- [ ] Парсинг labels: `metric_name{label="val",...} value [timestamp]`

**`src/metrics/cache.rs`** — переписать стаб:
- [ ] `MetricsSnapshot { packets: PacketMetrics, latency: LatencyMetrics, peers: Vec<PeerMetric>, networks: Vec<NetworkMetric>, errors: ErrorMetrics }`
- [ ] `PacketMetrics`, `LatencyMetrics`, `PeerMetric`, `NetworkMetric`, `ErrorMetrics`
- [ ] `MetricsCache { parsed: RwLock<Option<MetricsSnapshot>>, raw: RwLock<Option<String>>, last_updated: RwLock<Option<DateTime<Utc>>> }`

**`src/metrics/collector.rs`** — доработать:
- [ ] Сохранять raw text + parsed snapshot
- [ ] При ошибке — логировать и продолжать

**`src/server/handlers/metrics.rs`** — новый файл:

### REST API
- [ ] `GET /api/metrics`
- [ ] `GET /api/metrics/raw`
- [ ] `GET /api/metrics/status`

### Критерии готовности
- [ ] Парсер без внешних crate
- [ ] Кэш thread-safe без deadlock
- [ ] При недоступном ZT — возвращает последний снимок

---

## feat/part2-exitnode ⏳

**Цель:** backend для настройки Exit Node, только Linux.

### Задачи

**`src/exitnode/platform.rs`** — доработать:
- [ ] `PlatformSupport { supported: bool, os: String, reason: Option<String> }`
- [ ] `check() -> PlatformSupport`

**`src/exitnode/deps.rs`** — доработать:
- [ ] `DepsStatus { iptables, nftables, is_root, missing }`
- [ ] `check_deps() -> DepsStatus` — `which` + `nix::unistd::getuid() == 0`
- [ ] `install_missing(preferred) -> Result<DepsStatus>`

**`src/exitnode/interfaces.rs`** — доработать:
- [ ] `NetworkInterface { name, addresses, is_zerotier }`
- [ ] `list_interfaces() -> Result<Vec<NetworkInterface>>` — через nix `getifaddrs`

**`src/exitnode/rules.rs`** — доработать:
- [ ] `ExitNodeRules::apply()`: ip_forward + nftables/iptables через Command (без shell)
- [ ] `ExitNodeRules::remove()`: полный откат правил

**`src/exitnode/mod.rs`** — доработать:
- [ ] `ExitNodeManager { state: RwLock<ExitNodeState>, config }` с `enable()` / `disable()`

**`src/server/handlers/exitnode.rs`** — новый файл:

### REST API
- [ ] `GET  /api/exitnode/platform`
- [ ] `GET  /api/exitnode/deps`
- [ ] `POST /api/exitnode/deps/install`
- [ ] `GET  /api/exitnode/interfaces`
- [ ] `GET  /api/exitnode/status`
- [ ] `POST /api/exitnode/enable`
- [ ] `POST /api/exitnode/disable`

### Критерии готовности
- [ ] Нет curl/wget/shell-скриптов
- [ ] Root-проверка → 403 если не root
- [ ] `remove()` полностью откатывает правила
