# PART 4 — Integration, Security & Release

> Ветки: `feat/part4-integration` → `feat/part4-security` → `feat/part4-release` → `release/v0.1.0`

---

## feat/part4-integration

**Цель:** сквозное тестирование всех слоёв, проверка контрактов API.

### Задачи

**`tests/api_local.rs`** — интеграционные тесты Local API:
- [ ] Тест: `GET /api/health` → `{ status: "ok" }`
- [ ] Тест: `GET /api/local/status` — корректный JSON при живом ZT, ApiError при недоступном
- [ ] Тест: `GET /api/local/networks` → массив
- [ ] Тест: join/leave cycle — `POST` + `DELETE /api/local/networks/{test_id}`
- [ ] Тест: `GET /api/local/controller/networks` → массив
- [ ] Тест: create/delete network cycle
- [ ] Тест: add/update/delete member cycle

**`tests/api_tokens.rs`** — тесты управления токенами:
- [ ] Тест: `GET /api/settings/tokens` → пустой массив при старте
- [ ] Тест: `POST /api/settings/tokens/validate` с фиктивным токеном → `{ valid: false }`
- [ ] Тест: `POST /api/settings/tokens` с некорректным токеном → ошибка
- [ ] Тест: добавление → активация → удаление cycle
- [ ] Тест: удаление активного токена → `active_token_id` сбрасывается
- [ ] Тест: `GET /api/settings/tokens` никогда не возвращает реальный токен в поле `token`

**`tests/api_metrics.rs`**:
- [ ] Тест: `GET /api/metrics` → структура `MetricsSnapshot` даже без ZT метрик
- [ ] Тест: `GET /api/metrics/raw` → `Content-Type: text/plain`
- [ ] Тест: парсер — unit тест с примером Prometheus text format

**`tests/api_exitnode.rs`**:
- [ ] Тест: `GET /api/exitnode/platform` → всегда возвращает структуру
- [ ] Тест: `GET /api/exitnode/deps` → структура `DepsStatus`
- [ ] Тест: `POST /api/exitnode/enable` без root → 403 ApiError

**`tests/api_config.rs`**:
- [ ] Тест: `GET /api/settings/config` → структура без секретов
- [ ] Тест: `PUT /api/settings/config` → изменения персистируются

**Unit тесты** (в модулях `src/`):
- [ ] `config::schema` — сериализация/десериализация round-trip
- [ ] `config::env` — ENV override перекрывает yml значения
- [ ] `metrics::parser` — парсинг корректного и некорректного Prometheus text
- [ ] `zerotier::central::token_store` — `mask_token()` корректно маскирует
- [ ] `exitnode::rules` — генерация nftables/iptables правил (без применения)

### Критерии готовности
- [ ] `cargo test` — все тесты проходят
- [ ] Покрытие: все публичные API эндпоинты имеют хотя бы один тест
- [ ] Нет тестов с моками данных в production-коде

---

## feat/part4-security

**Цель:** проверка и усиление безопасности — headers, bind, токены.

### Задачи

**Bind и сетевая изоляция:**
- [ ] Default host `127.0.0.1` — жёстко задокументирован в `config.yml.example` с комментарием о безопасности
- [ ] При запуске логировать предупреждение если `host != "127.0.0.1"`: `WARN: Server bound to {host} — ensure network-level access control`
- [ ] README: секция "Security Model" — объяснение что авторизация не требуется именно из-за bind к localhost

**Security headers** (проверка что все заданы в middleware):
- [ ] `Content-Security-Policy`: `default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; connect-src 'self'`
  - `img-src 'self' data:` нужен для QR-кода (data URL из canvas)
  - `connect-src 'self'` — fetch только к своему origin
- [ ] `X-Content-Type-Options: nosniff`
- [ ] `X-Frame-Options: DENY`
- [ ] `Referrer-Policy: no-referrer`
- [ ] Нет `X-Powered-By` и аналогичных информационных заголовков

**Токены:**
- [ ] Аудит: grep по всему коду что нигде не логируется и не сериализуется реальный токен
- [ ] `CentralToken` — поле `token` помечено `#[serde(skip_serializing)]` для safety (сериализация только при явном сохранении в файл)
- [ ] При сохранении config — отдельный метод `Config::save_with_secrets()` vs `Config::to_display()` (без токенов)

