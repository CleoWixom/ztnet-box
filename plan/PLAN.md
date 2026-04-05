# ZeroBox WebUI — Implementation Plan

> **Stack:** Backend — Rust (latest stable) · Frontend — HTML5/CSS3/JS · Config — YAML  
> **Repo:** `ztnet-box`  
> Ветки следуют **строго последовательно**. Каждая ветка — завершённый, компилируемый модуль без заглушек.

---

## Branch Map (последовательность)

```
main
 └── feat/project-scaffold          [1] Scaffold: Cargo, структура, конфиг
      └── feat/backend-core          [2] Backend core: config, ZT detection, HTTP server
           └── feat/backend-zt-local [3] ZT Local API client (zerotier-one)
                └── feat/backend-zt-central [4] ZT Central API client
                     └── feat/backend-metrics [5] Prometheus metrics proxy
                          └── feat/backend-exitnode [6] Exit Node логика
                               └── feat/frontend-build  [7] Frontend build pipeline (→ index.html)
                                    └── feat/frontend-dashboard [8] UI: Dashboard
                                         └── feat/frontend-networks [9] UI: My Networks + Details
                                              └── feat/frontend-controllers [10] UI: Controllers
                                                   └── feat/frontend-exitnode [11] UI: Exit Node
                                                        └── feat/frontend-settings [12] UI: Settings / Root Servers
                                                             └── feat/integration [13] Backend ↔ Frontend интеграция
                                                                  └── feat/packaging [14] Сборка, оптимизация, cross-platform
                                                                       └── release/v0.1.0
```

---

## [1] `feat/project-scaffold`

**Цель:** нулевая точка — структура проекта, Cargo.toml, конфиг-схема, CI.

### Задачи
- Инициализировать `cargo init --name ztnet-box`
- Определить структуру каталогов:
  ```
  ztnet-box/
  ├── Cargo.toml
  ├── config.yml.example
  ├── plan/
  │   └── PLAN.md
  ├── src/
  │   ├── main.rs
  │   ├── config/          # загрузка/валидация конфига
  │   ├── zerotier/        # ZT API клиенты
  │   ├── server/          # HTTP сервер
  │   ├── metrics/         # метрики proxy
  │   └── exitnode/        # Exit Node управление
  └── www/                 # исходники фронтенда (до сборки)
      ├── src/
      │   ├── html/
      │   ├── css/
      │   └── js/
      └── build/           # результат сборки → встраивается в бинарник
  ```
- `config.yml.example` — всё без хардкода:
  ```yaml
  server:
    host: "127.0.0.1"
    port: 3000
  zerotier:
    local:
      socket: "/var/lib/zerotier-one/zerotier.sock"   # или "http://127.0.0.1:9993"
      token_file: "/var/lib/zerotier-one/authtoken.secret"
    central:
      base_url: "https://api.zerotier.com/api/v1"
      api_token: ""   # задаётся через ZT_CENTRAL_TOKEN env или этот файл
  metrics:
    enabled: true
    zt_prometheus_url: "http://127.0.0.1:9993/metrics"
  exitnode:
    supported_platforms: ["linux"]
  ```
- Настроить `.github/workflows/ci.yml`: `cargo check`, `cargo clippy -- -D warnings`, `cargo test`
- Версионирование: `CHANGELOG.md` + тег `v0.1.0-alpha` на `main`

### Критерии готовности
- `cargo check` — без ошибок
- Структура каталогов зафиксирована
- CI проходит на push

---

## [2] `feat/backend-core`

**Цель:** Config loader, определение ZeroTier в системе, HTTP-сервер (Axum).

### Зависимости (Cargo.toml)
```toml
axum          = { version = "0.7", features = ["macros"] }
tokio         = { version = "1", features = ["full"] }
serde         = { version = "1", features = ["derive"] }
serde_yaml    = "0.9"
serde_json    = "1"
reqwest       = { version = "0.12", features = ["json", "rustls-tls"], default-features = false }
thiserror     = "1"
tracing       = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
tower-http    = { version = "0.5", features = ["fs", "cors", "compression-gzip"] }
tokio-util    = "0.7"
which         = "6"
```

### Модуль `src/config/`
- `mod.rs` — публичный API: `Config::load() -> Result<Config>`
- `schema.rs` — структуры `Config`, `ServerConfig`, `ZeroTierConfig`, `MetricsConfig`, `ExitNodeConfig`
- Приоритет: ENV vars > config.yml > defaults
- ENV маппинг: `ZT_SERVER_HOST`, `ZT_SERVER_PORT`, `ZT_CENTRAL_TOKEN`, `ZT_LOCAL_SOCKET`, `ZT_LOCAL_TOKEN_FILE`

