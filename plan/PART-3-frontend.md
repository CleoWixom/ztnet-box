# PART 3 — Frontend

> Ветки: `feat/part3-build-pipeline` → `feat/part3-ui-shell` → `feat/part3-ui-dashboard` → `feat/part3-ui-networks` → `feat/part3-ui-controllers` → `feat/part3-ui-exitnode` → `feat/part3-ui-settings`

---

## feat/part3-build-pipeline

**Цель:** `build.rs` собирает `www/src/**` → `www/build/index.html`, который встраивается в бинарник.

### Задачи

**Структура `www/src/`:**
```
www/src/
├── html/
│   ├── shell.html          # HTML shell с nav, sidebar, slot для страниц
│   ├── dashboard.html
│   ├── networks.html
│   ├── network-detail.html
│   ├── controllers-networks.html
│   ├── controllers-members.html
│   ├── controllers-config.html
│   ├── exitnode.html
│   ├── settings-global.html
│   └── settings-tokens.html
├── css/
│   ├── variables.css       # CSS custom properties (цвета, отступы, типография)
│   ├── reset.css           # minimal reset
│   ├── layout.css          # sidebar, main area, grid
│   ├── components.css      # кнопки, формы, таблицы, тоглы, снипеты, табы
│   └── pages.css           # page-specific стили
└── js/
    ├── api.js              # fetch-обёртка: все запросы к /api/*, error handling
    ├── router.js           # hash-роутер: #/dashboard, #/networks, ...
    ├── state.js            # глобальное in-memory состояние (нет localStorage)
    ├── components/
    │   ├── toast.js        # всплывающие уведомления
    │   ├── modal.js        # confirm/dialog
    │   ├── qrcode.js       # QR генерация через Canvas API (нативно, без либ)
    │   ├── snippets.js     # компонент выбора сети из снипетов
    │   └── table.js        # переиспользуемая таблица с фильтром
    └── pages/
        ├── dashboard.js
        ├── networks.js
        ├── network-detail.js
        ├── controllers-networks.js
        ├── controllers-members.js
        ├── controllers-config.js
        ├── exitnode.js
        ├── settings-global.js
        └── settings-tokens.js
```

**`build.rs`:**
- [ ] Читает все `.css` файлы в порядке: `variables` → `reset` → `layout` → `components` → `pages`
- [ ] Читает все `.js` файлы: сначала `components/`, затем `pages/`, затем `api.js`, `router.js`, `state.js`
- [ ] Читает HTML шаблоны страниц как JS-строки → встраивает в `TEMPLATES` JS-объект
- [ ] Оборачивает CSS в `<style>`, JS в `<script>`
- [ ] Генерирует единый `www/build/index.html`
- [ ] `println!("cargo:rerun-if-changed=www/src")` — пересборка при изменении исходников

**`src/server/router.rs`** (дополнение):
- [ ] `const FRONTEND: &str = include_str!("../../www/build/index.html");`
- [ ] Handler: `GET /` и все non-API пути → `Html(FRONTEND)`

### Критерии готовности
- [ ] `cargo build` автоматически пересобирает фронтенд
- [ ] `GET /` возвращает валидный HTML без внешних ресурсов (нет CDN, нет шрифтов извне)
- [ ] Итоговый `index.html` < 500 KB (без минификации допустимо)

---

## feat/part3-ui-shell

**Цель:** общий layout — sidebar навигация, header, routing, toast/modal система.

### Задачи

**`www/src/html/shell.html`** — структура:
```html
<div id="app">
  <nav id="sidebar">
    <!-- logo, nav-items, version -->
  </nav>
  <main id="content">
    <!-- router outlet -->
  </main>
</div>
<div id="toast-container"></div>
<div id="modal-overlay"></div>
```

**Навигация** (согласно спецификации):
- [ ] **Nodes** (ярлык: Dashboard)
  - Dashboard
  - Peers
- [ ] **My Networks**
  - (список → Detail & Config)
- [ ] **Controllers**
  - Networks
  - Members
  - Configuration
- [ ] **Settings**
  - Global
  - Root Servers
  - API Tokens ← **новый пункт**
- [ ] Активный пункт подсвечивается, раскрытие секций

**`www/src/js/router.js`**:
- [ ] Hash-роутер: `#/dashboard`, `#/peers`, `#/networks`, `#/networks/:id`, `#/networks/:id/config`, `#/controllers/networks`, `#/controllers/members`, `#/controllers/config`, `#/exitnode`, `#/settings/global`, `#/settings/roots`, `#/settings/tokens`
- [ ] `Router.navigate(path)`, `Router.onRoute(pattern, handler)`
- [ ] При навигации: рендер шаблона → вызов page-инициализатора → обновление активного nav-item

