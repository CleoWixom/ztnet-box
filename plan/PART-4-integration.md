# PART 4 — Integration, Security & Release

> Ветки: `feat/part4-integration` → `feat/part4-security` → `feat/part4-release` → `release/v0.1.0`

---

## feat/part4-integration ✅ merged (v0.5.1)

**Цель:** сквозное тестирование всех слоёв, проверка контрактов API.

### Задачи

**`tests/api_health.rs`** — интеграционные тесты (объединены в один файл):
- [x] Тест: `GET /api/health` → `{ status: "ok" }`
- [x] Тест: SPA fallback → `text/html`
- [x] Тест: security headers (X-Content-Type-Options, X-Frame-Options, CSP, Referrer-Policy)
- [x] Тест: `GET /api/settings/config` → структура без секретов
- [x] Тест: `PUT /api/settings/config` с port=0 → 422
- [x] Тест: `GET /api/settings/tokens` → пустой массив при старте
- [x] Тест: `POST /api/settings/tokens/validate` с фиктивным токеном → `{ valid: false }`
- [x] Тест: `GET /api/settings/tokens` не содержит raw token в теле
- [x] Тест: `GET /api/metrics/status` → структура с `enabled`
- [x] Тест: `GET /api/metrics/raw` → `Content-Type: text/plain`
- [x] Тест: `GET /api/exitnode/platform` → `{ supported, os }`
- [x] Тест: `GET /api/exitnode/deps` → `{ is_root, missing }`
- [x] Тест: `POST /api/exitnode/enable` без root → 403/422
- [x] Тест: `GET /api/system/zt-status` → `{ cli_available }`
- [x] Тест: invalid network_id → 422
- [x] Тест: invalid node_id → 422
- [x] Тест: тело > 64 KB → 413

**Unit тесты (в модулях `src/`):**
- [x] `metrics::cache` — 10 тестов: bytes from zt_data, packets from zt_packet,
  num_networks, peer latency histogram, peer path counts, peer packets,
  aggregate latency, network packets, errors with error_type label, empty metrics
- [x] `metrics::parser` — 6 тестов: simple, labels, comments, timestamp, unparseable, multiple
- [x] `server::validate` — 11 тестов: network_id, node_id, world_id, ip_addr, cidr
- [x] `config::schema` — masked_token тест
- [x] `exitnode::interfaces` — zerotier_detection тест
- [x] `server::router` — 6 тестов: health, body, index html, fallback, security_headers, central_no_token_502

### Критерии готовности
- [x] Все публичные API эндпоинты имеют хотя бы один тест
- [x] Нет тестов с моками данных в production-коде

---

## feat/part4-security ✅ merged (v0.6.0)

**Цель:** проверка и усиление безопасности — headers, bind, токены, валидация входных данных.

### Задачи

**Bind и сетевая изоляция:**
- [x] Default host `127.0.0.1` задокументирован в `config.yml.example`
- [x] При запуске логируется `WARN` если `host != "127.0.0.1"`
- [x] README: секция "Security Model"

**Security headers:**
- [x] `Content-Security-Policy`: `default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; connect-src 'self'`
- [x] `X-Content-Type-Options: nosniff`
- [x] `X-Frame-Options: DENY`
- [x] `Referrer-Policy: no-referrer`

**Токены:**
- [x] `CentralToken.token` аннотирован — никогда не возвращается напрямую из API
- [x] Сериализация в API всегда через `masked_token()` / `TokenView`

**Input validation:**
- [x] `server/validate.rs` — `network_id` (16 hex), `node_id` (10 hex), `world_id`, `ip_addr`, `cidr`
- [x] Все path-параметры в `/api/local/*` и `/api/central/*` валидируются → 422
- [x] `max_body_size`: `DefaultBodyLimit::max(64 * 1024)`

**CSRF:**
- [x] Нет Set-Cookie заголовков; нет сессий → CSRF не актуален, задокументировано в README

**Metrics auth fix:**
- [x] `metricstoken_file` добавлен в `MetricsConfig`
- [x] Коллектор читает токен и отправляет `Authorization: Bearer` на `/metrics` endpoint

### Критерии готовности
- [x] Все security headers присутствуют в каждом ответе
- [x] Реальный токен не утекает ни в одном API ответе
- [x] Некорректный ввод всегда даёт явную ошибку, а не panic

---

## feat/part4-release ✅ (v0.6.0)

**Цель:** cross-platform сборка, оптимизация, документация, GitHub Release.

### Задачи

**`Cargo.toml` — release profile:**
- [x] `opt-level = 3`, `lto = true`, `codegen-units = 1`, `strip = true`, `panic = "abort"`

**Cross-platform targets (CI `build-check` матрица):**
- [x] `x86_64-unknown-linux-gnu`
- [x] `aarch64-unknown-linux-gnu` (через `cross`)
- [x] `x86_64-apple-darwin`
- [x] `aarch64-apple-darwin`
- [x] `x86_64-pc-windows-msvc`

