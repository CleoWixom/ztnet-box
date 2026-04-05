# PART 2 — Backend Modules

> Ветки: `feat/part2-zt-local-api` → `feat/part2-zt-central-api` → `feat/part2-token-store` → `feat/part2-metrics` → `feat/part2-exitnode`

---

## feat/part2-zt-local-api

**Цель:** типизированный HTTP клиент к ZeroTier One Service API (`http://127.0.0.1:9993`).

### Задачи

**`src/zerotier/local/types.rs`** — все DTO:
- [ ] `NodeStatus { address, public_identity, world_id, cluster_node_id, clock, online, tcp_fallback_active, relay_policy, version }`
- [ ] `NetworkMembership { id, name, status, type_, mac, mtu, dhcp, bridge, broadcast_enabled, port_error, netconf_revision, assigned_addresses, routes, dns, allow_managed, allow_global, allow_default, allow_dns }`
- [ ] `NetworkMembershipUpdate { allow_managed?, allow_global?, allow_default?, allow_dns?, dns? }` (все поля Optional для PATCH-семантики)
- [ ] `PeerInfo { address, version, role, latency, last_unicast_frame, last_multicast_frame, paths }` где `paths: Vec<PeerPath { address, last_send, last_receive, active, expired, preferred, trusted_path_id }>`
- [ ] `ControllerNetwork { id, name, private, creation_time, routes, ip_assignments_pool, v4_assign_mode, v6_assign_mode, mtu, multicast_limit, enable_broadcast, dns }`
- [ ] `ControllerMember { node_id, authorized, active_bridge, ip_assignments, name, description, no_auto_assign_ips, sso_exempt, revision, last_modified_time, capabilities, tags }`
- [ ] `MemberDetails { address, version, physical_address, last_seen, client_version }` (дополнительно к ControllerMember)
- [ ] `Route { target: String, via: Option<String> }`
- [ ] `Dns { domain: String, servers: Vec<String> }`
- [ ] `V4AssignMode { zt: bool }` / `V6AssignMode { zt: bool, rfc4193: bool, plan6: bool }`
- [ ] Все структуры: `#[derive(Debug, Clone, Serialize, Deserialize)]`

**`src/zerotier/local/client.rs`**:
- [ ] `ZtLocalClient { base_url: String, token: String, http: reqwest::Client }`
- [ ] `ZtLocalClient::new(api_url: &str, token: &str) -> Self`
- [ ] Чтение токена из файла: `ZtLocalClient::from_config(cfg: &LocalConfig) -> Result<Self>` — читает `token_file`
- [ ] Внутренний метод `request<T: DeserializeOwned>(&self, method, path, body?) -> Result<T, ApiError>`

Методы (все async):

| Метод | HTTP | Путь |
|---|---|---|
| `node_status()` | GET | `/status` |
| `networks()` | GET | `/network` |
| `network(id)` | GET | `/network/{id}` |
| `join_network(id, update)` | POST | `/network/{id}` |
| `leave_network(id)` | DELETE | `/network/{id}` |
| `peers()` | GET | `/peer` |
| `peer(node_id)` | GET | `/peer/{node_id}` |
| `controller_status()` | GET | `/controller` |
| `controller_networks()` | GET | `/controller/network` |
| `controller_network(net_id)` | GET | `/controller/network/{net_id}` |
| `create_or_update_network(net_id, cfg)` | POST | `/controller/network/{net_id}` |
| `generate_network_id()` | POST | `/controller/network/generate-id` |
| `delete_controller_network(net_id)` | DELETE | `/controller/network/{net_id}` |
| `network_members(net_id)` | GET | `/controller/network/{net_id}/member` |
| `network_member(net_id, node_id)` | GET | `/controller/network/{net_id}/member/{node_id}` |
| `update_member(net_id, node_id, cfg)` | POST | `/controller/network/{net_id}/member/{node_id}` |
| `delete_member(net_id, node_id)` | DELETE | `/controller/network/{net_id}/member/{node_id}` |
| `orbit_moon(world_id, seed)` | POST | `/moon/{world_id}` |
| `deorbit_moon(world_id)` | DELETE | `/moon/{world_id}` |
| `moons()` | GET | `/moon` |

### REST API (Backend → Frontend)

