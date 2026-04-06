# PART 2 — Backend Modules

> Ветки: `feat/part2-zt-local-api` → `feat/part2-zt-central-api` → `feat/part2-token-store` → `feat/part2-metrics` → `feat/part2-exitnode`

---

## feat/part2-zt-local-api ✅ merged #5

**Цель:** типизированный HTTP клиент к ZeroTier One Service API.

### Задачи

**`src/zerotier/local/types.rs`**:
- [x] `NodeStatus` (address, public_identity, world_id, clock, online, tcp_fallback_active, version)
- [x] `NetworkMembership` + `NetworkMembershipUpdate` (все поля Optional)
- [x] `PeerInfo` + `PeerPath`
- [x] `ControllerNetwork` + `ControllerNetworkCreate` + `ControllerMember` + `ControllerMemberUpdate`
- [x] `Moon`, `MoonRoot`, `OrbitRequest`
- [x] `Route`, `Dns`, `IpRange`, `V4AssignMode`, `V6AssignMode`
- [ ] `MemberDetails` — не реализован (не критично для PART 2)

**`src/zerotier/local/client.rs`**:
- [x] `ZtLocalClient::new()`, `from_config()`, generic `request<T>()` + `request_empty()`
- [x] node_status, networks CRUD (join/leave → 204), peers, controller networks CRUD, controller members CRUD, moons CRUD

**`src/server/handlers/local.rs`** — 19 хэндлеров:
- [x] `GET /api/local/status`
- [x] `GET/POST/DELETE /api/local/networks/:id`
- [x] `GET /api/local/peers`, `GET /api/local/peers/:id`
- [x] `GET/POST /api/local/controller/networks`
- [x] `GET/PUT/DELETE /api/local/controller/networks/:id`
- [x] `GET /api/local/controller/networks/:id/members`
- [x] `GET/PUT/DELETE /api/local/controller/networks/:id/members/:node_id`
- [x] `GET /api/local/moons`, `POST/DELETE /api/local/moons/:world_id`

---

## feat/part2-zt-central-api ✅ merged #6

**Цель:** типизированный клиент ZeroTier Central API с rate limiting.

### Задачи

**`src/zerotier/central/types.rs`**:
- [x] `CentralNetwork` + `CentralNetworkConfig` + `NetworkCreateOrUpdate`
- [x] `CentralMember` + `CentralMemberUpdate`
- [x] `CentralUser`, `AccountStatus` (с `rate_limit()` → `RateLimit`), `ApiTokenRecord`
- [x] `CentralRoute`, `CentralIpRange`, `CentralDns`

**`src/zerotier/central/client.rs`**:
- [x] `ZtCentralClient::new(base_url, token, rate_limit)`
- [x] `RateLimiter`: Semaphore-based, Free=20 req/s, Paid=100 req/s (tokio spawn)
- [x] networks CRUD (5 методов), members CRUD (4 методов)
- [x] `user()`, `account_status()`, `create_api_token()`, `delete_api_token()`, `random_token()`
- [x] 401 → `ApiError::ZtCentral("AUTH_FAILED")`, 404 → `ApiError::NotFound`

**`src/server/handlers/central.rs`** — 11 хэндлеров:
- [x] `GET/POST /api/central/networks`
- [x] `GET/PUT/DELETE /api/central/networks/:id`
- [x] `GET /api/central/networks/:id/members`
- [x] `GET/PUT/DELETE /api/central/networks/:id/members/:node_id`
- [x] `GET /api/central/user`, `GET /api/central/status`
- [x] NO_ACTIVE_TOKEN → 502 когда токен не настроен

---

## feat/part2-token-store ✅ merged #7

**Цель:** управление набором токенов ZeroTier Central API.

### Задачи

**`src/zerotier/central/token_store.rs`**:
- [x] `TokenStore { inner: Arc<RwLock<...>>, base_url }`
- [x] `active_client() -> Option<ZtCentralClient>`
- [x] `add()`, `remove()` (сброс active при удалении активного), `set_active()`, `list()`, `find()`
- [x] `with_base_url()` для конфигурации base URL
- [ ] `CentralClientPool` как замена `TokenStore` — не переименован (план изменён)
- [ ] `validate_token()` на уровне store — реализован в handler напрямую

**`src/server/handlers/tokens.rs`**:
- [x] `TokenView` (masked token, никогда raw)
- [x] `GET /api/settings/tokens`
- [x] `POST /api/settings/tokens` — валидирует через `account_status()`, определяет rate_limit, первый токен → активный
- [x] `PUT /api/settings/tokens/:id` — обновление с перевалидацией при смене токена
- [x] `DELETE /api/settings/tokens/:id` → 204
- [x] `POST /api/settings/tokens/:id/activate`
- [x] `POST /api/settings/tokens/validate` — без side-effects
- [x] `persist_tokens()` → `Config::save()` после каждой мутации

---