**`www/src/js/api.js`**:
- [ ] `api.get(path)`, `api.post(path, body)`, `api.put(path, body)`, `api.delete(path)` — все возвращают Promise
- [ ] При `4xx/5xx` — пробрасывают `{ error, code }` из JSON тела
- [ ] Нет глобального axios/fetch-полифила — только нативный `fetch`

**`www/src/js/state.js`**:
- [ ] `State { networks: [], peers: [], metrics: null, activeNetworkId: null, tokens: [] }`
- [ ] `State.set(key, value)` / `State.get(key)` — простое in-memory хранилище
- [ ] Нет localStorage (не нужен для локального UI без сессий)

**`www/src/js/components/toast.js`**:
- [ ] `Toast.success(msg)`, `Toast.error(msg)`, `Toast.info(msg)` — 4s автоудаление
- [ ] Стек тостов, анимация slide-in/out

**`www/src/js/components/modal.js`**:
- [ ] `Modal.confirm(message) -> Promise<boolean>` — для деструктивных действий
- [ ] `Modal.prompt(message, placeholder) -> Promise<string|null>`

### Критерии готовности
- [ ] Навигация работает без перезагрузки страницы
- [ ] Toast отображается при API-ошибках
- [ ] Confirm modal показывается перед удалением

---

## feat/part3-ui-dashboard

**Цель:** страница Dashboard — статус ноды, метрики, таблица пиров.

### Задачи

**Секция: Node Status** (`GET /api/local/status`):
- [ ] Node ID (моноширинный), Online badge (зелёный/красный), Version, World ID
- [ ] Public IP (если доступен)

**Секция: Metrics Cards** (`GET /api/metrics`, polling каждые 5s):
- [ ] Card: **Traffic** — RX/TX bytes в человекочитаемом формате (KB/MB/GB)
- [ ] Card: **Packets** — RX/TX packets
- [ ] Card: **Avg Latency** — ms, цветовая индикация (зелёный < 50ms, жёлтый < 150ms, красный)
- [ ] Card: **Errors** — total packet errors
- [ ] Индикатор последнего обновления метрик (`last_updated`)
- [ ] При `metrics.enabled = false` — информационный баннер

**Секция: Peers** (`GET /api/local/peers`):
- [ ] Таблица: Node ID | Role | Latency | Direct paths | Last Unicast | Version
- [ ] Role badge: LEAF / PLANET / MOON с разными цветами
- [ ] Latency cell: цветовая индикация
- [ ] Авто-обновление каждые 10s (через `State`)
- [ ] Клик по Node ID → `#/peers/:node_id` (детальный view)

**`www/src/js/pages/dashboard.js`**:
- [ ] `init()` — запускает polling, рендерит все секции
- [ ] `destroy()` — останавливает polling при смене страницы (clearInterval)
- [ ] Данные из State кэшируются — первый рендер мгновенный если данные уже есть

### Критерии готовности
- [ ] Polling корректно останавливается при уходе со страницы
- [ ] Метрики отображаются если ZT запущен

---

## feat/part3-ui-networks

**Цель:** My Networks, Details, Node Configuration.

### Задачи

**My Networks** (`GET /api/local/networks` + `GET /api/central/networks` если есть активный токен):

- [ ] Объединение списков: local (тип "own") + central (тип "official")
- [ ] Таблица: Type badge | Network ID | Name | Description | Subnet | Nodes | Created
- [ ] Фильтр tabs: All | Own | Official
- [ ] Действия:
  - **Join** — input для ввода Network ID, кнопка → `POST /api/local/networks/:id`
  - **Toggle** — activate/deactivate → `POST /api/local/networks/:id` с `{ allow_managed }`
  - **Details** → navigate `#/networks/:id`
  - **Delete** — confirm modal → `DEL /api/local/networks/:id`

**Network Detail** (`GET /api/local/networks/:id`):

- [ ] Сниппет сети в заголовке: Network ID + Name
- [ ] **Tab: Details**
  - [ ] Network ID, Name, Status (Connected / Not Connected badge)
  - [ ] Type, MAC, MTU, Broadcast enabled
  - [ ] Bridging enabled/disabled
  - [ ] Managed IPs: список assignedAddresses
  - [ ] DNS: Search Domain + Server Addresses (или "Network DNS is not configured")
  - [ ] QR Code: генерация через `qrcode.js` из Network ID (canvas → data URL → img)
  - [ ] Кнопка «Copy Network ID»

- [ ] **Tab: Configuration**
  - [ ] Toggle: "Route all traffic through ZeroTier" (`allow_default`) — предупреждение что требует внешней настройки
  - [ ] DNS Radio group:
    - `No DNS` — DNS не настраивается (default)
    - `Network DNS` — использует dns из network config
    - `Custom DNS` — показывает формы:
      - IPv4 DNS ×2: input с валидацией IPv4
      - IPv6 DNS ×2: input с валидацией IPv6
  - [ ] Сохранение: `POST /api/local/networks/:id`