```
# Нода
GET  /api/local/status

# Подключённые сети
GET  /api/local/networks
GET  /api/local/networks/:id
POST /api/local/networks/:id           body: NetworkMembershipUpdate
DEL  /api/local/networks/:id

# Пиры
GET  /api/local/peers
GET  /api/local/peers/:node_id

# Контроллер — сети
GET  /api/local/controller/networks
POST /api/local/controller/networks    body: ControllerNetworkCreate (генерирует ID сам)
GET  /api/local/controller/networks/:id
PUT  /api/local/controller/networks/:id
DEL  /api/local/controller/networks/:id

# Контроллер — участники
GET  /api/local/controller/networks/:id/members
GET  /api/local/controller/networks/:id/members/:node_id
PUT  /api/local/controller/networks/:id/members/:node_id
DEL  /api/local/controller/networks/:id/members/:node_id

# Moon-серверы
GET  /api/local/moons
POST /api/local/moons/:world_id        body: { seed_id: String }
DEL  /api/local/moons/:world_id
```

### Критерии готовности
- [ ] Все методы покрыты handler'ами
- [ ] Ошибки ZT (4xx/5xx) → `ApiError::ZtLocal` с сохранением кода
- [ ] `leave_network` возвращает 204 если ZT ответил 200

---

## feat/part2-zt-central-api

**Цель:** типизированный клиент ZeroTier Central API с поддержкой нескольких токенов.

### Задачи

**`src/zerotier/central/types.rs`** — DTO Central API (отдельные от Local где структуры различаются):
- [ ] `CentralNetwork { id, config: CentralNetworkConfig, description, rules_source, permissions, owner_id, created_at, updated_at }`
- [ ] `CentralNetworkConfig { name, private, routes, ip_assignment_pools, v4_assign_mode, v6_assign_mode, mtu, multicast_limit, enable_broadcast, dns }`
- [ ] `CentralMember { node_id, name, description, authorized, active_bridge, no_auto_assign_ips, ip_assignments, capabilities, tags, network_id, last_online, physical_address, client_version, protocol_version, supports_rules_engine, sso_exempt, identity }`
- [ ] `CentralMemberUpdate` — все поля опциональны
- [ ] `CentralUser { id, display_name, email, sms_number }`
- [ ] `AccountStatus { id, display_name, email, auth, under_limit, plan_type }` где `plan_type` соответствует `RateLimit`
- [ ] `ApiTokenRecord { id, token_name, created_at, last_used }`

**`src/zerotier/central/client.rs`**:
- [ ] `ZtCentralClient { base_url: String, token: String, http: reqwest::Client }`
- [ ] `ZtCentralClient::new(base_url, token) -> Self`
- [ ] Rate limit enforcement: `Arc<RateLimiter>` (простая реализация через `tokio::time::Interval`)
  - `Free`: не более 20 запросов в секунду
  - `Paid`: не более 100 запросов в секунду
  - Настраивается из `CentralToken.rate_limit`

Методы:

| Метод | HTTP | Путь |
|---|---|---|
| `networks()` | GET | `/network` |
| `create_network(cfg)` | POST | `/network` |
| `network(id)` | GET | `/network/{id}` |
| `update_network(id, cfg)` | POST | `/network/{id}` |
| `delete_network(id)` | DELETE | `/network/{id}` |
| `network_members(net_id)` | GET | `/network/{net_id}/member` |
| `network_member(net_id, node_id)` | GET | `/network/{net_id}/member/{node_id}` |
| `update_member(net_id, node_id, cfg)` | PUT | `/network/{net_id}/member/{node_id}` |
| `delete_member(net_id, node_id)` | DELETE | `/network/{net_id}/member/{node_id}` |
| `user()` | GET | `/auth` |
| `account_status()` | GET | `/status` |
| `create_api_token(name)` | POST | `/auth/token` |
| `delete_api_token(token_id)` | DELETE | `/auth/token/{id}` |
| `random_token()` | GET | `/randomToken` |

### REST API (Backend → Frontend)

```
# Сети
GET  /api/central/networks
POST /api/central/networks            body: { name, description, private }
GET  /api/central/networks/:id
PUT  /api/central/networks/:id
DEL  /api/central/networks/:id

# Участники
GET  /api/central/networks/:id/members
GET  /api/central/networks/:id/members/:node_id
PUT  /api/central/networks/:id/members/:node_id
DEL  /api/central/networks/:id/members/:node_id

# Аккаунт
GET  /api/central/user
GET  /api/central/status
```