## feat/part2-metrics ✅ merged #8 (v0.2.0)

**Также включает: `fix(ci)` version.yml — 3 критических бага.**

### Задачи

**`src/metrics/parser.rs`**:
- [x] `MetricSample { name, labels: HashMap<String,String>, value, timestamp }`
- [x] `parse(input) -> Vec<MetricSample>` — парсит `name{k="v"} value [ts]`
- [x] Пропускает `# HELP` / `# TYPE`, не паникует на плохих строках
- [x] 6 unit-тестов: simple, labels, comments, timestamp, unparseable, multiple

**`src/metrics/cache.rs`**:
- [x] `MetricsSnapshot { packets: PacketMetrics, latency: LatencyMetrics, peers: Vec<PeerMetric>, networks: Vec<NetworkMetric>, errors: ErrorMetrics }`
- [x] `PacketMetrics`, `LatencyMetrics`, `PeerMetric`, `NetworkMetric`, `ErrorMetrics`
- [x] `MetricsCache { parsed, raw, last_updated, last_error }` (все `Arc<RwLock<...>>`)
- [x] `update_from_raw()` → parse → build_snapshot → store; `record_error()`
- [x] `snapshot()`, `raw_text()`, `last_updated()`, `last_error()`
- [x] `build_snapshot()`: маппинг MetricSample → typed fields по имени + labels
- [x] 3 tokio-теста: update+read, peers from labels, error recording

**`src/metrics/collector.rs`**:
- [x] `MetricsCollector::start(url, interval, cache)` — tokio task, loop с `tokio::time::interval`
- [x] При ошибке fetch — `record_error()` + `tracing::warn`, продолжает

**`src/server/handlers/metrics.rs`**:
- [x] `GET /api/metrics` → `MetricsSnapshot` JSON
- [x] `GET /api/metrics/raw` → `text/plain; version=0.0.4` Prometheus format
- [x] `GET /api/metrics/status` → `{ enabled, last_updated, error }`

**`fix(ci): .github/workflows/version.yml`**:
- [x] `grep -E '^version\s*='` + `sed -E` — устойчив к пробелам в Cargo.toml
- [x] `$((MAJOR+1))` вместо `${MAJOR+1}` — критический арифметический баг
- [x] Python через `sys.argv` вместо bash heredoc — избегает проблем с кавычками
- [x] Cargo.toml: нормализована строка `version = "x.y.z"` (убраны лишние пробелы)
- [x] Версия автоматически поднята до v0.2.0 после merge

---

## feat/part2-exitnode ⏳ следующая

**Цель:** backend для настройки Exit Node, только Linux.

### Задачи

**`src/exitnode/platform.rs`** — переписать стаб:
- [ ] `PlatformSupport { supported: bool, os: String, reason: Option<String> }`
- [ ] `check() -> PlatformSupport` — `cfg!(target_os = "linux")`, иначе reason

**`src/exitnode/deps.rs`** — переписать стаб:
- [ ] `DepsStatus { iptables: Option<PathBuf>, nftables: Option<PathBuf>, is_root: bool, missing: Vec<String> }`
- [ ] `check_deps() -> DepsStatus` — `which` + `nix::unistd::getuid() == 0`
- [ ] `install_missing(preferred) -> Result<DepsStatus>` — без shell/curl/wget

**`src/exitnode/interfaces.rs`** — переписать стаб:
- [ ] `NetworkInterface { name, addresses: Vec<String>, is_zerotier: bool }`
- [ ] `list_interfaces() -> Result<Vec<NetworkInterface>>` — через `/proc/net/dev` + nix

**`src/exitnode/rules.rs`** — переписать стаб:
- [ ] `ExitNodeRules { zt_iface, wan_iface, backend: FirewallBackend }`
- [ ] `apply()`: ip_forward (`fs::write "/proc/sys/net/ipv4/ip_forward"`) + nftables/iptables через `Command`
- [ ] `remove()`: полный откат правил

**`src/exitnode/mod.rs`** — переписать стаб:
- [ ] `ExitNodeState { enabled, zt_network_id, wan_interface, backend, applied_at }`
- [ ] `ExitNodeManager::enable(zt_net_id, wan_iface) -> Result<ExitNodeState>`
- [ ] `ExitNodeManager::disable() -> Result<()>`

**`src/server/handlers/exitnode.rs`** — новый файл:
- [ ] `GET  /api/exitnode/platform`
- [ ] `GET  /api/exitnode/deps`
- [ ] `POST /api/exitnode/deps/install`
- [ ] `GET  /api/exitnode/interfaces`
- [ ] `GET  /api/exitnode/status`
- [ ] `POST /api/exitnode/enable`
- [ ] `POST /api/exitnode/disable`

### Критерии готовности
- [ ] Нет curl/wget/shell-скриптов
- [ ] Root-проверка → `ApiError` 403 если не root
- [ ] `remove()` полностью откатывает правила