### Модуль `src/zerotier/detection.rs`
- Проверка наличия `zerotier-one` и `zerotier-idtool` через `which` crate
- Если отсутствует — попытка установки через системный пакетный менеджер:
  - Linux: `apt` / `yum` / `pacman` (определять через наличие бинарника)
  - macOS: `brew`
  - Windows: winget / прямая ссылка (сообщение пользователю)
- Возвращает `ZtInstallStatus { zerotier_one: PathBuf, idtool: PathBuf, version: String }`

### Модуль `src/server/`
- `router.rs` — регистрация всех маршрутов (без логики)
- `state.rs` — `AppState { config, zt_local_client, zt_central_client, metrics_client }`
- `error.rs` — единый тип `ApiError` → JSON `{ "error": "...", "code": "..." }`
- `middleware.rs` — логирование запросов, CORS

### Критерии готовности
- Сервер стартует, читает конфиг, логирует
- `GET /api/health` возвращает `{ "status": "ok", "zt_installed": true/false }`

---

## [3] `feat/backend-zt-local`

**Цель:** Клиент ZeroTier One Service API (локальный контроллер).

### Модуль `src/zerotier/local/`
- `client.rs` — `ZtLocalClient` с базовым URL и auth-токеном
- Все методы — типизированные запросы/ответы через serde:

| Метод | Endpoint | Описание |
|---|---|---|
| `node_status()` | `GET /status` | Статус ноды |
| `networks()` | `GET /network` | Все подключённые сети |
| `network(id)` | `GET /network/{id}` | Сеть по ID |
| `join_network(id, cfg)` | `POST /network/{id}` | Подключиться/обновить |
| `leave_network(id)` | `DELETE /network/{id}` | Отключиться |
| `peers()` | `GET /peer` | Все пиры |
| `peer(node_id)` | `GET /peer/{node_id}` | Пир по ID |
| `controller_status()` | `GET /controller` | Статус контроллера |
| `controller_networks()` | `GET /controller/network` | Список сетей контроллера |
| `controller_network(id)` | `GET /controller/network/{id}` | Сеть контроллера |
| `create_network(cfg)` | `POST /controller/network/{id}` | Создать/обновить сеть |
| `random_network_id()` | `POST /controller/network/generate-id` | Случайный ID |
| `network_members(net_id)` | `GET /controller/network/{id}/member` | Участники |
| `network_member(net_id, node_id)` | `GET /controller/network/{id}/member/{node}` | Участник |
| `update_member(net_id, node_id, cfg)` | `POST /controller/network/{id}/member/{node}` | Изменить участника |
| `delete_member(net_id, node_id)` | `DELETE /controller/network/{id}/member/{node}` | Удалить |

- `types.rs` — все DTO: `NodeStatus`, `NetworkConfig`, `MemberConfig`, `PeerInfo` и т.д.
- Чтение токена из файла (путь из конфига), не хардкод

### REST API (Backend → Frontend)
```
GET  /api/local/status
GET  /api/local/networks
GET  /api/local/networks/:id
POST /api/local/networks/:id
DEL  /api/local/networks/:id
GET  /api/local/peers
GET  /api/local/peers/:node_id
GET  /api/local/controller/networks
POST /api/local/controller/networks
GET  /api/local/controller/networks/:id
PUT  /api/local/controller/networks/:id
DEL  /api/local/controller/networks/:id
GET  /api/local/controller/networks/:id/members
POST /api/local/controller/networks/:id/members/:node_id
PUT  /api/local/controller/networks/:id/members/:node_id
DEL  /api/local/controller/networks/:id/members/:node_id
```

### Критерии готовности
- Все методы вызываются, возвращают типизированный ответ
- Ошибки ZT API прокидываются в `ApiError`

---

## [4] `feat/backend-zt-central`

**Цель:** Клиент ZeroTier Central API (облачный контроллер, `api.zerotier.com`).

### Модуль `src/zerotier/central/`
- `client.rs` — `ZtCentralClient` (токен из конфига/env)
- Rate limiting: `tower` middleware с учётом лимитов (paid: 100/s, free: 20/s) — настраивается в конфиге
- Методы:

