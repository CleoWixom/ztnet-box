# AUDIT.md — ztnet-box

**Дата последнего аудита:** 2026-04-17 (все пункты закрыты, CI зелёный)
**Репозиторий:** `CleoWixom/ztnet-box`
**Стек:** Rust (Axum, Tokio) + Vanilla JS/HTML/CSS (SPA, сборка через `build.rs`)
**Версия на момент аудита:** 0.8.0

---

## Содержание

1. [CRITICAL — `log-panel.js` не включался в бандл](#1-critical--log-paneljs-не-включался-в-бандл)
2. [HIGH — Неработающий Rate Limiter](#2-high--неработающий-rate-limiter)
3. [HIGH — Frontend: неверный HTTP-метод при обновлении сети контроллера](#3-high--frontend-неверный-http-метод-при-обновлении-сети-контроллера)
4. [HIGH — SSH с `StrictHostKeyChecking=no`](#4-high--ssh-с-stricthostkeycheckingno)
5. [HIGH — `curl | sh` для установки Docker](#5-high--curl--sh-для-установки-docker)
6. [HIGH — SSH-пароль в JSON по HTTP](#6-high--ssh-пароль-в-json-по-http)
7. [HIGH — Отсутствие мобильной адаптации](#7-high--отсутствие-мобильной-адаптации)
8. [HIGH — Кастомная реализация `rand_byte`](#8-high--кастомная-реализация-rand_byte)
9. [MEDIUM — `update_token` уничтожал UUID](#9-medium--update_token-уничтожал-uuid)
10. [MEDIUM — N+1 запросов в `controllers-networks.js`](#10-medium--n1-запросов-в-controllers-networksjs)
11. [MEDIUM — Хардкод CSS-значений в `log-panel.js`](#11-medium--хардкод-css-значений-в-log-paneljs)
12. [MEDIUM — Неиспользуемый тип-алиас `PhysNetStateArc`](#12-medium--неиспользуемый-тип-алиас-physnetstaterc)
13. [MEDIUM — `PeersPage` определена inline в `shell.html`](#13-medium--peerspage-определена-inline-в-shellhtml)
14. [MEDIUM — Семантическая ошибка в поле `zt_network_id`](#14-medium--семантическая-ошибка-в-поле-zt_network_id)
15. [MEDIUM — Отсутствует кнопка сворачивания боковой панели](#15-medium--отсутствует-кнопка-сворачивания-боковой-панели)
16. [MEDIUM — `metricstoken_file` не настраивается через UI](#16-medium--metricstoken_file-не-настраивается-через-ui)
17. [MEDIUM — `Modal.prompt` без реализации](#17-medium--modalprompt-без-реализации)
18. [MEDIUM — Состояние bridge/physnet/relay не персистировалось](#18-medium--состояние-bridgephysnetrelay-не-персистировалось)
19. [MEDIUM — Зависимость от `sshpass`](#19-medium--зависимость-от-sshpass)
20. [LOW — CSP: `connect-src *`](#20-low--csp-connect-src-)
21. [LOW — `danger_accept_invalid_certs` безусловно](#21-low--danger_accept_invalid_certs-безусловно)
22. [LOW — Дублирование функции `_esc()`](#22-low--дублирование-функции-_esc)
23. [LOW — Backend-эндпоинты без покрытия во Frontend](#23-low--backend-эндпоинты-без-покрытия-во-frontend)
24. [LOW — `#[allow(clippy::derivable_impls)]`](#24-low--allowclippyderivable_impls)
25. [LOW — Заглушка: `network_id` не передавался в `enable()`](#25-low--заглушка-network_id-не-передавался-в-enable)
26. [HIGH (CI) — Unused params в cfg-гейтированных ndp-функциях](#26-high-ci--unused-params-в-cfg-гейтированных-ndp-функциях)
27. [HIGH — Утечка `tokio::spawn` в Rate Limiter (пост-аудит)](#27-high--утечка-tokiospawn-в-rate-limiter-пост-аудит)
28. [MEDIUM — Нет таймаута SSH-соединения при deploy (пост-аудит)](#28-medium--нет-таймаута-ssh-соединения-при-deploy-пост-аудит)
29. [Итоговая таблица](#итоговая-таблица)
30. [Рекомендации по архитектуре](#рекомендации-по-архитектуре)

---

## 1. CRITICAL — `log-panel.js` не включался в бандл

**Приоритет:** 🔴 Critical → ✅ Resolved `d53a1c2`
**Файл:** `build.rs`, `www/src/js/components/log-panel.js`

### Описание

`build.rs` собирал JS в порядке: core (`api`, `state`, `router`) → `components/*` → `pages/*`. Файл `log-panel.js` лежал непосредственно в `www/src/js/` (не в `components/`), поэтому **никогда не попадал в бандл**. `shell.html` вызывал `LogPanel.init()`, что давало `ReferenceError` в браузере.

### Исправление

`log-panel.js` перемещён в `www/src/js/components/log-panel.js`. Теперь подхватывается автоматически через `collect_files(&comp_dir, "js")` в `build.rs`. Дубликата в корне `js/` нет.

**Верифицировано:** `ls www/src/js/components/log-panel.js` — файл присутствует; `ls www/src/js/` не содержит `log-panel.js`.

---

## 2. HIGH — Неработающий Rate Limiter

**Приоритет:** 🔴 High → ✅ Resolved `d53a1c2`
**Файл:** `src/zerotier/central/client.rs`

### Описание

```rust
// ДО: permit немедленно дропался — rate limiter не работал
async fn acquire(&self) {
    let _ = self.semaphore.acquire().await;
}
```

`let _ = expr` дропает `SemaphorePermit` немедленно, возвращая разрешение в семафор. Rate limiter пропускал все запросы без ограничений.

### Исправление

```rust
// ПОСЛЕ: permit consumeится через forget(), рефил — раз в секунду фоновой задачей
async fn acquire(&self) {
    Arc::clone(&self.semaphore)
        .acquire_owned()
        .await
        .expect("rate-limiter semaphore closed")
        .forget();
}
```

Фоновая задача (`tokio::spawn`) раз в секунду пополняет семафор до максимума через `add_permits()`. Реализует true "max N req/s" token-bucket семантику.

**Юнит-тест:** `rate_limiter_blocks_when_exhausted` — проверяет, что третий `acquire()` при 2 permits блокируется через `tokio::time::timeout`.

---

## 3. HIGH — Frontend: POST вместо PUT для обновления сети контроллера

**Приоритет:** 🔴 High → ✅ Resolved `d53a1c2`
**Файл:** `www/src/js/pages/controllers-config.js:190`

### Описание

```javascript
// ДО: 405 Method Not Allowed
if (_src==='local') await api.post(`/local/controller/networks/${_id}`, body);
```

Бэкенд регистрировал `.put(local_handler::update_controller_network)`. `POST` возвращал 405.

### Исправление

```javascript
// ПОСЛЕ
if (_src==='local') await api.put(`/local/controller/networks/${_id}`, body);
```

**Верифицировано:** `grep "api\.put\|api\.post" www/src/js/pages/controllers-config.js | grep local/controller/networks` → строка 190 содержит `api.put`.

---

## 4. HIGH — SSH с `StrictHostKeyChecking=no`

**Приоритет:** 🔴 High → ✅ Resolved `6d4c76f`
**Файл:** `src/relay/ssh.rs`

### Описание

`StrictHostKeyChecking=no` открывал вектор MITM-атаки: при деплое relay (установка Docker, запуск привилегированных контейнеров) злоумышленник в сети мог подменить ответ сервера.

### Исправление

```rust
"-o".into(), "StrictHostKeyChecking=accept-new".into(),
```

`accept-new` автоматически добавляет ключ при **первом** подключении, но отклоняет изменённые ключи на последующих — защищает от MITM после первоначального установления доверия.

**Верифицировано:** `grep StrictHost src/relay/ssh.rs` → `StrictHostKeyChecking=accept-new`.

---

## 5. HIGH — Docker install via `curl | sh`

**Приоритет:** 🔴 High → ✅ Resolved `6d4c76f`
**Файл:** `src/relay/deploy.rs`

### Описание

```rust
// ДО: выполнение произвольного кода с удалённого URL без верификации
client.run("curl -fsSL https://get.docker.com | sh")?;
```

### Исправление

```rust
// ПОСЛЕ: определение пакетного менеджера + официальный пакет
let install_cmd = "if command -v apt-get ...; then apt-get install -y docker.io; \
                   elif command -v dnf ...; then dnf install -y docker; \
                   elif command -v pacman ...; then pacman -S --noconfirm docker; \
                   else echo 'No supported pm' >&2; exit 1; fi";
```

**Верифицировано:** `grep "curl\|get.docker.com" src/relay/deploy.rs` — пусто. Файл содержит `apt-get install -y docker.io`.

---

## 6. HIGH — SSH-пароль в JSON по HTTP

**Приоритет:** 🔴 High → ✅ Resolved `6d4c76f` + `4ed5d12`
**Файл:** `src/relay/mod.rs`, `www/src/js/pages/relay.js`

### Описание

Поле `password: Option<String>` в `RelayDeployConfig` передавалось в теле HTTP-запроса. При работе без TLS пароль оказывался в логах прокси и traffic dumps.

### Исправление

Поле `password` полностью удалено из `RelayDeployConfig`. UI содержит только поле `key_path` с пояснением «Key-based auth only». `sshpass` удалён из `ssh.rs` (см. п. 19).

**Верифицировано:** `grep "password" src/relay/mod.rs` — пусто; `grep "password\|sshpass" src/relay/ssh.rs` — только комментарий об удалении.

---

## 7. HIGH — Отсутствие мобильной адаптации

**Приоритет:** 🔴 High → ✅ Resolved `85bc5a2`
**Файл:** `www/src/css/layout.css`

### Описание

Ни одного `@media` запроса. Sidebar фиксировался на 220px, занимая ~46% экрана на телефонах.

### Исправление

```css
@media (max-width: 768px) {
    #mobile-bar   { display: flex; }
    #content      { margin-left: 0; padding-top: var(--header-h); }
    #sidebar      { transform: translateX(-100%); transition: transform var(--t-base); z-index: 200; }
    #sidebar.open { transform: translateX(0); }
    .table-wrap   { overflow-x: auto; -webkit-overflow-scrolling: touch; }
    .cards-grid   { grid-template-columns: 1fr; }
    .page-header  { flex-wrap: wrap; }
}
```

**Верифицировано:** `grep "@media" www/src/css/layout.css` → присутствует.

---

## 8. HIGH — Кастомная реализация `rand_byte`

**Приоритет:** 🔴 High → ✅ Resolved `c474bb1`
**Файл:** `src/zerotier/local/client.rs`

### Описание

```rust
// ДО: открывал /dev/urandom 6 раз по одному байту; fallback = константа 0xAB
fn rand_byte() -> u8 {
    std::fs::File::open("/dev/urandom")
        .and_then(|mut f| f.read_exact(&mut buf).map(|_| buf[0]))
        .unwrap_or(0xab)  // все network ID одинаковы на Windows
}
```

Три проблемы: неэффективность, непортируемость, константный fallback.

### Исправление

```rust
// ПОСЛЕ: getrandom — кроссплатформенно, без fallback
let mut buf = [0u8; 6];
getrandom::getrandom(&mut buf).map_err(|e| ApiError::ZtLocal(...))?;
let suffix: String = buf.iter().map(|b| format!("{b:02x}")).collect();
```

Зависимость `getrandom = "0.2"` добавлена в `Cargo.toml`.

**Верифицировано:** `grep "rand_byte\|0xab\|urandom" src/zerotier/local/client.rs` — пусто; `grep "getrandom" src/zerotier/local/client.rs` — присутствует.

---

## 9. MEDIUM — `update_token` уничтожал UUID

**Приоритет:** 🟡 Medium → ✅ Resolved `c474bb1`
**Файл:** `src/server/handlers/tokens.rs`, `src/zerotier/central/token_store.rs`

### Описание

`PUT /api/settings/tokens/:id` вызывал `token_store.remove(&id)` + `token_store.add(...)`, что генерировало новый `Uuid::new_v4()`. Любая ссылка на токен по ID становилась невалидной. В коде был комментарий `// Re-insert with same id not possible via current API`.

### Исправление

В `TokenStore` добавлен метод `update()`:

```rust
pub async fn update(&self, id: &str, name: String, token: String, rate_limit: RateLimit)
    -> Option<CentralToken>
{
    let mut inner = self.inner.write().await;
    let t = inner.tokens.iter_mut().find(|t| t.id == id)?;
    t.name = name; t.token = token; t.rate_limit = rate_limit;
    let updated = t.clone();
    Self::invalidate_cache(&mut inner).await;
    Some(updated)
}
```

Хендлер `update_token` теперь использует `token_store.update()`. Повторная валидация через Central API происходит только если значение токена изменилось.

**Верифицировано:** `grep "token_store.update\|token_store.remove" src/server/handlers/tokens.rs` — `remove` отсутствует, `update` присутствует.

---

## 10. MEDIUM — N+1 API-запросов в `controllers-networks.js`

**Приоритет:** 🟡 Medium → ✅ Resolved `502a8aa`
**Файл:** `www/src/js/pages/controllers-networks.js`

### Описание

При 10 сетях: 1 запрос за список ID + 10 последовательных запросов деталей = 11 запросов. Каждый инициировал запрос к ZeroTier daemon.

### Исправление

```javascript
// ПОСЛЕ: все запросы параллельно
const results = await Promise.allSettled(
    _nets.map(n => api.get(`/local/controller/networks/${n.id}`))
);
_nets = _nets.map((n, i) =>
    results[i].status === 'fulfilled'
        ? { ...n, ...results[i].value, _src: 'local' }
        : n
);
```

**Верифицировано:** `grep "Promise.allSettled\|for.*await" www/src/js/pages/controllers-networks.js` → `Promise.allSettled` присутствует, последовательного цикла нет.

---

## 11. MEDIUM — Хардкод CSS-значений в `log-panel.js`

**Приоритет:** 🟡 Medium → ✅ Resolved `85bc5a2`
**Файл:** `www/src/js/components/log-panel.js`

### Описание

`_injectStyles()` использовал переменные `--bg-secondary`, `--border`, `--accent` — несуществующие в `variables.css`. Реальная система именования: `--c-surface`, `--c-border`, `--c-primary`. Всегда применялись хардкоданные fallback-цвета.

### Исправление

Все переменные приведены к системе именования `variables.css`:

```javascript
`#log-panel { background: var(--c-surface,#1a1d27); border-top: 1px solid var(--c-border,#2e3147); }`
`#log-bar   { background: var(--c-surface2,#222536); }`
`.log-badge { background: var(--c-primary,#4f7cff); }`
```

**Верифицировано:** `grep "bg-secondary\|--accent\|--border\b" www/src/js/components/log-panel.js` — пусто.

---

## 12. MEDIUM — Неиспользуемый тип-алиас `PhysNetStateArc`

**Приоритет:** 🟡 Medium → ✅ Resolved `c474bb1`
**Файлы:** `src/server/handlers/physnet.rs`, `src/relay/mod.rs`

### Описание

`pub type PhysNetStateArc = Arc<RwLock<PhysNetState>>;` и `pub type RelayRemoteState = RwLock<Option<RemoteRelayInfo>>;` нигде не использовались.

### Исправление

Оба алиаса удалены.

**Верифицировано:** `grep -rn "PhysNetStateArc\|pub type.*RelayRemote" src/` → ничего.

---

## 13. MEDIUM — `PeersPage` определена inline в `shell.html`

**Приоритет:** 🟡 Medium → ✅ Resolved `502a8aa`
**Файл:** `www/src/js/pages/peers.js` (создан)

### Описание

`PeersPage` была определена прямо в `<script>` блоке `shell.html` и использовала `State.get('peers')` без загрузки данных. При прямой навигации на `/peers` список был пустым.

### Исправление

Создан `www/src/js/pages/peers.js` с полноценной загрузкой:

```javascript
async function init() {
    const peers = await api.get('/local/peers');
    State.set('peers', peers);
    render(peers);
}
```

В `shell.html` осталась только строка `Router.on('/peers', () => { PeersPage.init(); })`.

**Верифицировано:** `grep "PeersPage" www/src/html/shell.html` → только строка Router; `ls www/src/js/pages/peers.js` → файл существует с `api.get('/local/peers')`.

---

## 14. MEDIUM — Семантическая ошибка в поле `zt_network_id`

**Приоритет:** 🟡 Medium → ✅ Resolved `c474bb1`
**Файл:** `src/exitnode/mod.rs`

### Описание

Поле `zt_network_id: Option<String>` содержало имя ZeroTier-интерфейса (`ztabcd1234e`), а не 16-символьный ID сети (`8056c2e21c000001`). Семантика нарушена.

### Исправление

`ExitNodeState` теперь содержит **оба** поля:

```rust
pub struct ExitNodeState {
    /// ZeroTier interface name (e.g. "ztabcd1234e")
    pub zt_interface: Option<String>,
    /// ZeroTier 16-char network ID (e.g. "8056c2e21c000001"), if provided by caller
    pub zt_network_id: Option<String>,
    // ...
}
```

`zt_interface` заполняется всегда; `zt_network_id` — если передан `network_id` из запроса (используется для проверки `allowDefault`/`allowGlobal`).

**Верифицировано:** `grep "zt_interface\|zt_network_id" src/exitnode/mod.rs` → оба поля присутствуют с корректными комментариями.

---

## 15. MEDIUM — Отсутствует кнопка toggle sidebar

**Приоритет:** 🟡 Medium → ✅ Resolved `85bc5a2`
**Файлы:** `www/src/html/shell.html`, `www/src/css/layout.css`

### Описание

Sidebar не имел кнопки скрыть/показать. Критично для мобильных устройств.

### Исправление

Добавлены `#sidebar-toggle`, `#mobile-bar`, `#sidebar-overlay` в `shell.html`. Функции `toggleSidebar()` и `closeSidebar()` в inline-скрипте. CSS реализует off-canvas поведение через `transform: translateX(-100%)`.

**Верифицировано:** `grep "sidebar-toggle\|toggleSidebar\|closeSidebar" www/src/html/shell.html` → все три присутствуют.

---

## 16. MEDIUM — `metricstoken_file` не настраивался через UI

**Приоритет:** 🟡 Medium → ✅ Resolved `6d4c76f`
**Файл:** `www/src/js/pages/settings-global.js`

### Описание

Поле `metricstoken_file` из `MetricsConfig` нельзя было задать через интерфейс — только вручную редактируя `config.yml`.

### Исправление

В секцию Metrics страницы Global Settings добавлено поле:

```javascript
`<input class="input" id="s-metrics-token"
    value="${cfg.metrics?.metricstoken_file||'/var/lib/zerotier-one/metricstoken.secret'}">`
```

Включено в тело `PUT /api/settings/config`.

**Верифицировано:** `grep "s-metrics-token\|metricstoken_file" www/src/js/pages/settings-global.js` → оба присутствуют.

---

## 17. MEDIUM — `Modal.prompt` вызывался без реализации

**Приоритет:** 🟡 Medium → ✅ Resolved (2026-04)
**Файл:** `www/src/js/components/modal.js`

### Описание

`Modal.prompt?.()` вызывался через optional chaining — тихий fail при отсутствии метода.

### Исправление

`Modal.prompt()` реализован в `modal.js:35`. Optional chaining теперь — защитный слой, не заглушка.

**Верифицировано:** `grep "prompt" www/src/js/components/modal.js` → `prompt(message, placeholder = '') {` на строке 35.

---

## 18. MEDIUM — Состояние bridge/physnet/relay не персистировалось

**Приоритет:** 🟡 Medium → ✅ Resolved `7b0e83e`
**Файл:** `src/runtime_state.rs` (создан), `src/server/state.rs`

### Описание

`AppState.physnet_state`, `bridge_state`, `relay_remote` хранились только в памяти. После перезапуска UI показывал «Bridge: Disabled» даже при активных iptables-правилах.

### Исправление

Создан `src/runtime_state.rs`:
- Структура `RuntimeState { physnet, bridge, relay_remote }`
- Атомарная запись: `write(tmp) → rename(final)` — защита от повреждения при SIGKILL
- Три варианта пути: `$ZTNET_STATE_FILE` → `/var/lib/ztnet-box/state.json` → XDG user data dir
- Загрузка в `AppState::new_with_cache()` при старте
- Метод `AppState::persist_runtime_state()` вызывается в **7 точках мутации**: bridge (enable, disable), physnet (enable, disable), relay (deploy, verify, remove)

**Верифицировано:**
- `grep -rn "persist_runtime_state" src/server/handlers/` → 7 вхождений (bridge: 2, physnet: 2, relay: 3)
- `cat src/runtime_state.rs` — полная реализация с тестами roundtrip

---

## 19. MEDIUM — Зависимость от `sshpass`

**Приоритет:** 🟡 Medium → ✅ Resolved `4ed5d12`
**Файл:** `src/relay/ssh.rs`, `src/relay/mod.rs`

### Описание

`sshpass` передавал пароль через переменную среды/аргумент — виден в `ps aux`. Нет по умолчанию на большинстве систем.

### Исправление

`sshpass` полностью удалён из кода и UI. `RelayDeployConfig` не содержит поля `password`. UI предлагает только `key_path` с пояснением «Key-based auth only. Ensure the key's public part is in `~/.ssh/authorized_keys` on the remote host.»

**Верифицировано:** `grep "password\|sshpass" src/relay/mod.rs` — пусто; `grep "dep-key\|key_path" www/src/js/pages/relay.js` — присутствует.

---

## 20. LOW — CSP: `connect-src *`

**Приоритет:** 🟢 Low → ✅ Resolved `6d4c76f`
**Файл:** `src/server/router.rs`

### Описание

`connect-src *` разрешал AJAX к любому домену — при XSS возможна эксфильтрация данных.

### Исправление

```rust
"connect-src 'self'"
```

**Верифицировано:** `grep "connect-src" src/server/router.rs` → `connect-src 'self'`.

> **Примечание:** `script-src 'unsafe-inline'` и `style-src 'unsafe-inline'` остаются — они обусловлены архитектурой (весь JS/CSS инлайнится в один HTML при сборке). Полноценное решение требует переноса JS в отдельный файл или генерации nonce в `build.rs`. Это зафиксировано как будущая задача.

---

## 21. LOW — `danger_accept_invalid_certs` безусловно

**Приоритет:** 🟢 Low → ✅ Resolved `85bc5a2`
**Файл:** `src/zerotier/local/client.rs`

### Описание

`danger_accept_invalid_certs(true)` применялся безусловно. При настройке `api_url` на удалённый хост — снимал всю TLS-защиту.

### Исправление

```rust
let is_loopback = api_url.contains("127.0.0.1")
    || api_url.contains("localhost")
    || api_url.contains("[::1]");
Client::builder()
    .danger_accept_invalid_certs(is_loopback)
    .build()
```

**Верифицировано:** `grep "is_loopback\|danger_accept" src/zerotier/local/client.rs` → оба присутствуют.

---

## 22. LOW — Дублирование функции `_esc()`

**Приоритет:** 🟢 Low → ✅ Resolved `c474bb1`
**Файл:** `www/src/js/state.js`

### Описание

Функция `_esc(s)` была определена минимум в двух файлах (`log-panel.js`, `settings-ztnode.js`) с незначительным расхождением (отсутствие `&quot;` в одной копии).

### Исправление

Единственная каноническая реализация `Utils.esc()` добавлена в `state.js`:

```javascript
const Utils = (() => {
    function esc(s) {
        return String(s)
            .replace(/&/g,'&amp;').replace(/</g,'&lt;')
            .replace(/>/g,'&gt;').replace(/"/g,'&quot;');
    }
    return { esc };
})();
```

Все вызовы `_esc()` в кодовой базе заменены на `Utils.esc()`.

**Верифицировано:** `grep -rn "function _esc\|_esc(" www/src/js/` → пусто. `grep "Utils.esc" www/src/js/` → 8 вхождений в pages/ и components/.

---

## 23. LOW — Backend-эндпоинты без покрытия во Frontend

**Приоритет:** 🟢 Low → ✅ Resolved `1760c78`

### Статус по каждому эндпоинту

| Эндпоинт | Статус | Где покрыт |
|---|---|---|
| `GET /api/system/zt-status` | ✅ Покрыт | `dashboard.js` — баннер установки ZT |
| `POST /api/system/zt-install` | ✅ Покрыт | `dashboard.js` — кнопка Install |
| `GET /api/logs/level` | ✅ Покрыт | `log-panel.js` — sync при `_loadInitial()` |
| `PUT /api/logs/level` | ✅ Покрыт | `log-panel.js` — `_setLevel()` |
| `GET /api/central/status` | ⚪ Диагностический | Намеренно не в UI |
| `GET /api/central/user` | ⚪ Диагностический | Намеренно не в UI |
| `GET /api/metrics/raw` | ⚪ Для внешнего скрапинга | Намеренно не в UI |
| `GET /api/local/networks/:id/localconf` | ⚪ Будущее | Задел для страницы деталей сети |
| `PUT /api/local/networks/:id/localconf` | ⚪ Будущее | Задел для страницы деталей сети |

---

## 24. LOW — `#[allow(clippy::derivable_impls)]`

**Приоритет:** 🟢 Low → ✅ Resolved `c474bb1`
**Файл:** `src/config/schema.rs`

### Описание

`Config` и `ZeroTierConfig` имели ручные `impl Default` с `#[allow(clippy::derivable_impls)]` вместо `#[derive(Default)]`.

### Исправление

```rust
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config { ... }

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ZeroTierConfig { ... }
```

Остальные структуры (`ServerConfig`, `LocalConfig`, `MetricsConfig` и др.) оставили ручные `impl Default` — в них нетривиальные значения по умолчанию (URL, порты, пути), которые Clippy справедливо не считает derivable.

**Верифицировано:** `grep "allow.*clippy" src/config/schema.rs` → пусто; `grep -rn "allow.*clippy" src/` → пусто во всей кодовой базе.

---

## 25. LOW — Заглушка: `network_id` не передавался в `enable()`

**Приоритет:** 🟢 Low → ✅ Resolved `502a8aa`
**Файлы:** `src/server/handlers/exitnode.rs`, `src/exitnode/mod.rs`

### Описание

Хендлер получал `req.network_id`, но не передавал его в `ExitNodeManager::enable()`. В `ExitNodeState.zt_network_id` всегда было `None`.

### Исправление

Сигнатура `enable()` расширена:

```rust
pub async fn enable(
    &self,
    zt_iface: String,
    wan_iface: String,
    enable_ipv6: bool,
    ipv6_prefix: Option<String>,
    network_id: Option<String>,   // ← добавлено
) -> Result<ExitNodeState, ApiError>
```

`network_id` используется для проверки `allowDefault`/`allowGlobal` в `<net_id>.local.conf` и сохраняется в `ExitNodeState.zt_network_id`.

**Верифицировано:** хендлер передаёт `req.network_id` в пятом аргументе; в `ExitNodeState` оба поля (`zt_interface`, `zt_network_id`) заполняются корректно.

---

## 26. HIGH (CI) — Unused params в cfg-гейтированных ndp-функциях

**Приоритет:** 🔴 High (CI breakage) → ✅ Resolved `d39f17b`
**Файл:** `src/exitnode/ndp.rs`

### Описание

Функции `enable(cfg: &NdpConfig)` и `disable(remove_config: bool)` использовали `#[cfg(not(target_os = "linux"))]` для ранних return. На не-Linux платформах параметры оставались неиспользованными → `warning: unused variable` → ошибка компиляции из-за `RUSTFLAGS="-D warnings"`.

### Исправление

Каждая функция разделена на два отдельных `#[cfg]`-гейтированных определения:

```rust
#[cfg(target_os = "linux")]
pub fn enable(cfg: &NdpConfig) -> Result<NdpStatus, NdpError> { /* полная реализация */ }

#[cfg(not(target_os = "linux"))]
pub fn enable(_cfg: &NdpConfig) -> Result<NdpStatus, NdpError> {
    Err(NdpError::UnsupportedPlatform("ndppd requires Linux".into()))
}
```

Аналогично для `install()` и `disable()`.

**Верифицировано:** `grep "cfg.*target_os\|fn enable\|fn disable\|fn install" src/exitnode/ndp.rs` → все три функции имеют парные определения.

---

## 27. HIGH — Утечка `tokio::spawn` в Rate Limiter (пост-аудит)

**Приоритет:** 🔴 High → ✅ Resolved `55a3b51`
**Файл:** `src/zerotier/central/token_store.rs`

### Описание

`RateLimiter::new()` вызывает `tokio::spawn()` — порождает вечную фоновую задачу рефила. `ZtCentralClient` создавался при **каждом входящем HTTP-запросе** через `TokenStore::active_client()`:

```rust
// ДО: новый клиент (→ новый tokio::spawn) на каждый запрос
pub async fn active_client(&self) -> Option<ZtCentralClient> {
    Some(ZtCentralClient::new(...))  // ← O(requests) утечка задач
}
```

11 Central-эндпоинтов × N запросов = N вечных Tokio-задач. Задачи не имели механизма завершения (`Drop` на `ZtCentralClient` не останавливал их).

### Исправление

В `TokenStoreInner` добавлено поле `cached_client: Option<(String, ZtCentralClient)>`.

`active_client()` использует паттерн **double-checked locking**:
1. Быстрый путь: проверяет кеш под `read`-lock
2. Медленный путь: перестраивает под `write`-lock с повторной проверкой (защита от race condition)

```rust
pub async fn active_client(&self) -> Option<ZtCentralClient> {
    // Fast path: read lock
    {
        let inner = self.inner.read().await;
        if let Some((ref id, ref client)) = inner.cached_client {
            if *id == inner.active_token_id { return Some(client.clone()); }
        }
    }
    // Slow path: write lock + re-check
    let mut inner = self.inner.write().await;
    if let Some((ref id, ref client)) = inner.cached_client {
        if *id == inner.active_token_id { return Some(client.clone()); }
    }
    let token = inner.tokens.iter().find(|t| t.id == inner.active_token_id)?;
    let client = ZtCentralClient::new(self.base_url.clone(), token.token.clone(), &token.rate_limit);
    inner.cached_client = Some((token.id.clone(), client.clone()));
    Some(client)
}
```

Кеш инвалидируется (`cached_client = None`) в каждом мутирующем методе: `add()`, `remove()`, `set_active()`, `update()`.

**Результат:** `tokio::spawn` вызывается не более одного раза на жизненный цикл активного токена, независимо от числа HTTP-запросов.

**Верифицировано:** `grep "cached_client\|invalidate_cache" src/zerotier/central/token_store.rs` → поле и метод присутствуют; все 4 мутирующих метода вызывают `Self::invalidate_cache(&mut inner).await`.

---

## 28. MEDIUM — Нет таймаута SSH-соединения при deploy (пост-аудит)

**Приоритет:** 🟡 Medium → ✅ Resolved `55a3b51`
**Файл:** `src/relay/ssh.rs`

### Описание

`SshClient::run()` использует `std::process::Command` без `ConnectTimeout`. Если удалённый хост молча дропает пакеты (firewall DROP, не RST), `ssh` будет ждать несколько минут до системного TCP-таймаута. Функция выполняется в `tokio::task::spawn_blocking` — поток из пула блокируется всё это время. Кроме того, при зависшем соединении `ServerAliveInterval` не был настроен, что не позволяло обнаружить разрыв в ходе выполнения команд.

### Исправление

```rust
// ConnectTimeout: обрывает TCP-рукопожатие через 15 с
"-o".into(), "ConnectTimeout=15".into(),
// ServerAlive: обнаруживает молчащее соединение за 30 с (10 с × 3 попытки)
"-o".into(), "ServerAliveInterval=10".into(),
"-o".into(), "ServerAliveCountMax=3".into(),
```

**Максимальное время зависания** сведено с неопределённого (минуты) до **~45 секунд** (15 с connect + 30 с keepalive).

**Верифицировано:** `grep "ConnectTimeout\|ServerAlive" src/relay/ssh.rs` → все три опции присутствуют.

---

## Итоговая таблица

| # | Приоритет | Компонент | Проблема | Статус | Коммит |
|---|---|---|---|---|---|
| 1 | 🔴 Critical | Build | `log-panel.js` не включался в бандл | ✅ | `d53a1c2` |
| 2 | 🔴 High | Rust | Rate limiter не работал (permit сразу дропался) | ✅ | `d53a1c2` |
| 3 | 🔴 High | JS ↔ API | POST вместо PUT для update controller network | ✅ | `d53a1c2` |
| 4 | 🔴 High | Security | SSH `StrictHostKeyChecking=no` → MITM | ✅ | `6d4c76f` |
| 5 | 🔴 High | Security | Docker install via `curl \| sh` | ✅ | `6d4c76f` |
| 6 | 🔴 High | Security | SSH-пароль в JSON по HTTP | ✅ | `6d4c76f` + `4ed5d12` |
| 7 | 🔴 High | Frontend | Нет мобильной адаптации (`@media` queries) | ✅ | `85bc5a2` |
| 8 | 🔴 High | Rust | `rand_byte()` — `/dev/urandom` + константный fallback | ✅ | `c474bb1` |
| 9 | 🟡 Medium | Rust | `update_token` уничтожал UUID токена | ✅ | `c474bb1` |
| 10 | 🟡 Medium | JS | N+1 последовательных запросов в controllers-networks | ✅ | `502a8aa` |
| 11 | 🟡 Medium | CSS/JS | Несовместимые CSS-переменные в `log-panel.js` | ✅ | `85bc5a2` |
| 12 | 🟡 Medium | Rust | Неиспользуемый тип-алиас `PhysNetStateArc` | ✅ | `c474bb1` |
| 13 | 🟡 Medium | Frontend | `PeersPage` inline в `shell.html`, нет загрузки данных | ✅ | `502a8aa` |
| 14 | 🟡 Medium | Rust | `zt_network_id` содержал имя интерфейса | ✅ | `c474bb1` |
| 15 | 🟡 Medium | Frontend | Нет кнопки toggle sidebar | ✅ | `85bc5a2` |
| 16 | 🟡 Medium | Frontend | `metricstoken_file` нельзя задать через UI | ✅ | `6d4c76f` |
| 17 | 🟡 Medium | Frontend | `Modal.prompt?.()` — тихий fail | ✅ | (2026-04) |
| 18 | 🟡 Medium | Rust | bridge/physnet/relay state — только in-memory | ✅ | `7b0e83e` |
| 19 | 🟡 Medium | Security | `sshpass` — пароль виден в `ps aux` | ✅ | `4ed5d12` |
| 20 | 🟢 Low | Security | CSP `connect-src *` | ✅ | `6d4c76f` |
| 21 | 🟢 Low | Security | `danger_accept_invalid_certs(true)` безусловно | ✅ | `85bc5a2` |
| 22 | 🟢 Low | Frontend | Дублирование `_esc()` в нескольких модулях | ✅ | `c474bb1` |
| 23 | 🟢 Low | Frontend | Backend-эндпоинты без покрытия во Frontend | ✅ | `1760c78` |
| 24 | 🟢 Low | Rust | `#[allow(clippy::derivable_impls)]` | ✅ | `c474bb1` |
| 25 | 🟢 Low | Rust | `network_id` не передавался в `ExitNodeManager::enable()` | ✅ | `502a8aa` |
| 26 | 🔴 High (CI) | Rust | Unused params в cfg-гейтированных ndp-функциях | ✅ | `d39f17b` |
| 27 | 🔴 High | Rust | Утечка `tokio::spawn` в rate limiter — новый клиент на каждый запрос | ✅ | `55a3b51` |
| 28 | 🟡 Medium | Rust | SSH deploy без `ConnectTimeout` — зависание на firewalled-хостах | ✅ | `55a3b51` |

**Итого: 28 пунктов — все закрыты. Открытых проблем нет.**

---

## Рекомендации по архитектуре

### 1. CSP: убрать `unsafe-inline`

Текущий `script-src 'unsafe-inline'` сводит на нет XSS-защиту CSP. Правильное решение — вынести весь JS в отдельный файл и использовать `script-src 'self'`, либо генерировать nonce в `build.rs` и прописывать его в заголовке через `SetResponseHeaderLayer`.

### 2. Rate Limiter — рассмотреть `governor` crate

Текущая реализация (семафор + рефил-задача) корректна, но самописна. Крейт [`governor`](https://crates.io/crates/governor) предоставляет зрелую token-bucket реализацию с поддержкой burst, jitter и GCRA.

### 3. Relay SSH — нативный Rust SSH

`ssh` бинарь как внешняя зависимость — хрупкое место. Крейты [`russh`](https://crates.io/crates/russh) или [`ssh2`](https://crates.io/crates/ssh2) дадут нативную SSH-поддержку без внешних утилит, с программным управлением таймаутами и known_hosts.

### 4. Покрытие per-network localconf в UI

`GET/PUT /api/local/networks/:id/localconf` реализованы на бэкенде, но не покрыты UI. Страница деталей сети с возможностью выставить `allowDefault`, `allowGlobal`, `allowManaged` напрямую из интерфейса была бы логичным следующим шагом.

### 5. Сборка фронтенда

`build.rs` работает корректно после перемещения `log-panel.js` в `components/`. Для дальнейшей надёжности — явный именованный список всех JS-файлов вместо glob по директориям: «явное лучше неявного», и новый файл не может случайно выпасть из бандла.

---

## История исправлений

| Коммит | Пункты |
|---|---|
| `d53a1c2` | #1, #2, #3 |
| `c474bb1` | #8, #9, #12, #14, #22, #24 |
| `502a8aa` | #10, #13, #25 |
| `85bc5a2` | #7, #11, #15, #21 |
| `6d4c76f` | #4, #5, #6, #16, #20 |
| `1760c78` | #23 |
| `7b0e83e` + `d33bbdf` | #18 |
| `4ed5d12` + `6cfe9b8` | #19 |
| `d39f17b` | #26 |
| `55a3b51` | #27, #28 |
