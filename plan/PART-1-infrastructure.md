# PART 1 — Infrastructure

> Ветки: `feat/part1-scaffold` → `feat/part1-config` → `feat/part1-zt-detection` → `feat/part1-http-server`

---

## feat/part1-scaffold

**Цель:** нулевая точка — структура, Cargo.toml, CI, конфиг-пример.

### Задачи

- [ ] `cargo init --name ztnet-box` (edition 2021, rust-version = "1.75")
- [ ] Создать полную структуру каталогов (см. [README](./README.md))
- [ ] Заполнить `Cargo.toml` зависимостями (все версии зафиксированы):

```toml
[dependencies]
axum          = { version = "0.7", features = ["macros"] }
tokio         = { version = "1",   features = ["full"] }
serde         = { version = "1",   features = ["derive"] }
serde_yaml    = "0.9"
serde_json    = "1"
reqwest       = { version = "0.12", features = ["json", "rustls-tls"], default-features = false }
thiserror     = "1"
tracing       = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "fmt"] }
tower-http    = { version = "0.5", features = ["cors", "compression-gzip", "set-header"] }
which         = "6"
uuid          = { version = "1", features = ["v4"] }
chrono        = { version = "0.4", features = ["serde"] }
nix           = { version = "0.28", features = ["user", "process"] }

[build-dependencies]
# только std, нет внешних build-deps
```

- [ ] `config.yml.example`:

```yaml
server:
  host: "127.0.0.1"   # привязка к localhost — безопасность без авторизации
  port: 3000

zerotier:
  local:
    api_url: "http://127.0.0.1:9993"
    token_file: "/var/lib/zerotier-one/authtoken.secret"
  central:
    base_url: "https://api.zerotier.com/api/v1"
    tokens: []          # управляется через Settings UI
    active_token_id: "" # ID активного токена

metrics:
  enabled: true
  prometheus_url: "http://127.0.0.1:9993/metrics"
  poll_interval_seconds: 5

exitnode:
  nftables_preferred: true  # true = nftables, false = iptables
```

- [ ] `.gitignore`: `target/`, `config.yml`, `www/build/`
- [ ] `CHANGELOG.md` (пустой шаблон Keep-a-Changelog)
- [ ] `.github/workflows/ci.yml`:

```yaml
name: CI
on: [push, pull_request]
jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with: { components: clippy }
      - run: cargo check
      - run: cargo clippy -- -D warnings
      - run: cargo test
```

- [ ] Тег `v0.1.0-alpha` на `main` после merge

### Критерии готовности
- [ ] `cargo check` — без ошибок и предупреждений
- [ ] CI проходит на push

---

## feat/part1-config

**Цель:** модуль загрузки/сохранения конфига с поддержкой ENV override и управления токенами.

### Задачи

**`src/config/schema.rs`** — типы конфига:
- [ ] `Config { server, zerotier, metrics, exitnode }`
- [ ] `ServerConfig { host: String, port: u16 }`
- [ ] `ZeroTierConfig { local: LocalConfig, central: CentralConfig }`
- [ ] `LocalConfig { api_url: String, token_file: PathBuf }`
- [ ] `CentralConfig { base_url: String, tokens: Vec<CentralToken>, active_token_id: String }`
- [ ] `CentralToken { id: String, name: String, token: String, rate_limit: RateLimit, created_at: DateTime<Utc> }`
- [ ] `RateLimit` enum: `Free` (20/s) | `Paid` (100/s)
- [ ] `MetricsConfig { enabled: bool, prometheus_url: String, poll_interval_seconds: u64 }`
- [ ] `ExitNodeConfig { nftables_preferred: bool }`

**`src/config/env.rs`** — ENV override:
- [ ] Маппинг: `ZT_SERVER_HOST`, `ZT_SERVER_PORT`, `ZT_LOCAL_API_URL`, `ZT_LOCAL_TOKEN_FILE`, `ZT_CENTRAL_BASE_URL`
- [ ] Функция `apply_env_overrides(config: &mut Config)` — мутирует конфиг после загрузки yml

**`src/config/mod.rs`** — публичный API:
- [ ] `Config::load(path: &Path) -> Result<Config>` — загрузка yml + env override + defaults
- [ ] `Config::save(&self, path: &Path) -> Result<()>` — запись yml (нужен для Settings UI)
- [ ] `Config::find_config_file() -> PathBuf` — поиск: `./config.yml` → `~/.config/ztnet-box/config.yml` → `/etc/ztnet-box/config.yml`
- [ ] Валидация: `port` 1–65535, URL форматы, непустой `token_file` путь