| Метод | Описание |
|---|---|
| `networks()` | Список доступных сетей |
| `create_network(cfg)` | Создать сеть |
| `network(id)` | Сеть по ID |
| `update_network(id, cfg)` | Обновить конфигурацию |
| `delete_network(id)` | Удалить |
| `network_members(id)` | Участники сети |
| `network_member(net_id, node_id)` | Конкретный участник |
| `update_member(net_id, node_id, cfg)` | Изменить |
| `delete_member(net_id, node_id)` | Удалить |
| `user()` | Профиль пользователя |
| `account_status()` | Статус аккаунта |
| `create_api_token(name)` | Новый токен |
| `delete_api_token(id)` | Удалить токен |

- `types.rs` — DTO Central API (отдельные от Local, где структуры отличаются)

### REST API расширение
```
GET  /api/central/networks
POST /api/central/networks
GET  /api/central/networks/:id
PUT  /api/central/networks/:id
DEL  /api/central/networks/:id
GET  /api/central/networks/:id/members
GET  /api/central/networks/:id/members/:node_id
PUT  /api/central/networks/:id/members/:node_id
DEL  /api/central/networks/:id/members/:node_id
GET  /api/central/user
GET  /api/central/status
```

### Критерии готовности
- Все эндпоинты Central API покрыты
- Rate limiting активен

---

## [5] `feat/backend-metrics`

**Цель:** Прокси ZeroTier Prometheus метрик → JSON для фронтенда.

### Модуль `src/metrics/`
- `collector.rs` — периодический fetch `http://127.0.0.1:9993/metrics` (URL из конфига)
- `parser.rs` — парсинг Prometheus text format без внешних crate:
  - Нативный парсер: `# HELP`, `# TYPE`, `metric{labels} value timestamp`
  - Целевые метрики: `zt_packet`, `zt_latency`, `zt_peer_status`, `zt_network_status`, `zt_packet_error`
- `cache.rs` — хранение последнего снимка метрик в памяти (`Arc<RwLock<MetricsSnapshot>>`)
- Обновление по расписанию (интервал из конфига, default 5s)

### REST API
```
GET /api/metrics          → MetricsSnapshot (JSON)
GET /api/metrics/raw      → raw Prometheus text (проксируется as-is)
```

### Критерии готовности
- Парсинг метрик работает нативно (без curl, без shell)
- Данные обновляются в фоне

---

## [6] `feat/backend-exitnode`

**Цель:** Настройка Exit Node (VPN выходная нода).

### Модуль `src/exitnode/`
- `platform.rs` — детекция ОС и поддержки:
  - Поддерживается: Linux (проверка через `cfg!(target_os = "linux")`)
  - Возвращает `PlatformSupport { supported: bool, reason: Option<String> }`
- `deps.rs` — проверка и установка зависимостей:
  - Определить наличие `iptables` / `nftables` (приоритет nftables на современных системах)
  - Проверка root/sudo прав (`nix::unistd::getuid()`)
  - Установка через системный пакетный менеджер нативно (без shell exec curl)
- `interfaces.rs` — список WAN-интерфейсов через `/proc/net/dev` (Linux) или `getifaddrs` (macOS)
- `rules.rs` — применение/снятие правил маскарада:
  ```rust
  // iptables через nix/libc syscalls или subprocess с явным PATH
  // nftables — через nft NFT ruleset управление
  fn enable_masquerade(zt_iface: &str, wan_iface: &str) -> Result<()>
  fn disable_masquerade(zt_iface: &str, wan_iface: &str) -> Result<()>
  fn enable_ip_forwarding() -> Result<()>   // /proc/sys/net/ipv4/ip_forward
  ```
- `state.rs` — `ExitNodeState { enabled, zt_network_id, wan_interface }`

### REST API
```
GET  /api/exitnode/platform     → { supported, reason }
GET  /api/exitnode/deps         → { iptables, nftables, root_access }
GET  /api/exitnode/interfaces   → [{ name, addresses }]
GET  /api/exitnode/status       → ExitNodeState
POST /api/exitnode/enable       → { zt_network_id, wan_interface }
POST /api/exitnode/disable
```

### Критерии готовности
- Нативная работа без curl
- Root-проверка до выполнения операций

---

## [7] `feat/frontend-build`

**Цель:** Build pipeline: несколько HTML/CSS/JS файлов → один `www/index.html`, встраивается в Rust бинарник.