> Все эндпоинты `/api/central/*` работают с **активным** токеном из `CentralClientPool` (см. ниже)

### Критерии готовности
- [ ] Rate limiter не позволяет превысить лимит
- [ ] При 401 от Central API → `ApiError::ZtCentral { code: "AUTH_FAILED" }`

---

## feat/part2-token-store

**Цель:** управление набором ZeroTier Central API токенов — CRUD, валидация, активный токен.

### Задачи

**`src/zerotier/central/token_store.rs`**:
- [ ] `CentralClientPool { tokens: HashMap<String, ZtCentralClient>, active_id: Option<String>, base_url: String }`
- [ ] `CentralClientPool::from_config(cfg: &CentralConfig) -> Self`
- [ ] `active_client() -> Result<&ZtCentralClient>` — возвращает клиент активного токена или `ApiError::ZtCentral { code: "NO_ACTIVE_TOKEN" }`
- [ ] `add_token(&mut self, token: CentralToken) -> Result<()>`
- [ ] `remove_token(&mut self, id: &str) -> Result<()>` — если удаляется активный → сбрасывает `active_id`
- [ ] `set_active(&mut self, id: &str) -> Result<()>`
- [ ] `validate_token(&self, token_str: &str) -> Result<AccountStatus>` — делает `GET /status` с данным токеном (не сохраняя)
- [ ] При добавлении токена: автоматически определять `rate_limit` через `account_status().plan_type`
- [ ] После любого изменения: `Config::save()` для персистентности

**Маскировка токена при выдаче:**
- [ ] `fn mask_token(token: &str) -> String` — `ghp_AbCd...****` (первые 4 + маска)
- [ ] Все ответы `/api/settings/tokens` используют `masked_token`, реальный токен никогда не передаётся на фронт

### REST API

```
# Управление токенами
GET  /api/settings/tokens
     → [{ id, name, masked_token, rate_limit, created_at, is_active }]

POST /api/settings/tokens
     body: { name: String, token: String }
     → Валидирует токен → определяет rate_limit → сохраняет → { id, name, masked_token, rate_limit, is_active: false }

PUT  /api/settings/tokens/:id
     body: { name?: String, token?: String }
     → Если token изменён — перевалидировать

DEL  /api/settings/tokens/:id
     → 204, если был активным — active_id = null

POST /api/settings/tokens/:id/activate
     → Устанавливает активный токен → { is_active: true }

POST /api/settings/tokens/validate
     body: { token: String }
     → { valid: bool, account_status?: AccountStatus, rate_limit?: RateLimit }
     (для проверки нового токена до добавления)
```

### UI-поведение (требование для фронтенда, описано здесь для ясности)
- При добавлении первого токена — автоматически устанавливать активным
- При удалении активного — предупреждение что Central API недоступен
- Валидация токена происходит до сохранения (кнопка "Verify & Add")

### Критерии готовности
- [ ] Токены персистируются в `config.yml`
- [ ] Реальный токен никогда не покидает backend
- [ ] Валидация работает без side-effects

---

## feat/part2-metrics

**Цель:** периодический сбор ZeroTier метрик из Prometheus endpoint, кэш, JSON API.

### Задачи

**`src/metrics/parser.rs`** — нативный парсер Prometheus text format:
- [ ] Парсинг строк вида `metric_name{label="val",...} value [timestamp]`
- [ ] Игнорирование `# HELP` и `# TYPE` строк
- [ ] Структуры: `MetricSample { name, labels: HashMap<String,String>, value: f64, timestamp: Option<i64> }`
- [ ] `parse(input: &str) -> Vec<MetricSample>` — без panic, все ошибки skip + log

**`src/metrics/collector.rs`**:
- [ ] `MetricsCollector { url: String, client: reqwest::Client, interval: Duration }`
- [ ] `MetricsCollector::start(cache: Arc<MetricsCache>) -> JoinHandle<()>` — tokio task с loop
- [ ] При ошибке fetch — логировать, не паниковать, продолжать цикл
- [ ] Raw text сохранять в кэш вместе с parsed для `/metrics/raw`