**`www/src/js/components/qrcode.js`** — QR генератор:
- [ ] Нативная реализация QR matrix через Canvas API (алгоритм Reed-Solomon)
- [ ] `QRCode.render(text, canvasEl, { size, errorCorrection })`
- [ ] Только для Network ID (короткие строки) — достаточно Error Correction Level M

### Критерии готовности
- [ ] QR генерируется без внешних библиотек
- [ ] Custom DNS валидируется до отправки
- [ ] Toggle активации сети работает

---

## feat/part3-ui-controllers

**Цель:** Controllers → Networks, Members, Configuration.

### Networks

`GET /api/local/controller/networks` + `GET /api/central/networks`:
- [ ] Таблица с фильтром [All | Own | Official]
- [ ] Столбцы: Type | Network ID | Name | Description | Subnet | Members count | Created
- [ ] Действия: **Add** (→ Configuration page с пустой формой) / **Edit** (→ Configuration) / **Delete** (confirm)
- [ ] При Add: сначала modal выбора типа контроллера (Own / Official)
  - Own: `POST /api/local/controller/networks` (генерирует random ID)
  - Official: `POST /api/central/networks`

### Members

`GET /api/local/controller/networks/:id/members` (own) или `GET /api/central/networks/:id/members`:
- [ ] Фильтр: All | [по имени сети]
- [ ] Таблица: Auth checkbox | Address | Name/Desc | Managed IPs | Last Seen | Version | Physical IP | OS/Arch
- [ ] Auth badge: зелёный (authorized) / серый (unauthorized)
- [ ] Last Seen: человекочитаемый формат ("2 min ago", "yesterday")

**Edit Member panel** (slide-in или modal):
- [ ] Toggle: Authorized → `PUT /api/.../members/:node_id { authorized }`
- [ ] Input: Name
- [ ] Textarea: Description
- [ ] IP Assignments: список + добавление с CIDR валидацией (regex нативный)
- [ ] Spoiler «Advanced»:
  - Toggle: Exclude from SSO (`sso_exempt`)
  - Toggle: Allow Ethernet Bridging (`active_bridge`)
  - Toggle: Do Not Auto-Assign IPs (`no_auto_assign_ips`)
- [ ] Details секция (readonly): MAC, Last Seen, Client Version, Physical Address
- [ ] **Hide Member** — деавторизует + скрывает (confirm)
- [ ] **Delete Member** — confirm с предупреждением

### Configuration (Network Editor)

Открывается из: Networks → Add/Edit, или из Members → выбор сети:

- [ ] **Basics**
  - Network ID (readonly)
  - Name input
  - Description textarea

- [ ] **Access Control**
  - Radio: Private | Public
  - Public — только для Official контроллера (для Own — disabled с подсказкой)

- [ ] **Advanced — Managed Routes**
  - Список маршрутов: `[trash] destination via gateway`
  - LAN маршруты без via
  - Форма добавления: Destination (CIDR) + Via (IP, опционально)
  - Валидация CIDR нативным regex

- [ ] **IPv4 Auto-Assign** (Toggle)
  - Tabs: Easy | Advanced
  - Easy: сетка шаблонов (24 варианта в два столбца):
    ```
    10.147.17.*  10.147.18.*  10.147.19.*  10.147.20.*
    10.144.*.*   10.241.*.*   10.242.*.*   10.243.*.*
    10.244.*.*   172.22.*.*   172.23.*.*   172.24.*.*
    172.25.*.*   172.26.*.*   172.27.*.*   172.28.*.*
    172.29.*.*   172.30.*.*   192.168.191.* 192.168.192.*
    192.168.11.* 192.168.22.* 192.168.33.* 192.168.66.*
    ```
  - Advanced: таблица пулов (Start–End) + форма добавления

- [ ] **IPv6 Auto-Assign**
  - Toggle RFC4193: при включении показывать вычисленный prefix `fd??:????:????:????:????::/80`
  - Toggle 6PLANE: при включении показывать prefix `fc??:????:??::__:0:0:1`
  - Toggle Auto-Assign Range: таблица диапазонов + форма

- [ ] **Multicast**
  - Input int: Multicast Recipient Limit
  - Toggle: Enable Broadcast

- [ ] **DNS** (сервера для сети)
  - Input: Search Domain
  - Input + validation: Server Address → кнопка Add
  - Список серверов с удалением
  - Кнопка «Clear DNS config»

- [ ] **Manually Add Member**
  - Input: Node ID (10 hex символов)
  - Кнопка Add → `PUT /api/.../members/:node_id { authorized: false }`

