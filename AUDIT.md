# AUDIT.md — ztnet-box v0.9.1

**Дата аудита:** 2026-04-18  
**Репозиторий:** `CleoWixom/ztnet-box`  
**Стек:** Rust (Axum 0.7, Tokio) + Vanilla JS / HTML / CSS (SPA, сборка через `build.rs`)  
**Версия на момент аудита:** 0.9.1  
**Аудитор:** Автоматизированный архитектурный аудит  

---

## Содержание

1. [Сводная таблица](#сводная-таблица)  
2. [CRITICAL — Ошибки](#critical)  
3. [HIGH — Высокий приоритет](#high)  
4. [MEDIUM — Средний приоритет](#medium)  
5. [LOW — Низкий приоритет](#low)  
6. [Соответствие Frontend ↔ Backend API](#соответствие-frontend--backend-api)  
7. [Общие архитектурные рекомендации](#общие-архитектурные-рекомендации)  

---

## Сводная таблица

| # | Приоритет | Категория | Файл(ы) | Проблема |
|---|-----------|-----------|---------|----------|
| 1 | 🔴 CRITICAL | Bug | `dashboard.js` | `refresh()` недоступна из `_installZt()` — ReferenceError в рантайме |
| 2 | 🔴 CRITICAL | Bug / API | `exitnode.js` | POST `/exitnode/deps/install` → 404; backend слушает POST `/exitnode/deps` |
| 3 | 🔴 CRITICAL | Bug / Semantic | `exitnode.js` | `zt_interface` получает **network ID** вместо имени интерфейса |
| 4 | 🔴 CRITICAL | Security / XSS | `toast.js`, multiple pages | Серверные строки вставляются в `innerHTML` без экранирования |
| 5 | 🟠 HIGH | Memory leak | `tokens.rs` | Каждый `probe_token` создаёт `tokio::spawn` для rate-limiter, который никогда не завершается |
| 6 | 🟠 HIGH | Security | `router.rs` | CSP содержит `'unsafe-inline'` для script-src и style-src |
| 7 | 🟠 HIGH | Security | `router.rs` | CORS `allow_headers(Any)` — принимаются любые заголовки |
| 8 | 🟠 HIGH | Dead code | `table.js` | Компонент `Table` определён, но **нигде не используется** |
| 9 | 🟠 HIGH | Bug / UI | `shell.html` | Пункт «Members» в сайдбаре ведёт на `/controllers/networks`, а не на `/controllers/members` |
| 10 | 🟡 MEDIUM | Performance | `local.rs` | `ZtLocalClient` (+ `reqwest::Client`) создаётся заново на каждый HTTP-запрос |
| 11 | 🟡 MEDIUM | Duplication | `config.rs`, `tokens.rs` | Два независимых `struct TokenView` с разными полями |
| 12 | 🟡 MEDIUM | Duplication | `bridge.rs`, `exitnode.rs`, `physnet.rs`, `ndp.rs` | Четыре `struct EnableRequest` в разных файлах |
| 13 | 🟡 MEDIUM | Duplication / JS | `dashboard.js`, `peers.js` | `latencyClass()` — точная копия в двух файлах |
| 14 | 🟡 MEDIUM | API gap | `router.rs` | Несколько backend-эндпоинтов не имеют соответствующих страниц во frontend |
| 15 | 🟡 MEDIUM | Bug / JS | `toast.js` | `Toast.warn()` отсутствует; 5 мест вызывают несуществующий метод |
| 16 | 🟡 MEDIUM | Security | `deps.rs` | `std::env::set_var` в тестах — data race в многопоточной среде (rustc ≥ 1.80) |
| 17 | 🟡 MEDIUM | Security / External | `ssh.rs`, `deploy.rs` | Зависимость от системного бинарника `ssh`; path traversal через `key_path` |
| 18 | 🟡 MEDIUM | Hardcode | `router.rs` | Hardcode `localhost` в проверке CORS origin; не покрывает IPv6 `[::1]` |
| 19 | 🟡 MEDIUM | UX / API | `exitnode.js` | Предупреждения от backend (`res.warnings`) выводятся через `Toast.warn()` (несуществующий) |
| 20 | 🟡 MEDIUM | Architecture | `build.rs` | Нет минификации/бандлинга; весь JS работает в глобальном scope |
| 21 | 🟢 LOW | Duplication / JS | `controllers-config.js` | `IP_POOLS` — hardcode 24 пулов; изменение требует правки кода |
| 22 | 🟢 LOW | Unused backend | `local_config.rs` | `GET/PUT /local/networks/:id/localconf` реализованы, но не используются во frontend |
| 23 | 🟢 LOW | Unused backend | `handlers/` | `GET /metrics/raw`, `GET /central/user`, `GET /central/status` — нет страниц во frontend |
| 24 | 🟢 LOW | Unused backend | `tokens.rs` | `PUT /settings/tokens/:id` (`update_token`) — нет UI для редактирования токена |
| 25 | 🟢 LOW | Minor / Rust | `deps.rs` | Блокирующие `Command::new` вызываются из синхронной функции `ensure()`, что нормально в данном контексте, но документально не объяснено |
| 26 | 🟢 LOW | Metrics default | `schema.rs` | `MetricsConfig::default()` включает `enabled: true`; на системах без ZT-метрик будут повторяющиеся ошибки в логах |

---

## CRITICAL

---

### #1 — `refresh()` недоступна из `_installZt()` — ReferenceError в рантайме

**Приоритет:** 🔴 CRITICAL  
**Категория:** Bug  
**Файл:** `www/src/js/pages/dashboard.js`, строки 95–163  

**Описание:**  
Функция `refresh()` объявлена **внутри** функции `render()`. Метод `_installZt()` определён в возвращаемом объекте публичного API (вне `render()`), и при вызове `setTimeout(() => refresh(), 1500)` переменная `refresh` недоступна в этом scope — браузер выдаёт `ReferenceError: refresh is not defined`.

**Проблемный код:**
```js
// dashboard.js
function render() {
  // ...
  async function refresh() { /* ... */ }  // локальная функция

  refresh();
  _intervals.push(setInterval(refresh, 10000));
}

return {
  init() { render(); },
  async _installZt(btn) {
    // ...
    setTimeout(() => refresh(), 1500); // ❌ refresh не в scope — ReferenceError
  },
};
```

**Почему проблема:**  
После успешной установки ZeroTier статус дашборда не обновляется. Пользователь видит «Installing…» до следующей перезагрузки страницы. Ошибка скрыта в `catch(e)`, поэтому визуально молчащая.

**Рекомендация:**  
Вынести `refresh` на уровень модуля IIFE или передавать как параметр:

```js
const DashboardPage = (() => {
  let _intervals = [];
  let _refresh = null; // ссылка на текущую функцию refresh

  // ...
  function render() {
    async function refresh() { /* ... */ }
    _refresh = refresh; // сохраняем ссылку
    refresh();
    _intervals.push(setInterval(refresh, 10000));
  }

  return {
    init() { render(); },
    async _installZt(btn) {
      // ...
      setTimeout(() => _refresh?.(), 1500); // ✅
    },
  };
})();
```

---

### #2 — POST `/exitnode/deps/install` → 404 (неверный путь)

**Приоритет:** 🔴 CRITICAL  
**Категория:** Bug / API мismatch  
**Файлы:** `www/src/js/pages/exitnode.js:160`, `src/server/router.rs:127`  

**Описание:**  
Frontend вызывает `api.post('/exitnode/deps/install')`, но в роутере этот маршрут **не зарегистрирован**. Бэкенд регистрирует install handler на `POST /exitnode/deps` (через `.post(exit_handler::install_deps)` на route `/deps`).

**Проблемный код:**
```js
// exitnode.js — НЕВЕРНО
await api.post('/exitnode/deps/install'); // ❌ → 404

// router.rs — реальный маршрут
.route("/deps", get(exit_handler::get_deps).post(exit_handler::install_deps))
// маршрут /deps/install для exitnode НЕ СУЩЕСТВУЕТ
```

Для сравнения, bridge настроен правильно:
```rust
// router.rs — bridge
.route("/deps", get(bridge_handler::get_deps))
.route("/deps/install", post(bridge_handler::install_deps)) // ✅ отдельный маршрут
```

**Почему проблема:**  
Кнопка «Install missing» на странице Exit Node никогда не работает — всегда возвращает 404. Пользователь не может установить iptables/nftables через UI.

**Рекомендация:**  
Выбрать один из вариантов и применить единообразно:

*Вариант A — исправить frontend (минимальное изменение):*
```js
// exitnode.js
await api.post('/exitnode/deps'); // ✅ совпадает с реальным роутером
```

*Вариант B — добавить отдельный маршрут в роутер (консистентно с bridge):*
```rust
// router.rs
.route("/deps", get(exit_handler::get_deps))
.route("/deps/install", post(exit_handler::install_deps)) // ✅
```

---

### #3 — `zt_interface` получает Network ID вместо имени интерфейса

**Приоритет:** 🔴 CRITICAL  
**Категория:** Bug / Semantic  
**Файл:** `www/src/js/pages/exitnode.js:163–176`  

**Описание:**  
В методе `_enable()` переменная `zt` содержит значение из `#en-net` (select с **ZeroTier Network ID**, например `8056c2e21c000001`). Это значение отправляется одновременно в два поля: `zt_interface: zt` и `network_id: zt`.  

Бэкенд (`exitnode/mod.rs`) явно документирует: `zt_interface` — это **имя сетевого интерфейса** (например `ztabcd1234e`), а не 16-символьный network ID.

**Проблемный код:**
```js
async _enable() {
  const zt  = document.getElementById('en-net')?.value;  // = "8056c2e21c000001"
  // ...
  await api.post('/exitnode/enable', {
    zt_interface: zt,   // ❌ Network ID вместо interface name
    wan_interface: wan,
    network_id: zt,     // ✓ корректно
    // ...
  });
}
```

**Бэкенд (exitnode/rules.rs):**
```rust
// ExitNodeRules использует zt_interface в iptables-командах:
"-o", &self.zt_iface,  // передаётся системе как имя интерфейса
```

**Почему проблема:**  
`iptables -o 8056c2e21c000001` завершится ошибкой «No such device». Exit Node не будет работать никогда, пока этот баг не исправлен. На странице **отсутствует** селектор ZT-интерфейса.

**Рекомендация:**  
Добавить отдельный select для выбора ZT-интерфейса (аналогично страницам Bridge и PhysNet):

```js
// exitnode.js — в render()
const ztIfaces = (ifaces||[]).filter(i => i.is_zerotier);
const ztOpts = ztIfaces.map(i => `<option value="${i.name}">${i.name}</option>`).join('');
// ...
`<div class="field">
  <label class="field-label">ZeroTier Interface</label>
  <select class="select" id="en-zt">${ztOpts}</select>
</div>`

// В _enable()
const ztIface = document.getElementById('en-zt')?.value; // имя интерфейса
const netId   = document.getElementById('en-net')?.value; // network ID
await api.post('/exitnode/enable', {
  zt_interface: ztIface,  // ✅
  network_id:   netId,    // ✅
  // ...
});
```

---

### #4 — XSS: серверные строки вставляются в `innerHTML` без экранирования

**Приоритет:** 🔴 CRITICAL  
**Категория:** Security / XSS  
**Файлы:** `toast.js:7`, `network-detail.js:84`, `settings-tokens.js:79`, `settings-global.js:9`  

**Описание:**  
В нескольких местах данные, пришедшие с сервера, вставляются напрямую в `innerHTML` без санитизации. Хотя приложение работает на localhost, вредоносные данные в полях ZeroTier (например, имя сети, содержащее `<script>`) могут привести к self-XSS или XSS через скомпрометированный ZeroTier API.

**Проблемные места:**

```js
// toast.js:7 — msg вставляется напрямую в innerHTML
el.innerHTML = `<span class="toast-msg">${msg}</span>...`; // ❌

// network-detail.js:84 — e.message из сервера
document.getElementById('content').innerHTML =
  `<div class="banner">❌ ${e.message}</div>`; // ❌

// settings-tokens.js:79 — res.error из сервера
document.getElementById('verify-result').innerHTML =
  `<div class="banner">❌ ${res.error||'Invalid token'}</div>`; // ❌

// settings-global.js:9 — e.message из сервера
document.getElementById('content').innerHTML =
  `<div class="banner">❌ ${e.message}</div>`; // ❌
```

**Почему проблема:**  
Если ZeroTier Central API или локальный ZT daemon возвращает имена сетей/членов, содержащие HTML-теги, они будут выполнены браузером как разметка. Функция `Utils.esc()` уже существует в проекте, но применяется непоследовательно.

**Рекомендация:**  
Использовать `Utils.esc()` для всех данных из внешних источников и применить его в `Toast`:

```js
// toast.js — исправленный вариант
function show(msg, type) {
  const el = document.createElement('div');
  el.className = `toast toast-${type}`;
  // Использовать textContent вместо innerHTML для безопасной вставки
  const msgSpan = document.createElement('span');
  msgSpan.className = 'toast-msg';
  msgSpan.textContent = msg; // ✅ безопасно
  const closeSpan = document.createElement('span');
  closeSpan.className = 'toast-close';
  closeSpan.textContent = '✕';
  closeSpan.onclick = () => el.remove();
  el.append(msgSpan, closeSpan);
  container().appendChild(el);
  setTimeout(() => { el.classList.add('removing'); setTimeout(() => el.remove(), 200); }, 4000);
}
```

Для banners в страницах использовать `Utils.esc(e.message)` везде.

---

## HIGH

---

### #5 — Утечка `tokio::spawn` при каждой проверке токена

**Приоритет:** 🟠 HIGH  
**Категория:** Memory/Resource Leak  
**Файлы:** `src/server/handlers/tokens.rs:205–211`, `src/zerotier/central/client.rs:18–44`  

**Описание:**  
Каждый вызов `probe_token()` создаёт новый `ZtCentralClient`, который в конструкторе запускает `tokio::spawn` для фоновой задачи rate-limiter (пополнение семафора раз в секунду). Эта задача **никогда не завершается** — у неё нет cancellation token.

**Проблемный код:**
```rust
// tokens.rs
async fn probe_token(base_url: &str, token: &str) -> Result<AccountStatus, ApiError> {
    // ↓ создаёт новый Client → новый RateLimiter → новый tokio::spawn
    let client = ZtCentralClient::new(base_url.to_string(), token.to_string(), &RateLimit::Free);
    client.account_status().await
}
```

```rust
// client.rs — RateLimiter::new()
tokio::spawn(async move { // ← вечный цикл, никогда не завершается
    let mut interval = tokio::time::interval(Duration::from_secs(1));
    loop {
        interval.tick().await;
        // пополняет семафор каждую секунду
    }
});
```

**Почему проблема:**  
Каждый вызов «Verify» в Settings → Tokens и каждый `add_token` / `update_token` порождает новую вечную задачу. На долго работающем сервере с активным использованием токенов в памяти накапливаются тысячи мёртвых задач.

**Рекомендация:**  
Использовать `tokio_util::sync::CancellationToken` или хранить `JoinHandle` в `RateLimiter` и отменять при Drop:

```rust
pub struct RateLimiter {
    semaphore: Arc<Semaphore>,
    _task: tokio::task::JoinHandle<()>, // Drop отменяет задачу
}

impl Drop for RateLimiter {
    fn drop(&mut self) {
        self._task.abort();
    }
}
```

Или ещё проще — для `probe_token` использовать простой `reqwest::Client::new()` без rate-limiter, так как это разовая проверка:

```rust
async fn probe_token(base_url: &str, token: &str) -> Result<AccountStatus, ApiError> {
    // Используем минимальный клиент без rate-limiter для одиночного запроса
    let http = reqwest::Client::new();
    let resp = http.get(format!("{base_url}/self"))
        .bearer_auth(token)
        .send().await
        .map_err(|e| ApiError::ZtCentral(e.to_string()))?;
    // ...
}
```

---

### #6 — CSP разрешает `'unsafe-inline'` для скриптов и стилей

**Приоритет:** 🟠 HIGH  
**Категория:** Security  
**Файл:** `src/server/router.rs:190–197`  

**Описание:**  
Заголовок Content-Security-Policy содержит `script-src 'self' 'unsafe-inline'` и `style-src 'self' 'unsafe-inline'`. Директива `'unsafe-inline'` полностью нейтрализует XSS-защиту CSP для скриптов, делая заголовок бесполезным для своей основной цели.

**Проблемный код:**
```rust
// router.rs
"default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'; ..."
//                                    ^^^^^^^^^^^^^^^^                    ^^^^^^^^^^^^^^^^
```

**Почему проблема:**  
`'unsafe-inline'` позволяет выполнять любой inline-скрипт, включая те, которые могут попасть через XSS (#4). CSP становится декоративным заголовком, не обеспечивающим реальной защиты.

**Рекомендация:**  
Так как весь JS собирается в один файл через `build.rs` и встраивается в HTML как `<script>` блок, необходимо использовать nonce или hash:

```rust
// Генерировать nonce per-request и добавлять его в HTML и CSP
let nonce = generate_nonce(); // base64(random_bytes(16))
let csp = format!(
    "default-src 'self'; script-src 'self' 'nonce-{nonce}'; style-src 'self' 'nonce-{nonce}'; ...",
    nonce = nonce
);
// Заменять {{NONCE}} в index.html при сборке
```

Либо вынести все стили/скрипты в внешние файлы (`.js`, `.css`) с правильными хэшами в CSP.

---

### #7 — CORS `allow_headers(Any)` — избыточная широта

**Приоритет:** 🟠 HIGH  
**Категория:** Security  
**Файл:** `src/server/router.rs:33–36`  

**Описание:**  
CORS настроен с `allow_headers(tower_http::cors::Any)`, что означает принятие **любого** заголовка в preflight-запросе. Это может облегчить CSRF-атаки через нестандартные заголовки и нарушает принцип least-privilege.

**Проблемный код:**
```rust
let cors = CorsLayer::new()
    .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
    .allow_headers(tower_http::cors::Any)  // ❌ принимает ВСЕ заголовки
    .allow_origin([origin_host, origin_lo]);
```

**Рекомендация:**  
Явно перечислить только необходимые заголовки:

```rust
use axum::http::header::{CONTENT_TYPE, AUTHORIZATION};
let cors = CorsLayer::new()
    .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
    .allow_headers([CONTENT_TYPE]) // ✅ только то, что реально нужно
    .allow_origin([origin_host, origin_lo]);
```

---

### #8 — Компонент `Table` определён, но нигде не используется

**Приоритет:** 🟠 HIGH  
**Категория:** Dead Code  
**Файл:** `www/src/js/components/table.js`  

**Описание:**  
Файл `table.js` реализует универсальный компонент `Table.render()` для отрисовки фильтруемых таблиц. При поиске по всем JS-файлам страниц и компонентов обращений к `Table.` **не обнаружено**. Все страницы строят таблицы вручную через inline HTML-шаблоны, игнорируя этот компонент.

**Почему проблема:**  
Мёртвый код включается в финальный бандл, увеличивая его размер. Создаёт путаницу — разработчик не знает, следует ли использовать `Table` или писать inline. Несоответствие между архитектурным намерением и реальным использованием.

**Рекомендация:**  
Выбрать один из путей:
- Удалить `table.js`, если компонент не планируется использовать.
- Последовательно применить `Table.render()` хотя бы на трёх-четырёх страницах (peers, networks, controller networks, members), убрав дублирующийся inline-код таблиц.

---

### #9 — «Members» в сайдбаре ведёт на `/controllers/networks`

**Приоритет:** 🟠 HIGH  
**Категория:** Bug / UX  
**Файл:** `www/src/html/shell.html:49–51`  

**Описание:**  
В секции Controllers сайдбара два пункта: «Networks» и «Members». Оба имеют одинаковый `data-route` и `onclick`, ведущий на `/controllers/networks`. Пункт «Members» существует как отдельный UI-элемент, но не выполняет никакой уникальной функции навигации.

**Проблемный код:**
```html
<!-- shell.html -->
<div class="nav-item" data-route="/controllers/networks"
     onclick="Router.navigate('/controllers/networks')">Networks</div>

<div class="nav-item" data-route="/controllers/networks"  <!-- ❌ дублирует Networks -->
     onclick="Router.navigate('/controllers/networks')"
     title="Select a network to view members">Members</div>
```

**Почему проблема:**  
Пользователь ожидает, что «Members» откроет список участников (для выбранной сети), но нажатие ведёт на ту же страницу Networks. Элемент вводит в заблуждение.

**Рекомендация:**  
Удалить дублирующий пункт «Members» из сайдбара — доступ к членам осуществляется через кнопку «Members» на странице Networks. Либо — если нужна прямая навигация — реализовать выбор сети через `Modal.prompt()`:

```js
// onclick для Members
async function navigateToMembers() {
  const netId = await Modal.prompt('Enter Network ID to view members');
  if (netId) Router.navigate(`/controllers/members/${netId}`);
}
```

---

## MEDIUM

---

### #10 — `ZtLocalClient` создаётся заново на каждый запрос

**Приоритет:** 🟡 MEDIUM  
**Категория:** Performance  
**Файл:** `src/server/handlers/local.rs:12–15`, `src/zerotier/local/client.rs:22–34`  

**Описание:**  
Функция `client()` в `local.rs` создаёт новый `ZtLocalClient` при каждом входящем HTTP-запросе, читая токен с диска и инициализируя новый `reqwest::Client`. `reqwest::Client` внутри использует connection pool, но создание нового экземпляра на каждый запрос теряет все его преимущества: каждый запрос открывает новое TCP-соединение к `127.0.0.1:9993`.

```rust
// local.rs — вызывается при КАЖДОМ запросе
async fn client(state: &AppState) -> Result<ZtLocalClient, ApiError> {
    let cfg = state.config.read().await;
    ZtLocalClient::from_config(&cfg.zerotier.local) // ← новый Client + чтение файла
}
```

**Сравнение:** `ZtCentralClient` правильно кешируется через `TokenStore::active_client()`.

**Рекомендация:**  
Добавить `cached_local_client: Arc<RwLock<Option<ZtLocalClient>>>` в `AppState` с инвалидацией при изменении конфига — по аналогии с `TokenStore`.

---

### #11 — Два независимых `struct TokenView` с разными полями

**Приоритет:** 🟡 MEDIUM  
**Категория:** Duplication  
**Файлы:** `src/server/handlers/config.rs:33`, `src/server/handlers/tokens.rs:17`  

**Описание:**  
Оба файла объявляют `pub struct TokenView`, но с разным набором полей. `config.rs` использует поле `token: String` (замаскированный), тогда как `tokens.rs` использует `masked_token: String`. Это семантическое расхождение при описании одной и той же концепции.

```rust
// config.rs
pub struct TokenView { pub token: String, /* masked */ ... }

// tokens.rs
pub struct TokenView { pub masked_token: String, ... }
```

**Рекомендация:**  
Вынести единственный `TokenView` в `src/server/types.rs` или `src/config/schema.rs` с унифицированным именем поля. Оба хендлера импортируют его оттуда.

---

### #12 — Четыре `struct EnableRequest` в разных хендлерах

**Приоритет:** 🟡 MEDIUM  
**Категория:** Duplication  
**Файлы:** `handlers/bridge.rs:39`, `handlers/exitnode.rs:57`, `handlers/physnet.rs:51`, `handlers/ndp.rs:29`  

**Описание:**  
Каждый хендлер объявляет собственный `struct EnableRequest` с уникальными полями. Само по себе это нормальная практика (разные поля), но общее наименование без namespace создаёт путаницу при grep и code navigation, а также затрудняет рефакторинг.

**Рекомендация:**  
Переименовать структуры, добавив контекст:
```rust
// вместо EnableRequest:
pub struct BridgeEnableRequest { ... }
pub struct ExitNodeEnableRequest { ... }
pub struct PhysNetEnableRequest { ... }
pub struct NdpEnableRequest { ... }
```

---

### #13 — `latencyClass()` скопирована в двух модулях

**Приоритет:** 🟡 MEDIUM  
**Категория:** Duplication / DRY  
**Файлы:** `www/src/js/pages/dashboard.js:11`, `www/src/js/pages/peers.js:4`  

**Описание:**  
Функция `latencyClass(ms)` дословно скопирована в двух файлах с идентичной логикой:

```js
// dashboard.js:11 и peers.js:4 — точная копия
function latencyClass(ms) {
  if (ms < 50) return 'latency-good';
  if (ms < 150) return 'latency-medium';
  return 'latency-bad';
}
```

**Рекомендация:**  
Перенести в `Utils` (уже существует в `state.js`):

```js
// state.js — Utils
const Utils = (() => {
  function esc(s) { /* ... */ }
  function latencyClass(ms) {
    if (ms < 50) return 'latency-good';
    if (ms < 150) return 'latency-medium';
    return 'latency-bad';
  }
  return { esc, latencyClass };
})();
```

---

### #14 — Backend-эндпоинты без соответствующих страниц во frontend

**Приоритет:** 🟡 MEDIUM  
**Категория:** API Gap  
**Файлы:** `src/server/router.rs`, `src/server/handlers/`  

**Описание:**  
Следующие backend-маршруты реализованы полностью, но ни одна страница во frontend их не вызывает:

| Маршрут | Хендлер | Статус |
|---------|---------|--------|
| `GET /api/local/peers/:id` | `local_handler::get_peer` | Нет UI |
| `GET /api/local/networks/:id/localconf` | `lc_handler::get_network_local_conf` | Нет UI |
| `PUT /api/local/networks/:id/localconf` | `lc_handler::update_network_local_conf` | Нет UI |
| `GET /api/metrics/raw` | `metrics_handler::get_raw` | Нет UI |
| `GET /api/central/user` | `central_handler::get_user` | Нет UI |
| `GET /api/central/status` | `central_handler::get_status` | Нет UI |
| `PUT /api/settings/tokens/:id` | `tok_handler::update_token` | Нет UI |

**Рекомендация:**  
Для каждого необходимо принять решение:
- Создать соответствующий UI (per-network local.conf — ценная функциональность для настройки allowDefault/allowGlobal).
- Задокументировать как «только API» (для внешних клиентов).
- Удалить, если не планируется использование.

Особенно важен `PUT /local/networks/:id/localconf` — именно через него должна работать настройка allowDefault/allowGlobal для клиентов Exit Node, что является частью основного user flow.

---

### #15 — `Toast.warn()` не существует; вызывается в 5 местах

**Приоритет:** 🟡 MEDIUM  
**Категория:** Bug / JS  
**Файлы:** `toast.js`, `exitnode.js:177`, `relay.js:109`, `settings-ztnode.js:156`, и другие  

**Описание:**  
`Toast` экспортирует только три метода: `success`, `error`, `info`. Метод `warn` отсутствует. Тем не менее в коде встречается 5 вызовов `Toast.warn(...)`, каждый из которых молча завершается ошибкой `TypeError: Toast.warn is not a function` — предупреждения пользователю не показываются.

**Вызовы, которые не работают:**
```js
exitnode.js:177   res.warnings.forEach(w => Toast.warn(w));      // ❌
relay.js:109      res.warnings.forEach(w => Toast.warn(w));      // ❌
settings-ztnode.js:156  Toast.warn(`Saved with ${n} warning(s)`); // ❌
```

**Рекомендация:**  
Добавить метод `warn` в `Toast`:

```js
// toast.js
return {
  success: m => show(m, 'success'),
  error:   m => show(m, 'error'),
  info:    m => show(m, 'info'),
  warn:    m => show(m, 'warn'),  // ✅ добавить
};
```

И добавить соответствующие CSS-стили:
```css
.toast-warn { border-left: 3px solid var(--c-warn); }
```

---

### #16 — `std::env::set_var` в тестах — data race в многопоточной среде

**Приоритет:** 🟡 MEDIUM  
**Категория:** Security / Correctness  
**Файлы:** `src/config/env.rs:37–65`, `src/deps.rs:394–397`  

**Описание:**  
В unit-тестах используется `std::env::set_var` / `remove_var`. Начиная с Rust 1.80, вызов `set_var` из многопоточного контекста официально помечен как **undefined behavior** (RUSTSEC-2024-0375). Cargo запускает тесты в параллельных потоках по умолчанию, что может приводить к гонкам: один тест устанавливает переменную, другой читает дефолтное значение — и наоборот.

В `env.rs` есть попытка сериализации через `Mutex<()>`, но `deps.rs` не использует никакой защиты.

**Рекомендация:**  
- Использовать `temp-env` crate или `serial_test` для изоляции тестов переменных окружения.
- Или переписать функции так, чтобы переменные окружения принимались как параметр, а не читались из глобального состояния — это упростит тестирование без мутации окружения.

---

### #17 — Зависимость от системного `ssh`; path traversal через `key_path`

**Приоритет:** 🟡 MEDIUM  
**Категория:** Security / External Dependency  
**Файлы:** `src/relay/ssh.rs`, `src/relay/deploy.rs`  

**Описание:**  
`SshClient::run()` запускает системный бинарник `ssh`. Параметр `key_path` передаётся из JSON-тела запроса через `-i key_path`. Злоумышленник, отправивший запрос на `POST /api/relay/deploy` с `key_path: "../../../../etc/passwd"`, вынуждает `ssh` читать произвольный файл файловой системы как приватный ключ. Вход с localhost не защищён авторизацией API-токена.

**Проблемный код:**
```rust
// ssh.rs — key_path приходит из HTTP-запроса напрямую
if let Some(ref key) = self.key_path {
    args.push("-i".into());
    args.push(key.clone()); // ← path traversal, нет валидации
}
```

**Почему проблема:**  
Хотя `ssh` не выполнит `etc/passwd` как ключ (он не в формате PEM), содержимое файла попадёт в сообщение об ошибке SSH, которое возвращается клиенту через `SshError::Failed { stderr }` → `ApiError` → HTTP-ответ 500. Это информационная утечка.

**Рекомендация:**  
Валидировать `key_path` перед использованием:

```rust
fn validate_key_path(path: &str) -> Result<(), ApiError> {
    let p = std::path::Path::new(path);
    // Запретить относительные пути и path traversal
    if !p.is_absolute() {
        return Err(ApiError::InvalidInput("key_path must be an absolute path".into()));
    }
    if path.contains("..") {
        return Err(ApiError::InvalidInput("key_path must not contain '..'".into()));
    }
    // Убедиться, что файл существует и доступен для чтения
    if !p.exists() {
        return Err(ApiError::InvalidInput(format!("key file not found: {path}")));
    }
    Ok(())
}
```

---

### #18 — CORS origin hardcode `localhost`, не покрывает IPv6 `[::1]`

**Приоритет:** 🟡 MEDIUM  
**Категория:** Hardcode  
**Файл:** `src/server/router.rs:26–37`  

**Описание:**  
При построении CORS разрешённые origins формируются только для `http://{host}:{port}` и `http://localhost:{port}`. Если сервер привязан к `[::1]` (IPv6 loopback), браузер на `http://[::1]:3000` получит CORS-ошибку.

**Проблемный код:**
```rust
let origin_host = format!("http://{host}:{port}");  // e.g. http://127.0.0.1:3000
let origin_lo   = format!("http://localhost:{port}"); // жёстко закодировано
// ❌ отсутствует: http://[::1]:3000
```

**Рекомендация:**  
```rust
let mut origins = vec![
    format!("http://{host}:{port}").parse::<HeaderValue>().expect("origin"),
    format!("http://localhost:{port}").parse::<HeaderValue>().expect("origin"),
];
// Добавить IPv6 loopback если хост — IPv4 loopback
if host == "127.0.0.1" {
    if let Ok(v) = format!("http://[::1]:{port}").parse::<HeaderValue>() {
        origins.push(v);
    }
}
let cors = CorsLayer::new()
    .allow_methods([...])
    .allow_headers([...])
    .allow_origin(origins);
```

---

### #19 — Предупреждения Exit Node теряются из-за `Toast.warn()`

**Приоритет:** 🟡 MEDIUM  
**Категория:** UX / Bug  
**Файл:** `www/src/js/pages/exitnode.js:177`  

**Описание:**  
Бэкенд `POST /exitnode/enable` возвращает массив `warnings` с важными сообщениями (например, «allowDefault не установлен», «allowGlobal не установлен»). Frontend пытается показать их через `Toast.warn()`, который не существует (#15). В результате пользователь включает Exit Node, не видя критических предупреждений о конфигурации клиентов.

**Рекомендация:**  
После добавления `Toast.warn()` (см. #15) — убедиться, что все предупреждения отображаются. Дополнительно рассмотреть показ предупреждений в блоке statusBlock прямо на странице, а не только через toast (они исчезают через 4 секунды).

---

### #20 — Нет минификации, весь JS в глобальном scope

**Приоритет:** 🟡 MEDIUM  
**Категория:** Architecture  
**Файл:** `build.rs`  

**Описание:**  
`build.rs` конкатенирует все CSS и JS файлы без минификации и упаковывает в один `<script>` блок. Все константы, классы и функции объявлены в глобальном `window` scope. Это создаёт риск конфликтов имён при расширении проекта, и каждый IIFE-модуль виден извне.

Текущий размер бандла печатается при сборке (`cargo:warning=Frontend built: X KB`). Без минификации при росте кода это заметно скажется на времени загрузки.

**Рекомендация:**  
- Минимальный вариант: добавить минификацию CSS через `lightningcss` и JS через `swc` или `esbuild` (вызывать из `build.rs` как `Command::new("esbuild")`).
- Прогрессивный вариант: перейти на ES-модули и использовать `vite` или `rollup` для сборки.
- Краткосрочно: добавить `"use strict"` в начало каждого IIFE.

---

## LOW

---

### #21 — `IP_POOLS` — hardcode 24 IP-пулов в JS

**Приоритет:** 🟢 LOW  
**Категория:** Hardcode  
**Файл:** `www/src/js/pages/controllers-config.js:2–9`  

**Описание:**  
Список IP-пулов для auto-assignment жёстко зашит в JS-массив. Добавление нового пула требует изменения исходного кода. Эти пулы пересекаются с рекомендуемыми пулами ZeroTier Central, но не синхронизируются автоматически.

**Рекомендация:**  
Рассмотреть загрузку пулов с сервера или из конфига, либо документировать список как «фиксированный, соответствует рекомендациям ZeroTier».

---

### #22–#24 — Неиспользуемые backend-эндпоинты (детали)

**Приоритет:** 🟢 LOW  
**Файлы:** `src/server/handlers/local_config.rs`, `src/server/handlers/metrics.rs`, `src/server/handlers/central.rs`  

- **#22:** `GET/PUT /local/networks/:id/localconf` — управление `allowManaged`, `allowDefault`, `allowGlobal`, `allowDNS` на уровне каждой сети. Эта функциональность критически важна для пользователей Exit Node, но UI страница `network-detail.js` вкладка `config` реализует её через `POST /local/networks/:id` (join_network) вместо правильного `PUT /local/networks/:id/localconf`.
- **#23:** `GET /metrics/raw` возвращает сырой Prometheus-текст — полезен для отладки, но не отображается нигде в UI.
- **#24:** `PUT /settings/tokens/:id` позволяет переименовать токен — в `SettingsTokensPage` нет кнопки Edit.

---

### #25 — Блокирующие `Command::new` в `deps.rs`

**Приоритет:** 🟢 LOW  
**Категория:** Architecture  
**Файл:** `src/deps.rs`  

**Описание:**  
Функция `ensure()` вызывается из `main()` до запуска Tokio-сервера — это нормально. Однако если `ensure()` когда-либо будет перенесена в async-контекст (например, в health-check endpoint), блокирующие `Command::new` заблокируют worker thread. Отсутствует комментарий с предупреждением об этом ограничении.

**Рекомендация:**  
Добавить doc-comment:
```rust
/// # Panics / Blocking
/// This function blocks the calling thread via `std::process::Command`.
/// Must be called before `tokio::main` starts, or wrapped in `spawn_blocking`.
pub fn ensure() -> Result<(), DepsError> { ... }
```

---

### #26 — `MetricsConfig::default()` включает `enabled: true`

**Приоритет:** 🟢 LOW  
**Категория:** Configuration  
**Файл:** `src/config/schema.rs:116`  

**Описание:**  
По умолчанию сбор метрик включён (`enabled: true`). На системах, где ZeroTier не запущен или `metricstoken.secret` отсутствует, в лог каждые 15 секунд пишется `WARN metrics: fetch failed`. Для первого запуска пользователю придётся либо отключить метрики, либо разобраться с конфигурацией.

**Рекомендация:**  
Изменить дефолт на `enabled: false` и документировать в `config.yml.example`:

```rust
impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            enabled: false, // ← opt-in, не opt-out
            // ...
        }
    }
}
```

---

## Соответствие Frontend ↔ Backend API

### Критические несоответствия

| Frontend вызов | Backend маршрут | Результат |
|----------------|-----------------|-----------|
| `POST /exitnode/deps/install` | ❌ Не существует (есть `POST /exitnode/deps`) | **404** |
| `Toast.warn(...)` × 5 мест | ❌ Метод отсутствует в `Toast` | **TypeError (silent)** |
| `zt_interface: <network_id>` в exitnode enable | ❌ Ожидается имя интерфейса, не ID | **iptables ошибка** |

### Маршруты backend без UI

| Маршрут | Описание |
|---------|----------|
| `GET /api/local/peers/:id` | Детали одного пира |
| `GET /api/local/networks/:id/localconf` | Per-network local.conf |
| `PUT /api/local/networks/:id/localconf` | Обновление per-network local.conf |
| `GET /api/metrics/raw` | Сырые Prometheus-метрики |
| `GET /api/central/user` | Аккаунт ZeroTier Central |
| `GET /api/central/status` | Статус Central API |
| `PUT /api/settings/tokens/:id` | Редактирование токена |

### Маршруты, работающие корректно

Следующие маршруты были проверены и соответствуют реализации:

- ✅ Все CRUD-операции для Local Networks, Controller Networks, Members
- ✅ `POST /bridge/deps/install` (Bridge)
- ✅ `POST /relay/deploy`, `GET /relay/verify`, `POST /relay/remote`
- ✅ `GET/PUT /relay/local`
- ✅ `GET/PUT /settings/config`
- ✅ `GET/POST/DELETE/POST(:id/activate) /settings/tokens`
- ✅ `POST /settings/tokens/validate`
- ✅ `GET/PUT /local/config` (ZT node settings)
- ✅ `GET/POST /local/moons/:id`, `DELETE /local/moons/:id`
- ✅ All exitnode NDP sub-routes (`/ndp/status`, `/ndp/install`, `/ndp/enable`, `/ndp/disable`)

---

## Общие архитектурные рекомендации

### Rust Backend

1. **Кеширование `ZtLocalClient`** — добавить в `AppState` аналогично `TokenStore`, чтобы не создавать новый `reqwest::Client` и не читать токен с диска на каждый запрос.

2. **Унификация обработки ошибок** — некоторые хендлеры возвращают `ApiError::ZtLocal` там, где семантически уместнее `ApiError::InvalidInput` или специфический вариант. Рассмотреть добавление вариантов `Bridge`, `PhysNet`, `Relay` в `ApiError`.

3. **Graceful shutdown** — нет обработки `SIGTERM` для корректного завершения SSE-соединений и фоновых задач (metrics collector). Рекомендуется использовать `axum::serve().with_graceful_shutdown()`.

4. **Rate-limiter lifetime** — связать жизненный цикл задачи rate-limiter с жизненным циклом клиента через `CancellationToken` или `Drop`.

### Frontend

5. **Модульная система** — переход на ES-модули устранит большинство проблем с глобальным namespace и упростит тестирование.

6. **Единый слой XSS-защиты** — создать функцию `safeHTML(strings, ...values)` (tagged template literal), которая автоматически экранирует все интерполируемые значения, кроме явно помеченных как `raw(...)`.

7. **Консистентное использование Utils.esc()** — обязательно применять для всех данных, приходящих с сервера и вставляемых через `innerHTML`.

8. **Централизованный error handler** — вместо повторяющегося `catch(e) { Toast.error(e.message) }` в 37 местах — единая функция `handleApiError(e)` с логикой fallback и дифференциацией ошибок.

### Тестирование

9. **Интеграционные тесты API** — существующий `tests/api_health.rs` проверяет только `/api/health`. Добавить smoke-тесты для критических маршрутов (exitnode enable/disable, bridge enable/disable) с мок-зависимостями.

10. **Frontend тесты** — нет ни одного JS-теста. Для критических утилит (валидация network ID, `poolToRange`, `latencyClass`) добавить минимальные unit-тесты через Vitest или Jest.

---

*Аудит проведён на основе статического анализа кода версии 0.9.1. Все выводы основаны на фактах, подтверждённых конкретными строками кода.*
---

## Аудит-3 (2026-04-18) — Screenshots workflow

**Дата:** 2026-04-18  
**Файл:** `.github/workflows/screenshots.yml`  
**Проблема:** Все 22 скриншота показывали только «Loading…» — реальный UI не отображался ни на одном снимке. Все desktop-скриншоты байтово идентичны (MD5 `04b928773b6a9808b19f4fcfd7f91fbc`), все mobile — по 2-3 идентичных.

### Итоговая таблица

| # | Приоритет | Проблема | Статус |
|---|-----------|----------|--------|
| SCR-1 | 🔴 Critical | Шаг «Start ztnet-box» был закомментирован — Playwright подключался к порту 7979 где ничего не слушало | ✅ |
| SCR-2 | 🔴 Critical | Нет ZeroTier daemon в CI — все `/api/local/*` возвращали ошибку, страницы зависали в Loading | ✅ |
| SCR-3 | 🔴 Critical | `waitForSelector('.page')` срабатывал на начальный shell-div ДО запуска роутера | ✅ |
| SCR-4 | 🔴 High | `networkidle` срабатывал в паузе между загрузкой HTML и выполнением JS — до первых XHR | ✅ |
| SCR-5 | 🟡 Medium | Mobile: sidebar не закрывался перед скриншотом (класс `.open` не снимался) | ✅ |
| SCR-6 | 🟡 Medium | Маршрут `'/'` дублировал dashboard (SPA редирект), 2 одинаковых скриншота | ✅ |
| SCR-7 | 🟢 Low | `cargo cache` был закомментирован — каждый run компилировал с нуля (~10 мин) | ✅ |

### Детали

**SCR-1** `screenshots.yml` — шаг `Start ztnet-box` был закомментирован (`#`). Playwright обращался к `http://127.0.0.1:7979`, где ничего не слушало. Все навигации падали с Connection Refused, но `|| true` скрывал ошибку. Итог: Playwright снимал пустую страницу браузера с сайдбаром из предыдущего состояния. Исправление: шаг восстановлен с `sudo` (обязательно — `authtoken.secret` принадлежит root).

**SCR-2** ZeroTier `zerotier-one` был установлен но не запускался в нужном режиме. Шаг `Start ZeroTier` использовал `systemctl start`, что в GitHub Actions CI-среде (без systemd) не работало корректно. Добавлена проверка через `curl http://127.0.0.1:9993/status -H "X-ZT1-Auth: ..."` после `zerotier-one -U -d` (userspace mode, без TUN/TAP).

**SCR-3** `waitForSelector('.page')` — shell.html уже содержит `<div class="page"><div class="loading-row">` ещё до запуска JS роутера. Playwright находил `.page` немедленно, страница оставалась в Loading. Заменено на `waitForFunction()` который проверяет что `.loading-row` больше не присутствует в `#content` — это гарантирует что роутер заменил shell реальным контентом.

**SCR-4** `page.goto() + waitForLoadState('networkidle')` — `networkidle` срабатывал в 500мс тишине между загрузкой HTML и первым XHR от JS. `page.goto()` переключён на `waitUntil: 'domcontentloaded'`; `networkidle` перенесён на после проверки загрузки.

**SCR-5** `page.evaluate()` теперь снимает `.open` с sidebar и `.visible` с overlay перед мобильным скриншотом + 300мс пауза для завершения CSS-перехода.

**SCR-6** `'/'` → `'root'` заменено на `'/#/settings/roots'` → `'root'`. 11 уникальных страниц вместо 10 + дубликата.

**SCR-7** Cargo cache восстановлен (был закомментирован вместе со `Start ztnet-box`).

**Коммит:** `44f452c`

---

## Roadmap-1 (2026-04-18) — UX/Architecture backlog

Задачи по результатам product review. Статус: **открыты**, реализация не начата.

---

### RD-1 🔴 HIGH — Удалить страницу Peers из навигации

**Файлы:** `www/src/html/shell.html`, `www/src/js/pages/dashboard.js`, `www/src/js/pages/peers.js`

Страница `/peers` дублирует таблицу пиров на Dashboard (раздел «Peers» уже присутствует). Самостоятельной ценности страница не несёт: она показывает те же поля, что и Dashboard, но без автообновления метрик и без контекста node status.

**Что сделать:**
- Убрать nav-item `Peers` из sidebar (`shell.html`)
- Убрать регистрацию маршрута `/peers` из `Router.on`
- Убрать секцию NODE из sidebar — останется только `Dashboard`
- Файл `peers.js` оставить или удалить (логика переезжает в `dashboard.js`)
- Таблицу пиров на Dashboard расширить: добавить поля Version и Physical IP, которые сейчас есть только на отдельной странице

---

### RD-2 🔴 HIGH — Dashboard: подключение к сети + статус подключённых сетей + участники

**Файлы:** `www/src/js/pages/dashboard.js`, `www/src/css/pages.css`

**Текущее состояние:** Dashboard показывает node status, метрики и таблицу пиров. Подключённые сети и их участники — на отдельных страницах `/networks` и `/controllers/members/:id`.

**Что сделать:**

1. **Join-виджет прямо на Dashboard** — поле ввода Network ID (16 hex) + кнопка «Join» рядом со статусом ноды. Вызывает `POST /api/local/networks/:id`. При успехе список сетей ниже обновляется.

2. **Карточки подключённых сетей** — под метриками отображать каждую активную сеть как карточку:
   ```
   ┌─────────────────────────────────────┐
   │ [badge: OK]  8056c2e21c000001       │  ← ID + статус
   │ mynet.example.com  172.27.0.5/16    │  ← имя + assigned IP
   │ Members online: 3/12  [Details →]  │  ← участники + переход
   └─────────────────────────────────────┘
   ```

3. **Участники online/offline** — для каждой сети запрашивать `GET /api/local/controller/networks/:id/members` (если локальный контроллер) или `GET /api/central/networks/:id/members` (если есть токен). Считать online тех у кого `lastOnline > (now - 5min)`. Показывать счётчик `N/Total` и dot-индикаторы для первых 8 участников.

4. **Автообновление** — включить в существующий `setInterval(10s)` refresh.

**API:** уже реализованы `GET /api/local/networks`, `GET /api/local/controller/networks/:id/members`, `GET /api/central/networks/:id/members`. Дополнительный backend не нужен.

---

### RD-3 🔴 HIGH — Реструктуризация меню: CLIENT / SERVER разделение

**Файлы:** `www/src/html/shell.html`, `www/src/css/layout.css`, возможно новые JS-модули

#### Анализ текущей структуры

```
NODE          → Dashboard, Peers
MY NETWORKS   → Networks
CONTROLLERS   → Networks, Members
NETWORK       → Exit Node, Phys Routing, L2 Bridge, TCP Relay
SETTINGS      → Global, ZeroTier Node, Root Servers, API Tokens
```

Проблема: секция **NETWORK** содержит 4 несвязанных с точки зрения пользователя страницы — это разные режимы работы ноды как шлюза. Пользователь, который просто подключается к чужим сетям (CLIENT), никогда не заходит на эти страницы. Пользователь, который управляет контроллером (SERVER), заходит редко. Обе группы видят перегруженное меню.

NDP Proxy вообще отсутствует в sidebar, хотя его страница существует как часть Exit Node.

#### Предлагаемая структура

```
УЗЕЛ
  Dashboard          ← node status + join + сети + пиры (после RD-2)

МОИ СЕТИ
  Сети               ← список + join + настройки per-network

КОНТРОЛЛЕР           ← секция видна всегда, но с hint "нужен токен/локальный контроллер"
  Сети               ← управление контроллером
  Участники          ← открывается из контекста сети

ШЛЮЗ ▾              ← КОЛЛАПСИРУЕМАЯ секция, по умолчанию свёрнута
  Exit Node          ← с NDP Proxy внутри страницы (уже реализовано)
  Physical Routing   
  L2 Bridge          
  TCP Relay          

НАСТРОЙКИ
  Глобальные
  ZeroTier Node
  Root Servers
  API Токены
```

#### Ключевые решения

**Секция ШЛЮЗ — коллапсируемая:**
- По умолчанию свёрнута если ни одна из функций не активна (`nav-exitnode-badge` / `nav-bridge-badge` / `nav-physnet-badge` = не активны)
- Раскрывается автоматически если хотя бы одна функция активна (badge показывает dot)
- На мобильных: всегда свёрнута по умолчанию
- Реализация: CSS `.nav-group.collapsed > .nav-group-items { display: none }` + toggle по клику на заголовок

**Почему НЕ полное CLIENT/SERVER разделение с двумя режимами:**
- Пользователь может одновременно быть клиентом чужой сети и хостом своей
- Переключение режима добавляет шаг навигации и ломает прямые ссылки
- Статус (есть ли локальный контроллер / есть ли API токен) меняется динамически
- Достаточно: визуально отделить "шлюзовые" фичи через коллапс + сохранить всё в одном sidebar

**NDP Proxy:**
- Остаётся на странице Exit Node как подсекция (уже реализовано через `#exitnode/ndp`)
- В sidebar отдельный пункт не нужен

**Мобильная адаптация:**
- Секция ШЛЮЗ свёрнута по умолчанию — сокращает мобильный sidebar с 13 пунктов до 9
- Раскрывается тапом по заголовку секции

**Реализация заголовка секции:**
```html
<div class="nav-section nav-group" id="gateway-group">
  <div class="nav-section-label nav-group-toggle" onclick="toggleNavGroup('gateway-group')">
    Шлюз
    <svg class="nav-group-chevron">…</svg>
  </div>
  <div class="nav-group-items">
    <!-- Exit Node, Phys Routing, L2 Bridge, TCP Relay -->
  </div>
</div>
```

---

### RD-4 🟡 MEDIUM — Расширение тестового покрытия

**Файлы:** `tests/api_health.rs` (новые файлы: `tests/api_networks.rs`, `tests/api_tokens.rs`, ...)

**Текущее состояние:** один тест — `GET /api/health` возвращает 200.

**Что добавить:**

#### Группа: Tokens (без моков, только валидация)
```rust
// tests/api_tokens.rs
POST /api/settings/tokens          → 201 Created, UUID в ответе
GET  /api/settings/tokens          → 200, массив
PUT  /api/settings/tokens/:id      → 200, имя изменилось
POST /api/settings/tokens/:id/activate → 200
DELETE /api/settings/tokens/:id    → 204
```

#### Группа: Networks — join/leave (мок ZT daemon)
```rust
// Нужен мок-сервер zerotier-one на localhost:PORT
POST /api/local/networks/:id       → 200 (join)
DELETE /api/local/networks/:id     → 200 (leave)
GET  /api/local/networks           → 200, массив
GET  /api/local/networks/:id       → 200, объект
```

#### Группа: Controller — create/list/delete network
```rust
POST /api/local/controller/networks          → 200
GET  /api/local/controller/networks          → 200, содержит созданную сеть
GET  /api/local/controller/networks/:id      → 200
PUT  /api/local/controller/networks/:id      → 200, изменения применились
DELETE /api/local/controller/networks/:id    → 200
```

#### Группа: Members — add/authorize/delete
```rust
PUT  /api/local/controller/networks/:net/members/:node  → 200
GET  /api/local/controller/networks/:net/members        → 200
DELETE /api/local/controller/networks/:net/members/:node → 200
```

#### Группа: Feature routes — smoke (без реальной ОС)
```rust
// Проверяем что маршруты существуют и возвращают корректный JSON, не 404/500
GET /api/exitnode/platform   → 200, { supported: bool }
GET /api/exitnode/deps       → 200
GET /api/physnet/platform    → 200
GET /api/bridge/platform     → 200
GET /api/relay/status        → 200
GET /api/metrics/status      → 200
```

**Инфраструктура для мок-ZT:**
- Запустить `axum` сервер на случайном порту в `#[tokio::test]`
- Передать его URL через `AppState` с кастомным `api_url` в конфиге
- Не требует прав root, не зависит от ОС

**Оценка объёма:** ~400–600 строк Rust, ~3–4 новых test-файла.

---

### Итоговая таблица Roadmap-1

| # | Приоритет | Компонент | Задача | Статус |
|---|-----------|-----------|--------|--------|
| RD-1 | 🔴 High | Frontend | Удалить страницу Peers (дублирует Dashboard) | ✅ `afaab91` |
| RD-2 | 🔴 High | Frontend | Dashboard: join-виджет + карточки сетей + участники online | ✅ `afaab91` |
| RD-3 | 🔴 High | Frontend | Реструктуризация sidebar: коллапс секции ШЛЮЗ + убрать Peers из NODE | ✅ `afaab91` |
| RD-4 | 🟡 Medium | Rust/Tests | Интеграционные тесты: tokens, join/leave, controller CRUD, members, smoke routes | ✅ (этот коммит) |

#### Реализация RD-4

**Добавлены два новых тест-файла:**

`tests/api_local.rs` — 9 тестов против реального ZT daemon:
- `local_node_status_returns_address` — `/api/local/status`
- `local_peers_returns_array` / `local_peer_invalid_id_returns_422`
- `local_networks_list_returns_array`
- `local_network_join_and_leave` — join → verify in list → get → leave → verify gone (использует `ZT_TEST_NETWORK` env или Earth network `8056c2e21c000001`)
- `controller_network_crud` — create → get → list → update (rename) → delete → verify gone
- `controller_members_crud` — create net → authorize member → list → get → deauthorize → delete member → cleanup net
- `local_moons_list_returns_array`
- `local_config_roundtrip`

`tests/api_central.rs` — 9 тестов против реального ZeroTier Central API:
- `central_status_returns_structure` / `central_user_returns_account_info`
- `settings_tokens_list_includes_injected_token` — проверяет masked_token, отсутствие raw token
- `settings_token_add_invalid_token_returns_error` / `settings_token_empty_name_returns_422`
- `settings_token_validate_real_token` — реальная валидация токена через Central API
- `central_network_list_returns_array`
- `central_network_crud` — create → get → update → list members → delete → verify gone

**Skip-стратегия:** тесты автоматически пропускаются (не падают) при недоступном daemon или отсутствующем токене. Никаких мок-серверов.

**Запуск:**
```bash
# Локальные тесты (требует ZT daemon)
sudo ZT_RUNNING=1 cargo test --test api_local

# Central тесты (требует токен)
ZT_CENTRAL_TOKEN=<token> cargo test --test api_central

# С конкретной тестовой сетью
ZT_TEST_NETWORK=8056c2e21c000001 sudo cargo test --test api_local local_network_join_and_leave
```


---

## Roadmap-2 (2026-04-24) — Branding, UX, Docs

### RD-5 🔴 HIGH — Переименование: ZeroBox → ZTNetwork Panel

| Компонент | Текущее значение | Новое значение |
|-----------|-----------------|----------------|
| `sidebar-logo-text` в `shell.html` | `ZeroBox` | `ZTNetwork Panel` |
| `mobile-bar-title` в `shell.html` | `ZeroBox` | `ZTNetwork Panel` |
| `<title>` в `build.rs` HTML template | `ztnet-box` | `ZTNetwork Panel` |
| `package.json` / README title | `ztnet-box` | `ZTNetwork Panel` |
| Tab/window title при навигации | нет | `{Page} — ZTNetwork Panel` |

**Цель:** единое брендирование. Бинарный файл остаётся `ztnet-box`, репозиторий не переименовывается — меняется только UI-название.

---

### RD-6 🔴 HIGH — Help-панели для Exit Node / L2 Bridge / TCP Relay

На каждой из страниц добавить сворачиваемый блок **«How it works»** с:
- Принцип работы (1-2 абзаца, схема потока трафика)
- Требования к системе (ОС, привилегии, пакеты)
- Шаги настройки
- Диагностика / частые ошибки

#### Exit Node — требования и принципы

**Требования:**
- Linux, root (`sudo`)
- `iptables` ≥ 1.8 **или** `nftables` ≥ 0.9
- `ip_forward` (ztnet-box включает автоматически)
- ZeroTier сеть с `allowDefault=1` + `allowGlobal=1` на клиентских нодах

**Принцип:**
```
ZT peer → [zerotier interface] → MASQUERADE → [WAN interface] → Internet
```
ztnet-box применяет правила: `iptables -t nat -A POSTROUTING -o <wan> -j MASQUERADE`
и `FORWARD ACCEPT` для трафика из ZT. Состояние сохраняется в `state.json`.

**Диагностика:**
- `curl https://ipinfo.io` с клиента должен показывать IP exit-ноды
- `sudo iptables -t nat -L -v` — проверить MASQUERADE правило
- Если не работает: `cat /proc/sys/net/ipv4/ip_forward` должен быть `1`

#### L2 Bridge — требования и принципы

**Требования:**
- Linux, root
- `iproute2` (команда `ip`)
- `bridge-utils` (опционально, для `brctl`)
- `systemd-networkd` (для persistence) **или** manual `ip link`

**Принцип:**
```
Physical LAN ──[eth0]──┐
                        ├── [br0 bridge] ── ZT peers видят физическую сеть L2
ZeroTier ──[zt*]───────┘
```
ZeroTier пиры получают адреса из физического DHCP-сервера и видны на LAN как реальные устройства.

**Требования в ZeroTier Central:** для bridge-ноды включить `Bridging` в настройках участника.

**Диагностика:**
- `ip link show br0` — bridge должен быть UP
- `bridge fdb show` — таблица MAC-адресов
- `brctl showstp br0` — состояние Spanning Tree

#### TCP Relay — требования и принципы

**Требования:**
- SSH-ключ (key-based auth, без пароля)
- Docker на удалённом хосте (ztnet-box установит автоматически)
- Открытый порт на удалённом хосте (по умолчанию 443)
- `ssh` в PATH на локальной машине

**Принцип:**
```
ZT node ──TCP──► [relay server:443] ──── другие ZT ноды
                  (pylon reflect container)
```
Используется когда прямое UDP соединение невозможно (строгий NAT, firewall).
ztnet-box деплоит `zerotier/pylon:latest reflect` через SSH, прописывает endpoint в `local.conf`.

**Диагностика:**
- `ssh user@host docker ps | grep pylon` — контейнер должен работать
- ZeroTier: `zerotier-cli info` — проверить latency после подключения через relay

---

### RD-7 🟡 MEDIUM — Log Panel: кнопка очистки + zero-request по умолчанию

**Текущее поведение:** LogPanel делает запросы к `/api/logs` и открывает SSE `/api/logs/stream` при инициализации страницы, даже если панель закрыта.

**Требуемое поведение:**
1. При закрытой панели (`collapsed`) — **ноль запросов** к `/api/logs*`
2. SSE-соединение открывается **только** после нажатия ▶ (Play/Start)
3. При паузе (⏸) — SSE закрывается (`EventSource.close()`)
4. Кнопка **«Clear»** (🗑) — очищает буфер через `DELETE /api/logs`
5. Состояние (open/closed, level) сохраняется в `localStorage`

**Backend:** `DELETE /api/logs` уже реализован. Нужно только подключить во frontend.

---

### RD-8 🟡 MEDIUM — Реструктуризация README.md

**Текущая проблема:** README.md — 412 строк, содержит всё: конфиг, API-справочник, примеры, security, troubleshooting. Сложно поддерживать, тяжело читать.

**Цель:** README.md — компактный (~80 строк), ссылается на `docs/`.

**Новая структура docs/:**
```
docs/
├── screenshots/          # (уже существует)
├── configuration.md      # Полный config.yml reference + env vars
├── exit-node.md          # Детальная документация Exit Node
├── l2-bridge.md          # Детальная документация L2 Bridge
├── tcp-relay.md          # Детальная документация TCP Relay
├── api-reference.md      # Полный API reference (все эндпоинты)
├── security.md           # Security model, reverse proxy setup
└── development.md        # Build, test, project structure
```

**README.md содержит только:**
- Название + 1 строка описания
- Badges (CI, version)
- Скриншот
- Ключевые фичи (таблица 1-строчных описаний)
- Quick Start (3 команды)
- Install (ссылка на releases + 2-строчный пример)
- Ссылки на docs/

---

### Итоговая таблица Roadmap-2

| ID | Приоритет | Статус | Описание |
|----|-----------|--------|----------|
| RD-5 | 🔴 HIGH | ❌ Открыт | Переименование ZeroBox → ZTNetwork Panel |
| RD-6 | 🔴 HIGH | ❌ Открыт | Help-панели: Exit Node / L2 Bridge / TCP Relay |
| RD-7 | 🟡 MEDIUM | ❌ Открыт | Log Panel: zero-request при закрытой панели + Clear |
| RD-8 | 🟡 MEDIUM | ❌ Открыт | README.md compact + docs/ структура |

---

## Roadmap-2 (2026-04-24) — Branding, Help, Logs UX, Docs

### Сводная таблица

| # | Приоритет | Тип | Компонент | Задача | Статус |
|---|-----------|-----|-----------|--------|--------|
| RD2-1 | 🔴 HIGH | Branding | Frontend | Переименовать «ZeroBox» → «ZTNetwork Panel» везде в UI | ✅ `b17e586` |
| RD2-2 | 🔴 HIGH | UX/Docs | Frontend | Справка Exit Node: принцип работы, требования, пошаговая настройка | ✅ `b17e586` |
| RD2-3 | 🔴 HIGH | UX/Docs | Frontend | Справка L2 Bridge: принцип работы, требования, пошаговая настройка | ✅ `b17e586` |
| RD2-4 | 🔴 HIGH | UX/Docs | Frontend | Справка TCP Relay: принцип работы, требования, пошаговая настройка | ✅ `b17e586` |
| RD2-5 | 🔴 HIGH | UX | Frontend | Log Panel: кнопка очистки + не опрашивать API пока панель закрыта/логи выкл | ✅ `b17e586` |
| RD2-6 | 🟡 MEDIUM | Docs | README + docs/ | Переработать README.md: компактно, только общее описание + ссылки на docs/ | ✅ `b17e586` |
| RD2-7 | 🟡 MEDIUM | Docs | docs/ | Создать docs/: installation.md, configuration.md, features/, development.md | ✅ `b17e586` |

---

### RD2-1 🔴 HIGH — Переименование «ZeroBox» → «ZTNetwork Panel»

**Проблема:** название «ZeroBox» не отражает назначение продукта и конфликтует с другими проектами.

**Затронутые места:**
```
www/src/html/shell.html   — sidebar logo text, mobile-bar-title
www/src/js/z-init.js      — fallback title в _updateMobileTitle()
build.rs                  — <title> в HTML-шаблоне (если есть)
README.md                 — везде
docs/                     — везде
```

**Что заменить:**
- `ZeroBox` → `ZTNetwork Panel`
- `ZeroTier UI` (подзаголовок) → `ZeroTier Management Panel`
- HTML `<title>` тега → `ZTNetwork Panel`

---

### RD2-2 🔴 HIGH — Справка Exit Node

**Требование:** кнопка «?» или сворачиваемая секция «Help» на странице Exit Node  
с объяснением принципа работы и требований перед включением.

**Содержание справки:**

#### Как работает Exit Node
Exit Node направляет весь интернет-трафик участников ZeroTier-сети через этот хост.  
Участники устанавливают маршрут по умолчанию `0.0.0.0/0` через ZeroTier-адрес этого узла.

```
ZT участник ──→ [ZeroTier] ──→ Exit Node ──→ [NAT/masquerade] ──→ Интернет
```

#### Требования для работы
- ОС: **Linux** (iptables ≥ 1.8 или nftables ≥ 0.9)
- ZeroTier One ≥ 1.10 запущен
- Интерфейс WAN: реальный физический или виртуальный NIC с доступом в интернет
- Интерфейс ZT: `zt*` интерфейс сети, через которую придёт трафик
- Права: `root` или `CAP_NET_ADMIN + CAP_NET_RAW`
- В ZeroTier Central для нужной сети: включить `allowDefault` для этого участника

#### После включения (в ZeroTier Central)
1. Открыть сеть → вкладка «Members»
2. Найти этот узел → Advanced → поставить ✓ **Allow Default Route Override**
3. Участники сети должны получить маршрут `0.0.0.0/0` автоматически

---

### RD2-3 🔴 HIGH — Справка L2 Bridge

**Содержание справки:**

#### Как работает L2 Bridge
Подключает физическую Ethernet-сеть (LAN) к виртуальной ZeroTier-сети на уровне L2.  
Физические устройства получают ZT-адреса и появляются в ZT-сети как обычные участники.

```
Физическое LAN (192.168.1.x) ←── Bridge (ebtables/brctl) ──→ ZeroTier сеть
Устройства без ZT-клиента ─────────────────────────────────→ видны в ZT
```

#### Требования для работы
- ОС: **Linux**
- Пакеты: `bridge-utils` (`brctl`) + `ebtables`
- ZeroTier One ≥ 1.10 запущен
- В ZeroTier Central: включить **Bridging** для этого участника (вкладка Members → Advanced)
- Права: `root`

#### После включения (в ZeroTier Central)
1. Открыть сеть → вкладка «Members»
2. Найти этот узел → Advanced → поставить ✓ **Allow Bridging**
3. В сети добавить IP-пул из диапазона физической LAN

---

### RD2-4 🔴 HIGH — Справка TCP Relay (Pylon)

**Содержание справки:**

#### Как работает TCP Relay
Развёртывает [ZeroTier Pylon](https://github.com/zerotier/pylon) на удалённом сервере —  
TCP-ретранслятор для участников ZT за строгим NAT или файрволом (только порт 443/tcp открыт).

```
ZT узел за NAT ──→ TCP:443 ──→ [Pylon на VPS] ──→ ZeroTier root
```

Pylon запускается в Docker-контейнере на VPS через SSH-подключение с этого хоста.

#### Требования для работы
- **Локально:** SSH-ключ (`~/.ssh/id_ed25519` или другой RSA/ED25519)
- **Удалённо (VPS):**
  - SSH-доступ (ключ добавлен в `authorized_keys`)
  - Docker установлен или будет установлен автоматически
  - Открытый входящий TCP-порт (по умолчанию 443)
- Рекомендуемые ОС VPS: Ubuntu 22.04+, Debian 12+

#### Процесс деплоя
1. Указать хост (IP/hostname) и SSH-ключ
2. ztnet-box подключается по SSH и запускает Docker-контейнер с Pylon
3. Pylon-адрес прописывается в `local.conf` как `tcpFallbackRelay`
4. ZeroTier-клиент начинает использовать Pylon при недоступности UDP

---

### RD2-5 🔴 HIGH — Log Panel: очистка + умолчания

**Текущее поведение:**  
Log Panel опрашивает `/api/logs/stream` (SSE) при открытии приложения,  
даже если панель закрыта.

**Требуемое поведение:**
1. **По умолчанию** — панель закрыта, логи **выключены** (`level = off`)
2. **SSE-подключение** открывается только после нажатия кнопки **▶ Play**
3. **Кнопка «Clear»** — очищает текущий буфер логов в UI (не на сервере)
4. **Кнопка ■ Stop** — закрывает SSE-подключение, перестаёт получать новые логи
5. При закрытии панели — SSE отключается автоматически

**Изменения в коде:**
```
www/src/js/components/log-panel.js:
  - _stream: null по умолчанию (не открывать при init)
  - _play(): открывает EventSource, ставит level из <select>
  - _stop(): закрывает EventSource, не меняет level
  - _clear(): очищает DOM #log-entries, сбрасывает счётчик
  - init(): НЕ вызывает _play() автоматически

src/server/handlers/logs.rs:
  - GET /api/logs/level — текущий уровень (для синхронизации select)
  - Уровень по умолчанию: 'off' если панель не активна
```

---

### RD2-6 🟡 MEDIUM — Переработать README.md

**Текущее состояние:** README.md содержит подробные технические детали, которые плохо обновляются.

**Целевой README.md (компактный, ~80–120 строк):**

```markdown
# ZTNetwork Panel
> ZeroTier management UI — self-hosted web panel for ZeroTier One

[Badges: CI | Version | License]

## What is it?
Short 2-3 sentence description.

## Quick Start
- Requirements
- One-liner install
- Docker option (if any)

## Features (short bulleted list)

## Documentation
→ [Installation](docs/installation.md)
→ [Configuration](docs/configuration.md)
→ [Exit Node setup](docs/features/exit-node.md)
→ [L2 Bridge setup](docs/features/l2-bridge.md)
→ [TCP Relay setup](docs/features/tcp-relay.md)
→ [Development](docs/development.md)

## License
```

---

### RD2-7 🟡 MEDIUM — Создать docs/ структуру

```
docs/
├── installation.md       — требования, сборка из исходников, конфиг
├── configuration.md      — config.yml справочник всех параметров
├── features/
│   ├── exit-node.md      — детальное руководство Exit Node
│   ├── l2-bridge.md      — детальное руководство L2 Bridge
│   ├── tcp-relay.md      — детальное руководство TCP Relay
│   └── controller.md     — управление локальным ZT-контроллером
├── development.md        — сборка, тесты, архитектура, как добавить фичу
└── screenshots/          — PNG скриншоты UI (генерируются workflows)
```

Каждый файл должен содержать:
- Краткое описание что это
- Требования (ОС, пакеты, права)
- Пошаговая инструкция
- Примеры конфигов / команд
- Troubleshooting (типичные ошибки)


---

## Audit-4 — UX & Deserialization Bugs (2026-04-25)

| # | Priority | Type | Issue | Status |
|---|----------|------|-------|--------|
| A4-1 | 🔴 Critical | Backend | `Dns` struct missing `#[serde(default)]` → ZT returns `{}` → deserialization failure on join/create | ✅ Fixed |
| A4-2 | 🔴 Critical | Backend | `account_status()` called `GET /status` (server info) instead of `GET /self` (user/plan info) → token validation crash | ✅ Fixed |
| A4-3 | 🟡 Medium | UX | `NO_ACTIVE_TOKEN` error shows raw string; no call-to-action to add a token | ✅ Fixed — `ERR_NO_ACTIVE_TOKEN` variant + `errToast()` with Settings link |
| A4-4 | 🟡 Medium | UX | New Network dialog: `Modal.confirm()` binary yes/no → unclear; users don't know if Cancel = Central | ✅ Fixed — `Modal.choice()` with labelled cards: ZT Local / ZT Central |
| A4-5 | 🟡 Medium | UI | Mobile layout overflow, forms not full-width, tables not scrollable, metric cards cramped | ✅ Fixed — extended `@media (max-width: 768px)` + `@media (max-width: 400px)` |

---

## Roadmap-2 — Реализация (коммит `b17e586`, 2026-04-25)

### RD2-1 ✅ — Переименование «ZeroBox» → «ZTNetwork Panel»

**Файлы:** `www/src/html/shell.html`, `www/src/js/z-init.js`, `build.rs`

Все вхождения строки «ZeroBox» заменены на «ZTNetwork Panel»:
- Sidebar logo text и `<title>` в `build.rs`
- Заголовок мобильного бара в `shell.html`
- Fallback-значение в `_updateMobileTitle()` в `z-init.js`

---

### RD2-2 ✅ — Справка Exit Node

**Файл:** `www/src/js/pages/exitnode.js`

Добавлен коллапсируемый блок `<div class="help-box hidden">` под заголовком страницы.
Кнопка `? Help` в `page-header` переключает класс `hidden`.

Содержимое: что такое Exit Node, требования (Linux, nftables/iptables, root, WAN-интерфейс),
пошаговая настройка (5 шагов: deps → интерфейсы → Enable → Allow Default Route в контроллере → Default Route на девайсах).

---

### RD2-3 ✅ — Справка L2 Bridge

**Файл:** `www/src/js/pages/bridge.js`

Аналогичный коллапсируемый help-блок на странице L2 Bridge.

Содержимое: принцип работы (Layer 2 forwarding, ARP/DHCP прозрачны), требования
(systemd-networkd, iproute2, без dhcpcd/ifupdown конфликтов, root), 6-шаговая инструкция
включая обязательный шаг «Enable Active Bridge» в контроллере.

---

### RD2-4 ✅ — Справка TCP Relay

**Файл:** `www/src/js/pages/relay.js`

Однострочный info-баннер заменён полноценным коллапсируемым help-блоком.

Содержимое: как работает Pylon (зашифрованный TCP-relay), когда нужен (статус RELAY,
заблокированный UDP, только 443/TCP), два пути установки (через UI → Deploy и вручную),
предупреждение о влиянии Force TCP на производительность.

---

### RD2-5 ✅ — Log Panel: ленивая загрузка + SSE стоп при закрытии

**Файл:** `www/src/js/components/log-panel.js`

**Проблема:** при старте приложения `_loadInitial()` безусловно делал `GET /logs?limit=200`
даже когда панель логов была закрыта. SSE-стрим (`EventSource`) продолжал работать
пока панель была закрыта — бесполезный постоянный коннект к серверу.

**Исправление:**
- `_loadInitial()`: запрос истории логов пропускается если `_open === false`
- `_toggle()` при закрытии: вызывает `_stopStream()` — убивает `EventSource`
- `_toggle()` при открытии: если `_entries.length === 0` — загружает историю lazily,
  иначе просто рендерит накопленное

---

### RD2-6 ✅ — Компактный README.md

**Файл:** `README.md`

412 строк → 62 строки. Убраны: подробные инструкции по установке, полная таблица конфигурации,
длинные описания фич, примеры команд. Осталось: однострочный запуск, таблица фич (эмодзи + описание),
Quick Start (3 команды), таблица ссылок на `docs/`, секция License.

---

### RD2-7 ✅ — Структура docs/

**Новые файлы:**

| Файл | Содержимое |
|------|-----------|
| `docs/installation.md` | Бинарник, сборка из исходников, systemd unit, примечание о sudo |
| `docs/configuration.md` | Все поля `config.yml` с defaults, типами и описаниями |
| `docs/development.md` | Build/run/test команды, CI-таблица, структура проекта (дерево `src/` и `www/`) |
| `docs/features/exit-node.md` | Полное руководство: принцип, требования, setup, NDP Proxy |
| `docs/features/l2-bridge.md` | Принцип Layer 2, setup, флаг Active Bridge в контроллере |
| `docs/features/tcp-relay.md` | Pylon, оба пути деплоя (UI + ручной), Force TCP предупреждение |

---

## Аудит по скриншотам — SCR2 (коммиты `ba56c64`, `(текущий)`, 2026-04-25)

**Дата:** 2026-04-25  
**Источник:** анализ скриншотов после успешного запуска Playwright (PR #22, v0.12.0)

### Итоговая таблица SCR2

| # | Приоритет | Проблема | Коммит |
|---|-----------|----------|--------|
| SCR2-1 | 🔴 Critical | CSS `.hidden` не определён → help-боксы всегда открыты | ✅ `ba56c64` |
| SCR2-2 | 🟡 Medium | L2 Bridge: дублирование — старый `infoBox` + новый help-блок | ✅ `ba56c64` |
| SCR2-3 | 🟡 Medium | Peers: версия планет показывала `-1.-1.-1` вместо `—` | ✅ `ba56c64` |
| SCR2-4 | 🟡 Medium | Dashboard mobile: `node-status-bar` переполняется, таблица обрезается | ✅ `ba56c64` |
| SCR2-5 | 🟢 Low | Sidebar subtitle `ZeroTier UI` → `ZeroTier Web UI` | ✅ `ba56c64` |
| SCR2-6 | 🟡 Medium | Sidebar: метка секции `NODE` избыточна (один пункт — Dashboard) | ✅ (этот коммит) |
| SCR2-7 | 🟡 Medium | Controllers sidebar: `Members` подсвечивается одновременно с `Networks` | ✅ (этот коммит) |
| SCR2-8 | 🟢 Low | Physnet: нет кнопки `? Help` — несоответствие Exit Node / Bridge / Relay | ✅ (этот коммит) |

### Детали SCR2-6..8

**SCR2-6** — Убрана метка `Node` из sidebar. Секция содержала один пункт (Dashboard).
Метка `NODE` занимала вертикальное пространство и не несла информации.

**SCR2-7** — `Members` nav-item имел `data-route="/controllers/networks"` — совпадал с `Networks`.
Роутер подсвечивал оба пункта одновременно при открытии Controller Networks.
Исправлено: `data-route="/controllers/members"` (уникальный маршрут, который никогда
не совпадает с `/controllers/networks`), `onclick` оставлен на `/controllers/networks`
(Members открывается из контекста конкретной сети).

**SCR2-8** — Physnet был единственной Gateway-страницей без кнопки `? Help`.
Добавлены: кнопка в `page-header`, коллапсируемый help-блок с описанием NAT/L3 подхода,
требованиями, 4-шаговой инструкцией и ссылкой на официальную документацию ZeroTier.
Старый однострочный баннер убран.

---

## Roadmap-3 (2026-04-26) — Dashboard, Root Servers, Логирование

---

### RD3-1 🔴 HIGH — Dashboard: информация о сетях с привязкой к контроллеру

**Файл:** `www/src/js/pages/dashboard.js`

**Ответ на вопрос "join — local или central?":**  
ZeroTier сам определяет маршрут. `POST /api/local/networks/:id` всегда идёт через локального ZT-демона. Если сеть принадлежит локальному контроллеру (первые 10 символов ID = адрес ноды) — она отображается как **Local Controller** сеть. Если нет — это либо ZeroTier Central сеть, либо сеть стороннего контроллера. Тип определяется по наличию `controller` объекта в ответе `GET /api/local/networks/:id`.

**Что нужно добавить в Dashboard:**

1. **Тип сети в карточке** — бейдж: `LOCAL CTRL` / `CENTRAL` / `EXTERNAL`  
   Логика: если `net.id.startsWith(nodeAddress)` → LOCAL CTRL; если есть active Central токен и сеть есть в Central списке → CENTRAL; иначе → EXTERNAL

2. **Assigned IPs** — уже показываются, но нужно скопировать по клику (copy-on-click)

3. **Участники** — текущая реализация пытается запросить `/local/controller/networks/:id/members` и `/central/networks/:id/members`. Нужно добавить fallback: если ни один не ответил → показывать `—` вместо пустоты

4. **Status подключения** — поле `status` из `/local/networks/:id`:  
   `OK` → зелёный, `ACCESS_DENIED` → красный с подсказкой "Awaiting authorization", `NOT_FOUND` → серый "Network not found"

5. **Кнопка Details** → переход на `/networks/:id`, кнопка **Leave** с confirm

**Текущие проблемы в реализации:**
- `renderNetworkCard` получает `net` из `/api/local/networks` (список), но `status` и `type` нужно брать из `/api/local/networks/:id` (детальный). Либо обогащать список дополнительными запросами, либо использовать поля которые уже есть в listNetworks ответе
- Member dots: `lastOnline` в секундах (Unix timestamp), а не миллисекундах — нужна проверка масштаба

---

### RD3-2 🔴 HIGH — Root Servers: полная переработка страницы

**Файл:** `www/src/js/pages/settings-roots.js`

**Контекст из официальной документации:**

Мoons — удобный способ добавить пользовательские корневые серверы в пул. Пользователи могут создавать moons чтобы снизить зависимость от инфраструктуры ZeroTier Inc. или разместить root-серверы ближе для лучшей производительности.

Mobile clients должны быть настроены через UI или через специально форматированный URL. В обоих случаях необходимо base64-кодировать бинарный planet file.

Custom root servers (planets) являются deprecated. Moons — рекомендуемый способ добавления собственных root-серверов при этом сохраняя совместимость с публичными планетами ZeroTier.

**Что нужно реализовать:**

#### Секция 1: Текущие Planets (публичные корни)
- Таблица с текущими PLANET-пирами из `/api/local/peers` (filter role=PLANET)
- Показывать: address, IP, latency, version
- Это уже есть в Dashboard peers — нужно перенести/переиспользовать

#### Секция 2: Moons (орбитирование) — уже частично реализовано
- Список орбитируемых moons из `/api/local/moons`
- Поля: world ID, timestamp, количество roots, стабильные endpoints
- Форма: World ID + Seed ID → `POST /api/local/moons/:world_id`
- Удаление: `DELETE /api/local/moons/:world_id`

#### Секция 3: Planet File (НОВОЕ — не реализовано в backend)
Необходимо base64-кодировать бинарный planet file для использования на мобильных клиентах.

**Нужен новый backend handler:**
- `GET /api/system/planet-file` — читает `/var/lib/zerotier-one/planet` → возвращает base64
- `POST /api/system/planet-file` — принимает base64, сохраняет как бинарный файл, перезапускает zerotier
- `DELETE /api/system/planet-file` — удаляет кастомный planet file (восстанавливает дефолтный), перезапускает

**UI для Planet File:**
- Область для вставки base64 текста
- Кнопка загрузки файла (через `<input type="file">`)
- Отображение текущего planet file: hash/fingerprint, дата изменения, source (custom/default)
- **QR-код** для мобильных клиентов: генерировать QR из base64 строки `zerotier://join?planet=<base64>` — используя библиотеку `qrcode` (CDN) прямо в браузере
- Кнопка копирования base64

#### Секция 4: Создание собственного Moon (деплой) (НОВОЕ)
Для создания moon нужен `zerotier-idtool` для генерации world definition. Результат — `.moon` файл, который нужно разместить в `moons.d` директории.

**Минимальный UI:**
- Форма: IP-адрес сервера, порт (default 9993)
- Кнопка **Generate Moon** → вызов backend который запускает `zerotier-idtool` и создаёт `.moon` файл
- Показывать World ID и ссылку на скачивание сгенерированного файла
- Инструкция: куда положить файл на root-сервере

**Новые backend эндпоинты (требуют реализации в Rust):**
```
GET  /api/system/planet-file          → { base64: string, is_custom: bool, modified: datetime }
POST /api/system/planet-file          → body: { base64: string }
DELETE /api/system/planet-file        → сбрасывает к дефолту
POST /api/system/generate-moon        → body: { ip: string, port: u16, identity?: string }
                                       → { world_id: string, moon_file_base64: string }
```

---

### RD3-3 🟡 MEDIUM — Полный аудит логирования

**Результат аудита:**

Система логирования (`tracing` + кастомный `LogCollector` layer) полностью настроена, SSE-стрим работает, Log Panel в UI функционирует.

**Проблема: 13 из 14 handler-файлов не содержат ни одного `tracing::` вызова.**

| Файл | Вызовов tracing | Критические операции без лога |
|------|----------------|-------------------------------|
| `bridge.rs` | 0 | enable/disable bridge |
| `central.rs` | 0 | create/delete network, update member |
| `config.rs` | 0 | save global config |
| `exitnode.rs` | 0 | enable/disable exit node |
| `local.rs` | 0 | join/leave network, orbit/deorbit moon |
| `local_config.rs` | 0 | update ZT local config |
| `ndp.rs` | 0 | enable/disable NDP proxy |
| `physnet.rs` | 0 | enable/disable physical routing |
| `relay.rs` | 1 | deploy pylon ✓, но нет для local relay config |
| `system.rs` | 0 | ZT install, zt-status |
| `tokens.rs` | 0 | add/delete/activate token |
| `metrics.rs` | 0 | (read-only, нижний приоритет) |

**Что нужно добавить** — `tracing::info!` на каждую мутирующую операцию:

```rust
// Пример для local.rs
pub async fn join_network(...) {
    tracing::info!(network_id = %id, "joining ZeroTier network");
    // ...
    tracing::info!(network_id = %id, "joined network successfully");
}

pub async fn orbit_moon(...) {
    tracing::info!(world_id = %world_id, "orbiting moon");
}
```

**Приоритет по критичности:**
1. `exitnode.rs` — enable/disable (системные операции с firewall)
2. `bridge.rs` — enable/disable (системные операции с сетевыми интерфейсами)
3. `physnet.rs` — enable/disable
4. `tokens.rs` — add/delete/activate (безопасность)
5. `local.rs` — join/leave network, orbit/deorbit moon
6. `central.rs` — create/delete/update network и members
7. `config.rs` — save config
8. `system.rs` — ZT install

**Также:**
- Frontend: Log Panel не показывает source/target операции. Нужно добавить `tracing::info!` с полями (`network_id`, `node_id`, `action`) чтобы Log Panel был полезен а не только показывал HTTP-запросы

---

### Итоговая таблица Roadmap-3

| # | Приоритет | Компонент | Задача | Статус |
|---|-----------|-----------|--------|--------|
| RD3-1 | 🔴 High | Frontend | Dashboard: тип сети (LOCAL/CENTRAL/EXTERNAL), status ACCESS_DENIED, copy IP | 🔲 |
| RD3-2a | 🔴 High | Frontend + Backend | Root Servers: Planet File UI (upload/download/QR) | 🔲 |
| RD3-2b | 🔴 High | Backend | Root Servers: новые эндпоинты /system/planet-file, /system/generate-moon | 🔲 |
| RD3-2c | 🟡 Medium | Frontend | Root Servers: текущие Planets из peers + Moon deployment guide | 🔲 |
| RD3-3 | 🟡 Medium | Backend (Rust) | Логирование: `tracing::info!` во все 13 handler-файлов на мутирующие операции | 🔲 |

## Audit-5 — ZeroTier API соответствие официальной документации (2026-04-27)

**Источники:**
- [ZeroTier Client/Service API](https://github.com/zerotier/zerotier-one-api-spec/blob/main/main.tsp) (официальный TypeSpec)
- [ZeroTier Legacy Central API](https://docs.zerotier.com/api/central/legacy/) + реальные ответы API

### 🔴 Критические — ломают функциональность

| ID | Приоритет | Файл | Описание | Статус |
|----|-----------|------|----------|--------|
| ZT-C-2 | ✅ Fixed | `local/types.rs` | `NetworkMembership.dns` — ZT возвращает `[]` (пустой массив) для пустого DNS, текущий `Option<Dns>` не обрабатывает массив → deserialization error | 🔲 |
| ZT-C-6 | ✅ Fixed | `local/types.rs` | `ControllerMember` — локальный контроллер возвращает поле `"id"` (адрес ноды), а не `"nodeId"` (Central). Текущий `#[serde(rename="nodeId")]` не работает для local controller | 🔲 |
| ZT-C-7 | ✅ Fixed | `local/types.rs` | `V6AssignMode.plan6` — опечатка, должно быть `"6plane"` (spec: `` `6plane`?: boolean ``). Поле никогда не десериализуется | 🔲 |
| ZT-C-11 | ✅ Fixed | `central/types.rs` | `CentralMember` — Legacy Central API оборачивает мутируемые поля в `config: {...}`. Текущая структура читает `authorized`, `ipAssignments` и др. напрямую → поля всегда пусты/дефолтны | 🔲 |
| ZT-C-12 | ✅ Fixed | `central/types.rs` | `CentralMemberUpdate` — обновление члена требует тела `{"config": {...}}`. Текущая структура отправляет поля напрямую → сервер игнорирует изменения | 🔲 |
| ZT-C-13 | ✅ Fixed | `central/client.rs` | `user()` использует `GET /auth` — несуществующий endpoint. Правильный путь: `GET /self` | 🔲 |
| ZT-C-14 | ✅ Fixed | `central/types.rs` | `AccountStatus.planType` — поле не существует в ответе `GET /self`. Тарификация определяется через `subscriptions`, не `planType` → rate limit всегда Free | 🔲 |

### 🟡 Высокие — некорректное поведение

| ID | Приоритет | Файл | Описание | Статус |
|----|-----------|------|----------|--------|
| ZT-C-1 | ✅ Fixed | `local/types.rs` | `NodeStatus.world_id` → должно быть `planetWorldId`. Нет полей: `versionMajor/Minor/Rev/Build`, `config.settings.primaryPort`, `config.settings.surfaceAddresses` | 🔲 |
| ZT-C-3 | ✅ Fixed | `local/types.rs` | `NetworkMembership` — отсутствуют поля: `portDeviceName` (имя TUN-интерфейса), `multicastSubscriptions`, `authenticationURL`, `authenticationExpiryTime` | 🔲 |
| ZT-C-4 | ✅ Fixed | `local/types.rs` | `PeerInfo` — нет поля `tunneled: bool`. `PeerPath` — нет `localSocket: u64`. Тип `latency: i32` должен быть `i64` (spec: `uSafeint \| -1`) | 🔲 |
| ZT-C-5 | ✅ Fixed | `local/types.rs` | `ControllerNetwork` — отсутствуют: `nwid`, `objtype`, `revision`, `capabilities`, `rules`, `tags`. Без `nwid` нельзя получить сеть по дублированному ID | 🔲 |
| ZT-C-8 | ✅ Fixed | `local/types.rs` | `ControllerNetworkCreate` слишком ограничен — нет полей для `ipAssignmentPools`, `routes`, `mtu`, `v4AssignMode`, `v6AssignMode`, `multicastLimit` | 🔲 |
| ZT-C-9 | 🟡 High | `local/client.rs` | `network_members()` — N+1 запросов: `GET /controller/network/{id}/member` → N×`GET /controller/network/{id}/member/{node_id}`. При 100+ членах — 100+ последовательных HTTP запросов | 🔲 |
| ZT-C-10 | 🟡 High | `central/types.rs` | `CentralNetwork` — отсутствуют: `totalMemberCount`, `config.creationTime`, `config.lastModified`, `config.id`, `config.capabilities`, `config.rules`, `config.tags` | 🔲 |

### 🟢 Низкие — улучшение соответствия

| ID | Приоритет | Файл | Описание | Статус |
|----|-----------|------|----------|--------|
| ZT-M-1 | 🟢 Low | `local/client.rs` | Auth header: `X-ZT1-Auth` → официальный стандарт `X-ZT1-AUTH` (ZT принимает оба, но spec использует верхний регистр) | 🔲 |
| ZT-M-2 | 🟢 Low | `local/client.rs` | `create_controller_network` использует `getrandom` локально — ZT поддерживает `POST /controller/` для генерации случайного сетевого ID на сервере | 🔲 |
| ZT-M-3 | 🟢 Low | `local/client.rs` | `leave_network` — DELETE `/network/{id}` возвращает `{"result": true}`. Текущий `request_empty()` игнорирует тело и не различает успех от ошибки по телу ответа | 🔲 |
| ZT-L-4 | 🟢 Low | `local/client.rs` | Добавить поддержку `GET /unstable/controller/network/{id}/member` для получения полного списка членов одним запросом (вместо N+1) | 🔲 |

### Примечания

**ZT-C-11/12 (CentralMember/Update)** — наиболее критические для работы приложения. Все операции с членами сети через Central API (авторизация, смена IP, мосты) не работают из-за отсутствия обёртки `{"config": {...}}`.

**ZT-C-7 (V6AssignMode)** — поле `plan6` является опечаткой вместо `6plane`. В Rust нельзя использовать `6plane` как имя поля (начинается с цифры), поэтому требуется `#[serde(rename = "6plane")]`:
```rust
#[serde(rename = "6plane", default)]
pub six_plane: bool,
```

**ZT-C-14 (planType)** — `GET /self` возвращает объект `subscriptions` из которого нужно извлечь тип плана:
```json
{"subscriptions": {"zerotier": {"planId": "paid", ...}}}
```
Текущая логика определения тарифа через `planType` всегда возвращает `Free`.


---

## Аудит-6 (2026-04-28) — UI/UX баги из ручного тестирования

**Дата:** 2026-04-28  
**Источник:** ручное тестирование через cloudflared на мобильном устройстве

### Итоговая таблица

| # | Приоритет | Компонент | Проблема | Статус |
|---|-----------|-----------|----------|--------|
| UX-1 | 🔴 High | Frontend | QR-код на странице Root Servers не отображается | 🔲 |
| UX-2 | 🔴 High | Frontend / Backend | Log Panel переполнена `info ztnet_box::server::middleware` — HTTP access-логи засоряют вывод; отсутствует горизонтальный скроллинг длинных строк | ✅ |
| UX-3 | 🔴 High | Frontend | Отсутствует возможность отключения/удаления (leave) сетей на странице My Networks | ✅ |
| UX-4 | 🔴 High | Frontend | My Networks: сеть Central отображается как локальная (`Local  45b6e887e21780b3  LANFriendly  OK`), вкладка «Central» пуста | ✅ |
| UX-5 | 🔴 High | Frontend | Раздел Controllers: страницы Networks и Members избыточно разделены — нужно объединить в одну страницу (список сетей + inline таблица членов при выборе) | ✅ |
| UX-6 | 🔴 High | CSS | Отсутствует мобильная адаптация по ширине и высоте: контент выходит за экран, горизонтальный скролл, переполнение таблиц | ✅ |
| UX-7 | 🟡 Medium | Frontend | Непоследовательная структура страниц: у каждой страницы свой layout — нет единого паттерна для заголовка, кнопок действий, текстовых полей | 🔲 |

---

### Детали задач

#### UX-1 — QR-код Root Servers не отображается

**Симптом:** QR-код для planet file (мобильные клиенты) не рендерится.  
**Вероятная причина:** библиотека `qrcode` не загружается с CDN, или base64 планеты пуста/не запрошена.  
**Что сделать:**
- Проверить подключение `qrcodejs` через CDN в `settings-roots.js`
- Добавить fallback: если QR не рендерится — показывать текст base64 в `<textarea>` для ручного копирования
- Добавить индикатор ошибки если библиотека не загрузилась

---

#### UX-2 — Log Panel: middleware-логи и отсутствие скролла

**Симптом:** Log Panel заполнена сотнями строк вида `info ztnet_box::server::middleware — GET /api/local/peers 200 OK 12ms`. Они не несут пользовательской ценности и скрывают реальные события.  
**Вторая проблема:** длинные строки не обрезаются и не скроллятся горизонтально — ломают layout.

**Что сделать:**
1. **Backend** — middleware access-логи исключить из `LogCollector`. Использовать отдельный `tracing::Span` с полем `skip = true` в `LogCollectorLayer`, или снизить уровень middleware до `trace` (выше `info` по умолчанию):
   ```rust
   // tower_http TraceLayer с уровнем trace вместо info
   TraceLayer::new_for_http()
       .make_span_with(|_req: &_| tracing::trace_span!("http"))
       .on_response(|_res: &_, _latency: _, _span: &_| {})
   ```
2. **Frontend** — добавить `overflow-x: auto; white-space: pre; word-break: break-all` к строкам лог-панели
3. **Frontend** — добавить фильтрацию по `target` (namespace): кнопки `All | App | HTTP` — скрывать строки где `target` содержит `middleware`

---

#### UX-3 — Отсутствует Leave/Delete для сетей

**Симптом:** на странице My Networks нет кнопки покинуть сеть (leave). Пользователь может посмотреть список, но не может отключиться.  
**Что сделать:** добавить кнопку «Leave» (с confirm-диалогом) для каждой строки/карточки сети. Вызывает `DELETE /api/local/networks/:id`.

---

#### UX-4 — My Networks: неправильная классификация сети (Central показывается как Local)

**Корневая причина:** страница получает список из `GET /api/local/networks` (все сети, к которым подключён ZT-демон), но не различает их по типу. ZeroTier не возвращает в списке информацию об источнике (Central/Local/External).

**Алгоритм определения типа:**
1. `LOCAL CONTROLLER` — если `net.id.substring(0,10) === nodeAddress` (сеть создана локальным контроллером этого узла)
2. `CENTRAL` — если в активном токене Central API есть эта сеть в `GET /api/central/networks`
3. `EXTERNAL` — всё остальное

**Что сделать:**
- При загрузке My Networks: параллельно запросить `GET /api/local/networks` + `GET /api/central/networks` (если есть токен)
- Пересечение по ID → тип `CENTRAL`
- Вкладки `All | My Networks (local) | Central` фильтруют по вычисленному типу
- Показывать бейдж типа на каждой карточке/строке

---

#### UX-5 — Controllers: объединить Networks + Members в одну страницу

**Текущее состояние:** два отдельных пункта меню — `Networks` и `Members`. `Members` ведёт обратно на `Networks`. Это создаёт путаницу и лишние клики.

**Что сделать:**
- Убрать пункт `Members` из sidebar (уже частично сделано в аудите-2)
- Страница `/controllers/networks`: список сетей контроллера в виде таблицы-аккордеона или split-view
  - Клик на сеть → расширяет строку inline и показывает таблицу членов
  - Или: правая панель (desktop) / отдельный экран (mobile) с членами выбранной сети
- Кнопки управления членами (authorize/revoke/delete) прямо в inline таблице

---

#### UX-6 — Мобильная адаптация

**Симптомы:** таблицы шире экрана, горизонтальный скролл всей страницы, кнопки обрезаются, формы не помещаются.

**Что сделать:**
1. **Таблицы** — `min-width: 0` на ячейки, `overflow-x: auto` на `.table-wrap`, скрывать второстепенные колонки на `< 480px` через `@media`
2. **Формы** — `.input-row` → `flex-direction: column` на `< 600px`
3. **Карточки сетей** — уже есть `.net-cards-grid { grid-template-columns: 1fr }` на `< 768px`, проверить что работает
4. **Log Panel** — на мобильных: ограничить высоту `max-height: 40vh`, добавить `overflow-y: auto`
5. **Page header** — кнопки и заголовок переносить в column на `< 480px`
6. **Sidebar** — проверить что hamburger-кнопка видна и работает на всех экранах

---

#### UX-7 — Единый layout паттерн для всех страниц

**Текущее состояние:** каждая страница имеет свою структуру. Dashboard отличается от Networks, Networks от Controllers, Controllers от Settings и т.д.

**Единый паттерн (ввести как стандарт):**
```
┌─────────────────────────────────────────────────────┐
│  Заголовок страницы          [Вторичная кнопка] [+Действие] │  ← .page-header
├─────────────────────────────────────────────────────┤
│  [Поиск/фильтр поле]         [Вкладки если нужны]   │  ← .page-toolbar (опционально)
├─────────────────────────────────────────────────────┤
│                                                      │
│  Основной контент (таблица / карточки / форма)       │
│                                                      │
└─────────────────────────────────────────────────────┘
```

**Правила:**
- Заголовок всегда слева, основное действие всегда справа в `.page-header`
- Кнопки `+ Создать / + Добавить` — всегда `.btn-primary` справа в заголовке
- Вторичные действия (Export, Refresh) — `.btn-ghost` левее primary
- Поиск/фильтр — в отдельной `.page-toolbar` строке под заголовком
- Пустое состояние — всегда `.empty-state` с иконкой + описание + CTA кнопка
- Состояние загрузки — всегда `.loading-row` со спиннером
- Ошибка — всегда `.banner.banner-danger` с текстом

**Страницы требующие рефакторинга layout:**
- `networks.js` — нет единого toolbar
- `controllers-networks.js` — заголовок и кнопки в разных местах
- `settings-global.js` — кнопка Save не в заголовке
- `settings-tokens.js` — кнопка Add Token иногда в двух местах
- `exitnode.js` — Dependencies и Configuration не в едином .section паттерне

---

## Audit-6 — QR-ошибки и единый layout (2026-04-30)

| ID | Приоритет | Компонент | Описание | Статус |
|----|-----------|-----------|----------|--------|
| UX-Q1 | 🔴 Critical | `qrcode.js` | QR error на Network Details: `finder()` separator loop выходил за границы матрицы (`M[size][7]`, `M[7][size]`) при финальных finder pattern'ах. Исправлено: двойная проверка `r+7 < size && c+i < size` и `c+7 < size && r+i < size` | ✅ Fixed |
| UX-Q2 | 🔴 Critical | `settings-roots.js` | QR-код не отображался на странице Root Servers — функция вообще не была реализована. Добавлена кнопка [QR] для каждого moon, панель `#moon-qr-panel` и вызов `QRCode.render(worldId, canvas)` | ✅ Fixed |
| UX-L1 | 🟡 Medium | Все страницы | Непоследовательный текст загрузки: `"Loading..."` vs `"Loading…"`. Стандартизировано: все страницы используют `"Loading…"` | ✅ Fixed |
| UX-L2 | 🟡 Medium | Все страницы | Повторяющийся inline HTML для состояния загрузки. Добавлены `Utils.pageLoading(title?)` и `Utils.pageError(msg)` в `state.js`. Все 12 страниц переведены на `Utils.pageLoading()` | ✅ Fixed |
| UX-L3 | 🟡 Medium | Многие страницы | `Toast.error(e.message)` не обрабатывает `ERR_NO_ACTIVE_TOKEN`. Стандартизировано: `exitnode.js`, `networks.js`, `physnet.js`, `relay.js`, `settings-global.js`, `settings-tokens.js` переведены на `errToast(e)` | ✅ Fixed |