**`src/metrics/cache.rs`**:
- [ ] `MetricsCache { parsed: RwLock<Option<MetricsSnapshot>>, raw: RwLock<Option<String>>, last_updated: RwLock<Option<DateTime<Utc>>> }`
- [ ] `MetricsSnapshot { packets: PacketMetrics, latency: LatencyMetrics, peers: Vec<PeerMetric>, networks: Vec<NetworkMetric>, errors: ErrorMetrics }`
- [ ] Конкретные типы для каждой группы метрик:
  - `PacketMetrics { rx_bytes, tx_bytes, rx_packets, tx_packets }` из `zt_packet`
  - `LatencyMetrics { avg_ms, min_ms, max_ms }` из `zt_latency`
  - `PeerMetric { node_id, status, latency_ms }` из `zt_peer_status`
  - `NetworkMetric { network_id, status }` из `zt_network_status`
  - `ErrorMetrics { total, by_type: HashMap<String, u64> }` из `zt_packet_error`

### REST API

```
GET /api/metrics         → MetricsSnapshot (JSON)
GET /api/metrics/raw     → Prometheus text (Content-Type: text/plain)
GET /api/metrics/status  → { enabled: bool, last_updated: DateTime?, error: String? }
```

### Критерии готовности
- [ ] Парсер работает без внешних crate
- [ ] Кэш доступен из множества потоков без deadlock
- [ ] При недоступном ZT метрики возвращают последний снимок с `last_updated`

---

## feat/part2-exitnode

**Цель:** backend для настройки Exit Node (выходная нода VPN), только Linux.

### Задачи

**`src/exitnode/platform.rs`**:
- [ ] `PlatformSupport { supported: bool, os: String, reason: Option<String> }`
- [ ] `check() -> PlatformSupport` — `cfg!(target_os = "linux")`, иначе reason с инструкцией

**`src/exitnode/deps.rs`**:
- [ ] `DepsStatus { iptables: Option<PathBuf>, nftables: Option<PathBuf>, is_root: bool, missing: Vec<String> }`
- [ ] `check_deps() -> DepsStatus` — через `which` crate + `nix::unistd::getuid() == 0`
- [ ] `install_missing(preferred: FirewallBackend) -> Result<DepsStatus>`
  - Определить PM: apt/dnf/pacman
  - Установить `nftables` или `iptables` через `std::process::Command`
  - НЕ использовать shell, curl, wget

**`src/exitnode/interfaces.rs`**:
- [ ] `NetworkInterface { name: String, addresses: Vec<String>, is_zerotier: bool }`
- [ ] `list_interfaces() -> Result<Vec<NetworkInterface>>`
  - Linux: парсинг `/proc/net/if_inet6` + `/proc/net/fib_trie`
  - Или через `getifaddrs` (nix crate)
  - ZeroTier интерфейсы: имя начинается с `zt` или MAC совпадает с ZT диапазоном

**`src/exitnode/rules.rs`**:
- [ ] `FirewallBackend` enum: `Nftables` | `Iptables`
- [ ] `ExitNodeRules { zt_iface: String, wan_iface: String, backend: FirewallBackend }`
- [ ] `ExitNodeRules::apply() -> Result<()>`:
  - `ip_forward`: запись `1` в `/proc/sys/net/ipv4/ip_forward` через `std::fs::write`
  - nftables: формирование и применение ruleset через `nft -f -` stdin pipe (без shell)
  - iptables: вызов `iptables -t nat -A POSTROUTING ...` через Command
- [ ] `ExitNodeRules::remove() -> Result<()>` — откат правил

**`src/exitnode/mod.rs`**:
- [ ] `ExitNodeState { enabled: bool, zt_network_id: Option<String>, wan_interface: Option<String>, backend: Option<FirewallBackend>, applied_at: Option<DateTime<Utc>> }`
- [ ] `ExitNodeManager { state: RwLock<ExitNodeState>, config: ExitNodeConfig }`
- [ ] `enable(zt_net_id, wan_iface) -> Result<ExitNodeState>`
- [ ] `disable() -> Result<()>`

### REST API

```
GET  /api/exitnode/platform       → PlatformSupport
GET  /api/exitnode/deps           → DepsStatus
POST /api/exitnode/deps/install   → DepsStatus (устанавливает недостающее)
GET  /api/exitnode/interfaces     → Vec<NetworkInterface>
GET  /api/exitnode/status         → ExitNodeState
POST /api/exitnode/enable         body: { zt_network_id: String, wan_interface: String }
POST /api/exitnode/disable
```

### Критерии готовности
- [ ] Нет вызовов curl/wget/shell-скриптов
- [ ] Root-проверка до выполнения операций — 403 ApiError если не root
- [ ] `remove()` полностью откатывает правила