### Структура исходников
```
www/src/
├── html/
│   ├── base.html         # layout shell
│   ├── dashboard.html
│   ├── networks.html
│   ├── network-detail.html
│   ├── controllers.html
│   ├── members.html
│   ├── configuration.html
│   ├── exitnode.html
│   └── settings.html
├── css/
│   ├── reset.css
│   ├── variables.css     # CSS custom properties
│   ├── layout.css
│   ├── components.css
│   └── pages.css
└── js/
    ├── api.js            # fetch-обёртка над REST API
    ├── router.js         # клиентский роутер (hash-based)
    ├── state.js          # глобальное состояние (без фреймворков)
    ├── components/
    │   ├── snippets.js   # выпадающие сниппеты сетей
    │   ├── qrcode.js     # QR генератор (нативный canvas)
    │   └── toast.js      # уведомления
    └── pages/
        ├── dashboard.js
        ├── networks.js
        ├── network-detail.js
        ├── controllers.js
        ├── members.js
        ├── exitnode.js
        └── settings.js
```

### Build script (`build.rs`)
- Конкатенация и минификация CSS → inline `<style>`
- Конкатенация JS → inline `<script>`
- Inline всех HTML-шаблонов как JS-строки
- Результат: `www/build/index.html` — один самодостаточный файл
- Встраивание в бинарник: `include_str!("../www/build/index.html")`
- В `server/router.rs`: `GET /` → отдаёт встроенный HTML

### Критерии готовности
- `cargo build` автоматически перебилдит фронт при изменениях в `www/src/`
- `GET /` возвращает валидный HTML без внешних зависимостей

---

## [8] `feat/frontend-dashboard`

**Цель:** Страница Dashboard — метрики ZeroTier, статус ноды, пиры.

### Компоненты
- **Node Info** — `node_id`, `version`, `public_ip`, `online` статус
- **Metrics Cards** — `zt_packet` (rx/tx), `zt_latency`, `zt_packet_error` — данные с `/api/metrics`
- **Peers Table** — `address`, `latency`, `paths`, `version`, `role` — данные с `/api/local/peers`
  - Колонки: Node ID | Latency | Direct | Last Seen | Role
  - Авто-обновление каждые 10s

### JS (`pages/dashboard.js`)
- Polling через `setInterval` (интервал конфигурируется из мета-тега)
- Нет сторонних библиотек (нативный fetch, DOM API)

### Критерии готовности
- Метрики отображаются и обновляются
- Таблица пиров работает

---

## [9] `feat/frontend-networks`

**Цель:** My Networks + Details & Node Configuration.

### My Networks
- Таблица: `type` (own/official badge) | `network_id` | `name` | `description` | `subnet` | `nodes` | `created`
- Фильтр: all / own / official
- Действия: **Join** (ввод ID) / **Delete** / **Toggle** (activate/deactivate) / **Details**
- Данные: объединение `/api/local/networks` + `/api/central/networks`

### Details & Node Configuration
- Сниппет выбора сети (только когда пришли из списка)
- **Tab: Details**
  - Network ID, Name, Status, Type, MAC, MTU, Broadcast, Bridging
  - Managed IPs, DNS (Search Domain, Server Addresses)
  - QR Code: нативная генерация через Canvas API (в `components/qrcode.js`)
- **Tab: Configuration**
  - Toggle: Route all traffic
  - DNS: radio (No DNS / Network DNS / Custom DNS)
  - При Custom DNS: формы IPv4 (×2) + IPv6 (×2) с валидацией

### JS (`pages/networks.js`, `pages/network-detail.js`)
- Валидация IP через нативный regex без библиотек
- Сохранение `POST /api/local/networks/:id`

### Критерии готовности
- QR-код генерируется нативно
- DNS конфигурация сохраняется

---

## [10] `feat/frontend-controllers`

**Цель:** Networks + Members + Configuration (управление контроллерами).

### Networks
- Таблица с фильтром [all / own / official]
- Колонки: type | network_id | name | description | subnet | nodes | created
- Действия: **Add** (выбор типа контроллера) / **Delete** / **Edit config**

### Members
- Таблица с фильтром [all / network name]
- Колонки: Auth | Address | Name/Desc | Managed IPs | Last Seen | Version | Physical IP | OS Arch
- Действия: **Edit** / **Auth** / **De-auth**
- **Edit Member Panel:**
  - Toggle: Authorized
  - Input: Name
  - Textarea: Description
  - Input + validation: IP Assignments (CIDR валидация)
  - Spoiler «Advanced»: Exclude from SSO / Allow Ethernet Bridging / No Auto-Assign IPs
  - Details: MAC, Last Seen, Client Version, Physical Address
  - Кнопки: Hide Member / Delete Member

### Configuration (Network Editor)
- Сниппет выбора типа контроллера (own/official)
- **Basics:** Network ID (readonly), Name, Description
- **Access Control:** Private / Public (radio, Public только для official)
- **Advanced — Managed Routes:**
  - Список маршрутов с удалением
  - Форма: Destination + Via с валидацией CIDR