**CSRF:**
- [ ] Поскольку нет сессий и нет cookies — CSRF не актуален. Задокументировать это явно в README.
- [ ] Проверить что нет Set-Cookie заголовков в ответах сервера

**Input validation:**
- [ ] Все path параметры (network_id, node_id) — валидация формата (hex, длина)
- [ ] Все body параметры — `serde` десериализация с явными ограничениями типов
- [ ] IP адреса — валидация через `std::net::IpAddr::parse()`
- [ ] CIDR — валидация формата `addr/prefix`
- [ ] `max_body_size` middleware: ограничение тела запроса (например, 64 KB)

**Тест безопасности:**
- [ ] `tests/security.rs`:
  - [ ] Проверка наличия всех security headers в ответах
  - [ ] `GET /api/settings/tokens` не содержит реальный токен в теле ответа
  - [ ] Некорректные path params → 400/422, не 500

### Критерии готовности
- [ ] Все security headers присутствуют в каждом ответе
- [ ] Реальный токен не утекает ни в одном API ответе
- [ ] Некорректный ввод всегда даёт явную ошибку, а не panic

---

## feat/part4-release

**Цель:** cross-platform сборка, оптимизация, документация, GitHub Release.

### Задачи

**`Cargo.toml` — release profile:**
```toml
[profile.release]
opt-level     = 3
lto           = true
codegen-units = 1
strip         = true
panic         = "abort"
```

**Cross-platform targets:**
- [ ] `x86_64-unknown-linux-gnu` (основная)
- [ ] `aarch64-unknown-linux-gnu` (ARM, Raspberry Pi, серверы)
- [ ] `x86_64-apple-darwin`
- [ ] `aarch64-apple-darwin` (Apple Silicon)
- [ ] `x86_64-pc-windows-msvc`

**`.github/workflows/release.yml`:**
- [ ] Триггер: push тега `v[0-9]+.[0-9]+.[0-9]+`
- [ ] Matrix сборка: Linux (через `cross` crate), macOS (native runners), Windows
- [ ] Артефакты: `ztnet-box-{target}.tar.gz` (Linux/macOS) и `ztnet-box-{target}.zip` (Windows)
- [ ] Upload в GitHub Releases с авто-генерацией release notes из CHANGELOG

**`README.md`:**
- [ ] Описание проекта (1 абзац)
- [ ] **Секция: Требования** — zerotier-one (или авто-установка), root для Exit Node
- [ ] **Секция: Установка** — скачать бинарник или `cargo install`
- [ ] **Секция: Запуск**:
  ```bash
  cp config.yml.example config.yml
  # отредактировать config.yml
  ./ztnet-box
  # открыть http://127.0.0.1:3000
  ```
- [ ] **Секция: Конфигурация** — таблица всех параметров config.yml + ENV vars
- [ ] **Секция: Security Model** — почему нет авторизации, bind к localhost
- [ ] **Секция: Central API Tokens** — как добавить токены через Settings UI
- [ ] **Секция: Exit Node** — требования (Linux, root, iptables/nftables)

**`CHANGELOG.md`** — заполнить секцию `[0.1.0]` по итогам всех веток.

**Финальные проверки:**
- [ ] `cargo build --release` → один бинарник, нет runtime зависимостей кроме ZT daemon
- [ ] Размер бинарника с встроенным фронтендом — задокументировать
- [ ] `cargo clippy --release -- -D warnings` — чисто
- [ ] `cargo audit` — нет known vulnerabilities в зависимостях

**Тег и релиз:**
- [ ] Merge `feat/part4-release` → `main`
- [ ] `git tag v0.1.0`
- [ ] `git push origin v0.1.0` → триггер release workflow

### Критерии готовности
- [ ] Бинарник запускается на каждой целевой платформе
- [ ] GitHub Release содержит бинарники для всех 5 targets
- [ ] `config.yml.example` присутствует в релизе

---

## release/v0.1.0

Финальный чеклист перед тегом:

- [ ] Все ветки PART 1–4 смержены в `main`
- [ ] `cargo test` — все тесты зелёные
- [ ] `cargo clippy -- -D warnings` — 0 предупреждений
- [ ] Ручная проверка: все страницы UI рендерятся
- [ ] Ручная проверка: add/activate/delete token flow работает end-to-end
- [ ] Ручная проверка: Exit Node enable/disable (на Linux с root)
- [ ] CHANGELOG.md заполнен
- [ ] `git tag v0.1.0 && git push origin v0.1.0`

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