**`.github/workflows/release.yml`:**
- [x] Триггер: push тега `v[0-9]+.[0-9]+.[0-9]+`
- [x] Matrix сборка: Linux (cross), macOS (native), Windows
- [x] Артефакты: `ztnet-box-{version}-{target}.tar.gz` / `.zip`
- [x] Upload в GitHub Releases с release notes из CHANGELOG

**`README.md`:**
- [x] Описание проекта
- [x] Секция: Требования
- [x] Секция: Установка (таблица платформ)
- [x] Секция: Запуск
- [x] Секция: Конфигурация (таблица параметров + ENV)
- [x] Секция: Security Model
- [x] Секция: Central API Tokens
- [x] Секция: Exit Node

**`CHANGELOG.md`:**
- [x] Заполнен автоматически по Conventional Commits через `version.yml`

### Критерии готовности
- [x] `[profile.release]` настроен
- [x] GitHub Release workflow охватывает все 5 targets
- [x] `config.yml.example` присутствует в релизных артефактах

---

## release/v0.1.0 → фактически v0.6.0

Финальный чеклист:

- [x] Все ветки PART 1–4 смержены в `main`
- [x] `cargo test` — проходят на CI (ubuntu-latest)
- [x] `cargo clippy -- -D warnings` — проверяется в CI
- [x] Ручная проверка: все страницы UI рендерятся (проверено в build #10)
- [x] README.md заполнен
- [x] CHANGELOG.md актуален
- [ ] `git tag v0.6.0 && git push origin v0.6.0` — тег для финального релиза

> **Примечание:** в ходе разработки автоматический version workflow дошёл до v0.6.0.
> Тег `v0.1.0` не создавался — первый релизный тег должен быть `v0.6.0`.

---

## Сводная таблица API эндпоинтов

| Метод | Путь | Описание |
|---|---|---|
| GET | `/api/health` | Healthcheck |
| GET | `/api/system/zt-status` | Статус ZeroTier в системе |
| POST | `/api/system/zt-install` | Установить ZeroTier |
| GET | `/api/local/status` | Статус локальной ноды |
| GET | `/api/local/networks` | Подключённые сети |
| GET | `/api/local/networks/:id` | Сеть по ID |
| POST | `/api/local/networks/:id` | Join / Update |
| DELETE | `/api/local/networks/:id` | Leave |
| GET | `/api/local/peers` | Все пиры |
| GET | `/api/local/peers/:node_id` | Пир по ID |
| GET | `/api/local/moons` | Moon-серверы |
| POST | `/api/local/moons/:world_id` | Добавить moon |
| DELETE | `/api/local/moons/:world_id` | Удалить moon |
| GET | `/api/local/controller/networks` | Сети контроллера |
| POST | `/api/local/controller/networks` | Создать сеть |
| GET | `/api/local/controller/networks/:id` | Сеть контроллера |
| PUT | `/api/local/controller/networks/:id` | Обновить сеть |
| DELETE | `/api/local/controller/networks/:id` | Удалить сеть |
| GET | `/api/local/controller/networks/:id/members` | Участники |
| GET | `/api/local/controller/networks/:id/members/:node_id` | Участник |
| PUT | `/api/local/controller/networks/:id/members/:node_id` | Обновить |
| DELETE | `/api/local/controller/networks/:id/members/:node_id` | Удалить |
| GET | `/api/central/networks` | Сети Central API |
| POST | `/api/central/networks` | Создать сеть |
| GET | `/api/central/networks/:id` | Сеть |
| PUT | `/api/central/networks/:id` | Обновить |
| DELETE | `/api/central/networks/:id` | Удалить |
| GET | `/api/central/networks/:id/members` | Участники |
| GET | `/api/central/networks/:id/members/:node_id` | Участник |
| PUT | `/api/central/networks/:id/members/:node_id` | Обновить |
| DELETE | `/api/central/networks/:id/members/:node_id` | Удалить |
| GET | `/api/central/user` | Профиль Central аккаунта |
| GET | `/api/central/status` | Статус аккаунта |
| GET | `/api/metrics` | Метрики (JSON) |
| GET | `/api/metrics/raw` | Метрики (Prometheus text) |
| GET | `/api/metrics/status` | Статус сборщика метрик |
| GET | `/api/exitnode/platform` | Проверка платформы |
| GET | `/api/exitnode/deps` | Статус зависимостей |
| POST | `/api/exitnode/deps/install` | Установить зависимости |
| GET | `/api/exitnode/interfaces` | Сетевые интерфейсы |
| GET | `/api/exitnode/status` | Статус Exit Node |
| POST | `/api/exitnode/enable` | Включить |
| POST | `/api/exitnode/disable` | Выключить |
| GET | `/api/settings/config` | Конфигурация (без секретов) |
| PUT | `/api/settings/config` | Обновить конфигурацию |
| GET | `/api/settings/tokens` | Список токенов (masked) |
| POST | `/api/settings/tokens` | Добавить токен |
| PUT | `/api/settings/tokens/:id` | Обновить токен |
| DELETE | `/api/settings/tokens/:id` | Удалить токен |
| POST | `/api/settings/tokens/:id/activate` | Установить активным |
| POST | `/api/settings/tokens/validate` | Проверить токен (без сохранения) |