- [ ] **Flow Rules**
  - Textarea (свободный ввод, ZeroTier rule syntax)
  - Без валидации на фронте

- [ ] **Delete Network** — красная кнопка внизу, confirm modal

### Критерии готовности
- [ ] Создание сети работает для обоих типов контроллеров
- [ ] IPv6 prefix вычисляется корректно из Network ID (нативный JS)
- [ ] Все поля сохраняются через PUT/POST

---

## feat/part3-ui-exitnode

**Цель:** страница Exit Node с проверками платформы, зависимостей, управлением.

### Задачи

- [ ] **Шаг 1 — Platform check** `GET /api/exitnode/platform`:
  - Если `supported: false` → banner "Platform not supported: {reason}", остальное скрыто
  - Если `supported: true` → переход к шагу 2

- [ ] **Шаг 2 — Deps check** `GET /api/exitnode/deps`:
  - Checklist: ✅/❌ iptables | ✅/❌ nftables | ✅/❌ root access
  - Если что-то отсутствует: кнопка «Install missing dependencies» → `POST /api/exitnode/deps/install`
  - root access отсутствует → предупреждение что нужно перезапустить с sudo, кнопка недоступна

- [ ] **Сниппет выбора сети** — только подключённые и активные сети (`GET /api/local/networks`, фильтр `status: OK`)

- [ ] **Select WAN interface** `GET /api/exitnode/interfaces`:
  - Dropdown с именами интерфейсов (исключить ZeroTier интерфейсы из WAN-списка)
  - Показывать IP адреса рядом с именем

- [ ] **Toggle: Enable Exit Node**
  - Enable → `POST /api/exitnode/enable { zt_network_id, wan_interface }`
  - Disable → `POST /api/exitnode/disable`
  - При включении: confirmation с предупреждением о влиянии на трафик

- [ ] **Status block** `GET /api/exitnode/status`:
  - Enabled/Disabled badge
  - Активная сеть и WAN интерфейс
  - Используемый firewall backend (nftables/iptables)
  - Время включения

### Критерии готовности
- [ ] Невозможно включить без выбора сети и WAN интерфейса
- [ ] Состояние корректно отражается после перезагрузки страницы

---

## feat/part3-ui-settings

**Цель:** Settings → Global + Root Servers + API Tokens.

### Global Settings (`#/settings/global`)

`GET /api/settings/config`:
- [ ] **Server**
  - Host input (предупреждение: изменение требует перезапуска)
  - Port input (1–65535)
- [ ] **ZeroTier Local**
  - API URL
  - Token file path
- [ ] **Metrics**
  - Toggle: enabled
  - Prometheus URL (только если enabled)
  - Poll interval (seconds, 1–60)
- [ ] **Exit Node**
  - Toggle: nftables preferred
- [ ] Кнопка Save → `PUT /api/settings/config`
- [ ] Toast: "Config saved. Restart required for server settings." если изменился host/port

### Root Servers (`#/settings/roots`)

`GET /api/local/moons`:
- [ ] Таблица moon-серверов: World ID | Timestamp | Roots (список адресов)
- [ ] **Add Moon**: input World ID + Seed ID → `POST /api/local/moons/:world_id`
- [ ] **Remove Moon**: confirm → `DEL /api/local/moons/:world_id`
- [ ] Ссылка на документацию (внешняя в новой вкладке)
- [ ] Статус: отображать MOON пиры из `GET /api/local/peers` (role: "MOON") рядом

### API Tokens (`#/settings/tokens`)

`GET /api/settings/tokens`:
- [ ] Список карточек токенов: Name | Masked token | Rate limit badge | Created | Active badge
- [ ] Кнопка «+ Add Token» → inline форма:
  - Input: Name (обязательный)
  - Input: Token (password type, обязательный)
  - Кнопка «Verify» → `POST /api/settings/tokens/validate` — показывает результат (аккаунт инфо) без добавления
  - Кнопка «Add» → `POST /api/settings/tokens` (только после успешной верификации)
- [ ] Действия на карточке:
  - **Set Active** → `POST /api/settings/tokens/:id/activate` — помечает активным
  - **Edit Name** — inline редактирование → `PUT /api/settings/tokens/:id`
  - **Delete** — confirm modal → `DEL /api/settings/tokens/:id`
- [ ] Если нет токенов: призыв добавить с объяснением зачем (Central API недоступен без токена)
- [ ] Если нет активного токена: предупреждение-баннер
- [ ] Account status активного токена: Plan, Email — `GET /api/central/status`

### Критерии готовности
- [ ] Токен валидируется до добавления
- [ ] Активный токен подсвечен визуально
- [ ] Реальный токен нигде не виден в UI (только маска)