- **IPv4 Auto-Assign:**
  - Toggle → Tabs: Easy / Advanced
  - Easy: сетка шаблонов (24 варианта из спецификации)
  - Advanced: пулы с диапазонами
- **IPv6 Auto-Assign:**
  - RFC4193 toggle + отображение prefix
  - 6PLANE toggle + отображение prefix
  - Auto-Assign Range: таблица + форма
- **Multicast:** Recipient Limit + Broadcast toggle
- **DNS:** Search Domain + Server Addresses
- **Manually Add Member:** Node ID input
- **Flow Rules:** textarea
- **Delete Network:** с подтверждением

### Критерии готовности
- Все поля конфига работают с real API
- Валидация IP/CIDR нативная

---

## [11] `feat/frontend-exitnode`

**Цель:** Страница Exit Node.

### Логика страницы
1. `GET /api/exitnode/platform` → если не поддерживается — показать banner с причиной, остановить
2. `GET /api/exitnode/deps` → показать статус зависимостей (iptables/nftables, root)
3. Если зависимостей нет — кнопка «Install & Configure»
4. Сниппет выбора сети (из подключённых/активных)
5. Select: WAN интерфейс (из `/api/exitnode/interfaces`)
6. Toggle: Enable/Disable Exit Node
7. Статус: текущее состояние

### Критерии готовности
- Платформо-независимая проверка
- Включение/выключение работает через real API

---

## [12] `feat/frontend-settings`

**Цель:** Global Settings + Root Servers.

### Global Settings
- Отображение и редактирование конфига (через `/api/config`):
  - Server host/port
  - ZT Local socket/token
  - ZT Central API token (masked input)
  - Metrics: enabled, URL, interval
- `PUT /api/config` — сохранение в `config.yml` (через конфиг-модуль)

### Root Servers
- Отображение текущих moon-серверов (`GET /api/local/peers` с `role: "MOON"`)
- Добавление moon: `POST /api/local/moon/:world_id` (zerotier-cli orbit)
- Удаление moon: `DEL /api/local/moon/:world_id`
- Документация ссылки на `https://docs.zerotier.com/roots/`

### REST API расширение
```
GET /api/config
PUT /api/config
GET /api/local/moons
POST /api/local/moons/:world_id
DEL  /api/local/moons/:world_id
```

### Критерии готовности
- Конфиг перезаписывается без перезапуска (hot-reload через `notify` crate или restart-signal)
- Root серверы управляются

---

## [13] `feat/integration`

**Цель:** Сквозное тестирование Frontend ↔ Backend, финальная полировка.

### Задачи
- E2E проверка всех API эндпоинтов (integration tests в `tests/`)
- Проверка CSP заголовков (Content-Security-Policy)
- CSRF защита (SameSite cookies или X-Request-Token header)
- Проверка error handling: каждый API error → понятный toast в UI
- Доступность: ARIA labels на ключевых компонентах
- Консистентность: единый стиль `ApiError` и обработка на фронтенде

### Критерии готовности
- Все сценарии из спецификации работают end-to-end
- Нет открытых XSS/CSRF векторов

---

## [14] `feat/packaging`

**Цель:** Cross-platform сборка, оптимизация бинарника, документация.

### Cargo profiles
```toml
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
strip = true
```

### Cross-compilation targets
- `x86_64-unknown-linux-gnu`
- `aarch64-unknown-linux-gnu`
- `x86_64-apple-darwin` / `aarch64-apple-darwin`
- `x86_64-pc-windows-msvc`

### GitHub Actions (`release.yml`)
- Триггер: push тега `v*`
- Матрица сборок через `cross` crate
- Upload артефактов в GitHub Releases

### Документация
- `README.md` — установка, запуск, конфигурация
- `CHANGELOG.md` — обновляется автоматически из git log

### Критерии готовности
- `cargo build --release` → один бинарник без внешних зависимостей (кроме ZT daemon)
- Все платформы собираются в CI

---

## Release `v0.1.0`

- Merge `feat/packaging` → `main`
- Git tag `v0.1.0`
- GitHub Release с бинарниками для всех платформ
- Закрытие всех issue milestone `v0.1.0`

---

## Принципы (non-negotiable)

| Правило | Применение |
|---|---|
| Без хардкода | Все URL, токены, пути — из конфига/env |
| Без заглушек | Только реальная логика в production-коде |
| Без внешних CLI | Всё нативно на Rust или нативном JS |
| Без дублирования | DRY на всех уровнях |
| Без мёртвого кода | `cargo clippy -- -D warnings` обязателен |
| Без unsafe | Только если нет альтернативы — с явным обоснованием |