### API (REST)
```
GET  /api/settings/config          → Config (токены maskировать: первые 4 + ***)
PUT  /api/settings/config          → обновить server/zerotier.local/metrics/exitnode секции
```
> Токены управляются отдельными эндпоинтами (см. PART 2 / token-store)

### Критерии готовности
- [ ] `Config::load()` работает с любым из трёх путей
- [ ] ENV override перекрывает yml
- [ ] `Config::save()` не затирает комментарии (yaml round-trip через serde_yaml)

---

## feat/part1-zt-detection

**Цель:** обнаружение и автоустановка `zerotier-one` / `zerotier-idtool`.

### Задачи

**`src/zerotier/detection.rs`**:

- [ ] Структуры результата:
  ```rust
  pub struct ZtDetectionResult {
      pub zerotier_one: Option<PathBuf>,
      pub zerotier_idtool: Option<PathBuf>,
      pub version: Option<String>,        // парсится из `zerotier-cli info`
  }
  
  pub enum InstallResult {
      AlreadyInstalled(ZtDetectionResult),
      Installed(ZtDetectionResult),
      UnsupportedPlatform(String),
      Failed(String),
  }
  ```

- [ ] `detect() -> ZtDetectionResult` — поиск через `which` crate (не shell)
- [ ] `detect_package_manager() -> Option<PackageManager>` — проверка наличия: `apt-get`, `dnf/yum`, `pacman`, `brew`
- [ ] `install(pm: PackageManager) -> Result<InstallResult>` — нативный вызов PM через `std::process::Command` с явным PATH
- [ ] Поддерживаемые платформы для установки: Linux (apt/dnf/pacman), macOS (brew)
- [ ] Windows: возвращать `UnsupportedPlatform` с инструкцией

**Версия:**
- [ ] После установки перепроверить `detect()` и вернуть версию из stdout `zerotier-cli info`

### API (REST)
```
GET  /api/system/zt-status         → ZtDetectionResult
POST /api/system/zt-install        → InstallResult (запускает установку)
```

### Критерии готовности
- [ ] На системе без ZT возвращает `zerotier_one: null`
- [ ] Установка не использует curl/wget/shell-скрипты

---

## feat/part1-http-server

**Цель:** Axum HTTP сервер — скелет роутинга, middleware, error handling, отдача фронта.

### Задачи

**`src/server/state.rs`**:
- [ ] `AppState { config: Arc<RwLock<Config>>, config_path: PathBuf, zt_local: Arc<ZtLocalClient>, zt_central: Arc<CentralClientPool>, metrics_cache: Arc<MetricsCache>, exitnode_state: Arc<RwLock<ExitNodeState>> }`
- [ ] Конструктор `AppState::new(config, config_path) -> Result<Self>`

**`src/server/error.rs`**:
- [ ] `ApiError` enum с вариантами: `ZtLocal`, `ZtCentral`, `Config`, `ExitNode`, `NotFound`, `InvalidInput(String)`
- [ ] `impl IntoResponse for ApiError` → JSON `{ "error": "...", "code": "ERR_*" }` с правильным HTTP статусом

**`src/server/middleware.rs`**:
- [ ] Логирование каждого запроса: `method path → status latency` (через `tracing`)
- [ ] CORS: разрешить только `http://127.0.0.1:{port}` и `http://localhost:{port}` (из конфига)
- [ ] `Content-Security-Policy` header: `default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'`
- [ ] `X-Content-Type-Options: nosniff`, `X-Frame-Options: DENY`

**`src/server/router.rs`**:
- [ ] `build_router(state: AppState) -> Router` — регистрация всех маршрутов (все handler'ы в отдельных модулях)
- [ ] `GET /` → встроенный `index.html` (`include_str!("../../www/build/index.html")`)
- [ ] `GET /api/health` → `{ "status": "ok", "version": env!("CARGO_PKG_VERSION") }`
- [ ] Фоллбэк: любой не-API путь → `index.html` (SPA routing)

**`src/main.rs`**:
- [ ] Загрузка конфига → `AppState::new()` → `axum::serve(listener, router)`
- [ ] Логирование старта: адрес, версия, путь к конфигу

### Критерии готовности
- [ ] Сервер стартует и отдаёт `GET /api/health`
- [ ] Security headers присутствуют в каждом ответе
- [ ] CORS ограничен localhost
