# PART 1 — Infrastructure

> Ветки: `feat/part1-scaffold` → `feat/part1-config` → `feat/part1-zt-detection` → `feat/part1-http-server`

---

## feat/part1-scaffold ✅ merged #1

**Цель:** нулевая точка — структура, Cargo.toml, CI, конфиг-пример.

### Задачи

- [x] `cargo init --name ztnet-box` (edition 2021, rust-version = "1.75")
- [x] Создать полную структуру каталогов (см. [README](./README.md))
- [x] Заполнить `Cargo.toml` зависимостями (все версии зафиксированы):

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

- [x] `config.yml.example`
- [x] `.gitignore`: `target/`, `config.yml`, `www/build/`
- [x] `CHANGELOG.md` (пустой шаблон Keep-a-Changelog)
- [x] `.github/workflows/ci.yml` — уже был в репо
- [ ] Тег `v0.1.0-alpha` на `main` после merge

### Критерии готовности
- [x] `cargo check` — без ошибок и предупреждений
- [x] CI проходит на push

---

## feat/part1-config ✅ merged #2

**Цель:** модуль загрузки/сохранения конфига с поддержкой ENV override и управления токенами.

### Задачи

**`src/config/schema.rs`** — типы конфига:
- [x] `Config { server, zerotier, metrics, exitnode }`
- [x] `ServerConfig { host: String, port: u16 }`
- [x] `ZeroTierConfig { local: LocalConfig, central: CentralConfig }`
- [x] `LocalConfig { api_url: String, token_file: PathBuf }`
- [x] `CentralConfig { base_url: String, tokens: Vec<CentralToken>, active_token_id: String }`
- [x] `CentralToken { id: String, name: String, token: String, rate_limit: RateLimit, created_at: DateTime<Utc> }`
- [x] `RateLimit` enum: `Free` (20/s) | `Paid` (100/s)
- [x] `MetricsConfig { enabled: bool, prometheus_url: String, poll_interval_seconds: u64 }`
- [x] `ExitNodeConfig { nftables_preferred: bool }`

**`src/config/env.rs`** — ENV override:
- [x] Маппинг: `ZT_SERVER_HOST`, `ZT_SERVER_PORT`, `ZT_LOCAL_API_URL`, `ZT_LOCAL_TOKEN_FILE`, `ZT_CENTRAL_BASE_URL`
- [x] Функция `apply_env_overrides(config: &mut Config)`

**`src/config/mod.rs`** — публичный API:
- [x] `Config::load(path: &Path) -> Result<Config>`
- [x] `Config::save(&self, path: &Path) -> Result<()>`
- [x] `Config::find_config_file() -> PathBuf`
- [x] Валидация: `port` 1–65535, непустой `host`

### API (REST)
- [x] `GET  /api/settings/config` → Config (токены маскированы: первые 4 + ***)
- [x] `PUT  /api/settings/config` → обновить server/zerotier.local/metrics/exitnode секции

### Критерии готовности
- [x] `Config::load()` работает с любым из трёх путей
- [x] ENV override перекрывает yml
- [x] `Config::save()` работает

---

## feat/part1-zt-detection ✅ merged #3

**Цель:** обнаружение и автоустановка `zerotier-one` / `zerotier-idtool`.

### Задачи

**`src/zerotier/detection.rs`**:

- [x] Структуры результата: `ZtDetectionResult`, `InstallResult`, `PackageManager`
- [x] `detect() -> ZtDetectionResult` — поиск через `which` crate + `cli_available` флаг
- [x] `detect_package_manager() -> Option<PackageManager>` — apt-get, dnf/yum, pacman, brew
- [x] `install(pm: PackageManager) -> Result<InstallResult>` — нативный вызов PM без shell
- [x] Поддерживаемые платформы: Linux (apt/dnf/pacman), macOS (brew)
- [x] Windows: возвращает `UnsupportedPlatform` с инструкцией
- [x] Версия из stdout `zerotier-cli info` (парсинг "200 info <id> <ver> ONLINE")

### API (REST)
- [x] `GET  /api/system/zt-status`
- [x] `POST /api/system/zt-install`

### Критерии готовности
- [x] На системе без ZT возвращает `zerotier_one: null`
- [x] Установка не использует curl/wget/shell-скрипты
- [x] 3 unit-теста: detect, detect_package_manager, version_parse_format

---

## feat/part1-http-server ✅ merged #4

**Цель:** Axum HTTP сервер — скелет роутинга, middleware, error handling, отдача фронта.

### Задачи

**`src/server/state.rs`**:
- [x] `AppState { config, config_path, token_store, metrics_cache, exitnode }`
- [x] `AppState::new(config, config_path) -> Result<Self>`
- [x] `AppState::new_with_cache(config, config_path, cache) -> Result<Self>`

**`src/server/error.rs`**:
- [x] `ApiError` enum: `ZtLocal`, `ZtCentral`, `Config`, `ExitNode`, `NotFound`, `InvalidInput`, `Internal`
- [x] `impl IntoResponse for ApiError` → JSON `{ "error": "...", "code": "ERR_*" }`

**`src/server/middleware.rs`**:
- [x] Логирование каждого запроса: `method path → status latency_ms`
- [x] CORS: разрешить только `http://{host}:{port}` и `http://localhost:{port}`
- [ ] `Content-Security-Policy` header — есть, но задан статически, не из конфига

**`src/server/router.rs`**:
- [x] `build_router(state, host, port) -> Router`
- [x] `GET /` → встроенный `index.html`
- [x] `GET /api/health` → `{ "status": "ok", "version": ... }`
- [x] Фоллбэк: любой не-API путь → `index.html`

**`src/main.rs`**:
- [x] Загрузка конфига → `AppState::new()` → `axum::serve(listener, router)`
- [x] MetricsCollector.spawn() при `metrics.enabled = true`
- [x] Логирование старта: адрес, версия, путь к конфигу

### Критерии готовности
- [x] Сервер стартует и отдаёт `GET /api/health`
- [x] Security headers присутствуют в каждом ответе (тест)
- [x] CORS ограничен localhost (конфигурируется из host/port)
- [x] 10 unit/integration тестов через axum oneshot

---

## Workflows & Version Management ✅ (были в репо)

- [x] `.github/workflows/ci.yml` — cargo fmt + clippy + check + test + build-check matrix
- [x] `.github/workflows/version.yml` — автобамп по Conventional Commits
- [x] `.github/workflows/release.yml` — matrix build + GitHub Release
- [x] `.github/workflows/pr.yml` — валидация заголовка PR + скан секретов
- [x] `.github/COMMIT_CONVENTION.md`
- [x] `CHANGELOG.md` шаблон

### Критерии готовности
- [x] Push `.md` → ни один workflow не запускается (paths filter)
- [x] Push `src/` → CI + Version workflow
- [x] PR с неверным заголовком → pr.yml падает
