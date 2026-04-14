# AUDIT.md — ztnet-box

**Дата последнего аудита:** 2026-04-14 (финал)  
**Репозиторий:** `CleoWixom/ztnet-box`  
**Стек:** Rust (Axum, Tokio) + Vanilla JS/HTML/CSS (SPA, сборка через `build.rs`)  
**Текущая версия:** 0.7.6 (Cargo.toml: 0.6.3 — версия обновляется workflow)

---

## Содержание

1. [CRITICAL — Сборочный баг: `log-panel.js` не включается в бандл](#1-critical--log-paneljs-не-включается-в-бандл)
2. [HIGH — Неработающий Rate Limiter](#2-high--неработающий-rate-limiter)
3. [HIGH — Frontend: неверный HTTP-метод при обновлении сети контроллера](#3-high--frontend-неверный-http-метод-при-обновлении-сети-контроллера)
4. [HIGH — Безопасность: SSH с `StrictHostKeyChecking=no`](#4-high--безопасность-ssh-с-stricthostkeycheckingno)
5. [HIGH — Безопасность: `curl | sh` для установки Docker](#5-high--безопасность-curl--sh-для-установки-docker)
6. [HIGH — Безопасность: SSH-пароль передаётся в JSON по HTTP](#6-high--безопасность-ssh-пароль-передаётся-в-json-по-http)
7. [HIGH — Отсутствие мобильной адаптации (нет `@media` queries)](#7-high--отсутствие-мобильной-адаптации-нет-media-queries)
8. [HIGH — Кастомная реализация генерации случайных байт (`rand_byte`)](#8-high--кастомная-реализация-генерации-случайных-байт-rand_byte)
9. [MEDIUM — `update_token` уничтожает оригинальный UUID токена](#9-medium--update_token-уничтожает-оригинальный-uuid-токена)
10. [MEDIUM — N+1 API-запросов в `controllers-networks.js`](#10-medium--n1-api-запросов-в-controllers-networksjs)
11. [MEDIUM — Хардкод CSS-значений в `log-panel.js`](#11-medium--хардкод-css-значений-в-log-paneljs)
12. [MEDIUM — Неиспользуемый тип-алиас `PhysNetStateArc`](#12-medium--неиспользуемый-тип-алиас-physnetstaterc)
13. [MEDIUM — `PeersPage` определена inline в `shell.html`](#13-medium--peerspage-определена-inline-в-shellhtml)
14. [MEDIUM — Семантическая ошибка в поле `zt_network_id`](#14-medium--семантическая-ошибка-в-поле-zt_network_id)
15. [MEDIUM — Отсутствует кнопка сворачивания боковой панели](#15-medium--отсутствует-кнопка-сворачивания-боковой-панели)
16. [MEDIUM — `settings-global.js` не предоставляет UI для `metricstoken_file`](#16-medium--settings-globaljs-не-предоставляет-ui-для-metricstoken_file)
17. [MEDIUM — `Modal.prompt` вызывается с optional chaining без реализации](#17-medium--modalprompt-вызывается-с-optional-chaining-без-реализации)
18. [MEDIUM — Состояние bridge/physnet/relay не сохраняется между перезапусками](#18-medium--состояние-bridgephysnetrelay-не-сохраняется-между-перезапусками)
19. [MEDIUM — Внешние зависимости: `ssh`, `sshpass` CLI-инструменты](#19-medium--внешние-зависимости-ssh-sshpass-cli-инструменты)
20. [LOW — CSP разрешает `unsafe-inline` для скриптов и стилей](#20-low--csp-разрешает-unsafe-inline-для-скриптов-и-стилей)
21. [LOW — `danger_accept_invalid_certs` в ZtLocalClient](#21-low--danger_accept_invalid_certs-в-ztlocalclient)
22. [LOW — Дублирование функции `_esc()` в нескольких JS-модулях](#22-low--дублирование-функции-_esc-в-нескольких-js-модулях)
23. [LOW — Backend-эндпоинты без покрытия во Frontend](#23-low--backend-эндпоинты-без-покрытия-во-frontend)
24. [LOW — `#[allow(clippy::derivable_impls)]` на `Default` для `Config`](#24-low--allowclippy-derivable_impls-на-default-для-config)
25. [LOW — Заглушка в `exitnode_manager.enable()`: `zt_network_id` не передаётся](#25-low--заглушка-в-exitnode_managerenable-zt_network_id-не-передаётся)
26. [Итоговая таблица](#итоговая-таблица)
27. [Рекомендации по архитектуре](#рекомендации-по-архитектуре)

---

## 1. CRITICAL — `log-panel.js` не включается в бандл

**Приоритет:** 🔴 Critical  
**Файл:** `build.rs`, `www/src/js/log-panel.js`, `www/src/html/shell.html`

### Описание

`build.rs` собирает единый `www/build/index.html` из всех JS-файлов. Скрипты из `www/src/js/` включаются только поимённо: `api.js`, `state.js`, `router.js`. Все остальные файлы собираются из поддиректорий `components/` и `pages/`.

`log-panel.js` находится непосредственно в `www/src/js/` (не в `components/` и не в `pages/`), поэтому он **никогда не попадает** в собранный бандл.

При этом `shell.html` в конце вызывает `LogPanel.init()`, что при открытии в браузере приводит к:

```
Uncaught ReferenceError: LogPanel is not defined
```

### Участок кода

```rust
// build.rs, строки ~57-76
// 1. Core scripts first
for name in ["api", "state", "router"] {          // ← log-panel НЕ здесь
    let p = js_dir.join(format!("{name}.js"));
    ...
}
// 2. Component scripts (js/components/)
for f in collect_files(&comp_dir, "js") { ... }   // ← log-panel НЕ здесь
// 3. Page scripts (js/pages/)
for f in collect_files(&page_dir, "js") { ... }   // ← log-panel НЕ здесь
```

```html
<!-- shell.html, строка 147 -->
LogPanel.init();  <!-- ← LogPanel undefined в production -->
```

### Почему это проблема

Панель логов — ключевой компонент отладки. В production-сборке она полностью нерабочая. При этом на этапе разработки (если открывать `shell.html` напрямую с подключёнными скриптами) баг не проявляется.

### Рекомендация

**Вариант 1** (быстрый): добавить `log-panel` в список именованных core-скриптов:

```rust
for name in ["api", "state", "router", "log-panel"] {
    let p = js_dir.join(format!("{name}.js"));
```

**Вариант 2** (архитектурный): переместить `log-panel.js` в `www/src/js/components/`, где он подхватится автоматически — это семантически правильнее, так как LogPanel является компонентом.

---

## 2. HIGH — Неработающий Rate Limiter

**Приоритет:** 🔴 High  
**Файл:** `src/zerotier/central/client.rs`

### Описание

`RateLimiter` реализован через `tokio::sync::Semaphore`. Метод `acquire()` должен блокироваться до получения разрешения и удерживать его до завершения запроса. Однако в текущей реализации:

```rust
async fn acquire(&self) {
    let _ = self.semaphore.acquire().await;
    //  ^ SemaphorePermit немедленно дропается — permit возвращается обратно
}
```

`let _ = expr` в Rust немедленно дропает возвращённое значение. `SemaphorePermit` — RAII-обёртка: при дропе разрешение возвращается в семафор. В результате rate limiter никогда не блокирует ни одного запроса — пропускает всё без ограничений.

### Почему это проблема

Для Free-аккаунтов ZeroTier Central API ограничен 20 req/s. При превышении лимита API возвращает `429 Too Many Requests`. Сейчас защита от этого отсутствует.

### Рекомендация

```rust
// Правильный вариант — удерживать permit через возврат
async fn acquire(&self) -> tokio::sync::OwnedSemaphorePermit {
    Arc::clone(&self.semaphore)
        .acquire_owned()
        .await
        .expect("semaphore closed")
}

// В request():
async fn request<T: DeserializeOwned>(...) -> Result<T, ApiError> {
    let _permit = self.rate_limiter.acquire().await; // держится до конца запроса
    // ... HTTP запрос ...
}
```

---

## 3. HIGH — Frontend: неверный HTTP-метод при обновлении сети контроллера

**Приоритет:** 🔴 High  
**Файл:** `www/src/js/pages/controllers-config.js`, `src/server/router.rs`

### Описание

При сохранении конфигурации локальной сети контроллера фронтенд отправляет `POST`:

```javascript
// controllers-config.js, строка 190
if (_src==='local') await api.post(`/local/controller/networks/${_id}`, body);
```

Но бэкенд-роутер регистрирует обновление через `PUT`:

```rust
// src/server/router.rs
.route(
    "/controller/networks/:id",
    get(local_handler::get_controller_network)
        .put(local_handler::update_controller_network)   // ← PUT, не POST
        .delete(local_handler::delete_controller_network),
)
```

`POST` на `/api/local/controller/networks/:id` вернёт **405 Method Not Allowed**. Кнопка «Save Changes» на странице конфигурации сети контроллера полностью нерабочая.

### Рекомендация

```javascript
// controllers-config.js, строка 190
if (_src==='local') await api.put(`/local/controller/networks/${_id}`, body);
//                              ^^^
```

---

## 4. HIGH — Безопасность: SSH с `StrictHostKeyChecking=no`

**Приоритет:** 🔴 High  
**Файл:** `src/relay/ssh.rs`

### Описание

```rust
// src/relay/ssh.rs
"-o".into(),
"StrictHostKeyChecking=no".into(),   // ← MITM-уязвимость
"-o".into(),
"BatchMode=yes".into(),
```

Отключение проверки host key открывает вектор MITM-атаки: злоумышленник в сети может подменить ответ удалённого сервера, перехватить SSH-пароль и выполнить произвольные команды.

### Почему это проблема

Команды, выполняемые при деплое, включают установку Docker, запуск привилегированных контейнеров и управление firewall. При MITM-атаке всё это может быть выполнено на машине атакующего.

### Рекомендация

```rust
// Вариант 1: убрать StrictHostKeyChecking=no, требовать предварительного одобрения хоста
// Вариант 2: принять fingerprint хоста от пользователя и верифицировать его
"-o".into(),
format!("StrictHostKeyChecking=accept-new").into(), // только первое подключение

// Вариант 3: документировать требование к пользователю (добавить known_hosts перед деплоем)
// и добавить UI-предупреждение
```

---

## 5. HIGH — Безопасность: `curl | sh` для установки Docker

**Приоритет:** 🔴 High  
**Файл:** `src/relay/deploy.rs`

### Описание

```rust
// src/relay/deploy.rs
client
    .run("curl -fsSL https://get.docker.com | sh")
    .map_err(|e| DeployError::Step(format!("Docker install failed: {e}")))?;
```

Скачивание и немедленное исполнение shell-скрипта с внешнего URL без верификации — антипаттерн безопасности. Если `get.docker.com` скомпрометирован или DNS подменён — на удалённом сервере выполняется произвольный код с правами root.

### Рекомендация

```rust
// Вариант 1: использовать пакетный менеджер напрямую
client.run("apt-get install -y docker.io")?;

// Вариант 2: скачать скрипт, показать пользователю, запросить подтверждение
// Вариант 3: использовать официальный APT/YUM репозиторий Docker
client.run("apt-get install -y ca-certificates curl gnupg && \
    install -m 0755 -d /etc/apt/keyrings && \
    curl -fsSL https://download.docker.com/linux/ubuntu/gpg | gpg --dearmor -o /etc/apt/keyrings/docker.gpg && \
    ...")?;
```

---

## 6. HIGH — Безопасность: SSH-пароль передаётся в JSON по HTTP

**Приоритет:** 🔴 High  
**Файл:** `www/src/js/pages/relay.js`, `src/relay/mod.rs`

### Описание

При деплое relay пароль SSH отправляется в теле HTTP-запроса к бэкенду:

```javascript
// relay.js
await api.post('/relay/deploy', {
    host, ssh_port, ssh_user,
    password,   // ← SSH-пароль в plain JSON
    key_path, pylon_port, stop_ufw
});
```

По умолчанию ztnet-box слушает на `127.0.0.1:3000` без TLS. Если пользователь открывает UI через туннель или проксирует доступ — пароль может оказаться в логах прокси, браузерных инструментах разработчика или traffic dumps.

Тип `RelayDeployConfig` в `src/relay/mod.rs` сериализуется целиком, включая поле `password: Option<String>`.

### Рекомендация

- Добавить предупреждение в UI о том, что парольная аутентификация небезопасна без TLS.
- Рекомендовать использование SSH-ключей (`key_path`) как основного способа.
- Не логировать тело запроса (текущий middleware логирует только метод/путь/статус — ✓), убедиться, что `RelayDeployConfig` не попадает в tracing-события с `?` или `{:?}`.

---

## 7. HIGH — Отсутствие мобильной адаптации (нет `@media` queries)

**Приоритет:** 🔴 High  
**Файл:** `www/src/css/layout.css`, все CSS-файлы

### Описание

Ни в одном CSS-файле нет ни одного `@media` запроса. Боковая панель (`#sidebar`) зафиксирована на ширине 220px:

```css
/* layout.css */
#sidebar {
    width: var(--sidebar-w);   /* 220px — фиксировано */
    position: fixed;
    top: 0; left: 0; bottom: 0;
}

#content {
    margin-left: var(--sidebar-w);  /* всегда 220px отступ */
}
```

На экранах шириной < 480px (телефоны) sidebar занимает ~46% ширины, контент сжимается до нечитаемого состояния. Скриншоты в `/docs/screenshots/` показывают мобильные варианты — значит, адаптация задумывалась, но не реализована.

### Рекомендация

```css
/* Добавить в layout.css */
@media (max-width: 768px) {
    #sidebar {
        transform: translateX(-100%);
        transition: transform var(--t-base);
        z-index: 200;
    }
    #sidebar.open {
        transform: translateX(0);
    }
    #content {
        margin-left: 0;
    }
    /* Оверлей при открытом меню */
    #sidebar-overlay {
        display: block;
        position: fixed;
        inset: 0;
        background: rgba(0,0,0,.5);
        z-index: 150;
    }
}
```

(Связано с п. 15 — кнопка toggle sidebar.)

---

## 8. HIGH — Кастомная реализация генерации случайных байт (`rand_byte`)

**Приоритет:** 🔴 High  
**Файл:** `src/zerotier/local/client.rs`

### Описание

```rust
// local/client.rs, строки в конце файла
fn rand_byte() -> u8 {
    use std::io::Read;
    let mut buf = [0u8; 1];
    std::fs::File::open("/dev/urandom")
        .and_then(|mut f| f.read_exact(&mut buf).map(|_| buf[0]))
        .unwrap_or(0xab)  // ← fallback — константа, не случайное значение!
}
```

Проблемы:
1. Читает **по одному байту за раз** из `/dev/urandom`, открывая и закрывая файл 6 раз подряд — неэффективно.
2. На Windows или нестандартных Unix-системах без `/dev/urandom` возвращает константу `0xab` — все генерируемые network ID будут одинаковы (`<node_id>abababababab`).
3. `uuid` уже является зависимостью проекта и содержит правильную кроссплатформенную реализацию генерации случайных данных.

```rust
// Используется для генерации суффикса network ID
let suffix: String = (0..6).map(|_| format!("{:02x}", rand_byte())).collect();
```

### Рекомендация

```rust
// Использовать уже подключённый uuid crate
use uuid::Uuid;

fn random_network_suffix() -> String {
    let bytes = Uuid::new_v4();
    hex::encode(&bytes.as_bytes()[..6])
    // или без hex crate:
}

// Ещё проще — генерировать через Uuid напрямую:
let suffix: String = Uuid::new_v4()
    .as_bytes()[..3]
    .iter()
    .map(|b| format!("{b:02x}"))
    .collect();
```

---

## 9. MEDIUM — `update_token` уничтожает оригинальный UUID токена

**Приоритет:** 🟡 Medium  
**Файл:** `src/server/handlers/tokens.rs`

### Описание

```rust
// tokens.rs, update_token handler
s.token_store.remove(&id).await;
// ...
// Re-insert with same id not possible via current API; emit new entry, accept new id
let t = s.token_store.add(updated.name, updated.token, updated.rate_limit).await;
// ^ CentralToken::new() внутри генерирует новый Uuid::new_v4()
```

При `PUT /api/settings/tokens/:id` старый токен удаляется и создаётся новый с другим UUID. Это нарушает идемпотентность операции и ломает любые сохранённые ссылки на токен по ID (например, если `active_token_id` указывает на старый ID и токен был активным, состояние восстанавливается некорректно).

Также в комментарии явно написано `// Re-insert with same id not possible via current API` — это признание архитектурного дефекта в `TokenStore`.

### Рекомендация

Добавить в `TokenStore` метод `update(id, name, token, rate_limit)`, который обновляет запись без смены UUID:

```rust
// В TokenStore
pub async fn update(&self, id: &str, name: String, token: String, rl: RateLimit)
    -> Option<CentralToken>
{
    let mut store = self.inner.write().await;
    if let Some(t) = store.tokens.iter_mut().find(|t| t.id == id) {
        t.name = name;
        t.token = token;
        t.rate_limit = rl;
        Some(t.clone())
    } else {
        None
    }
}
```

---

## 10. MEDIUM — N+1 API-запросов в `controllers-networks.js`

**Приоритет:** 🟡 Medium  
**Файл:** `www/src/js/pages/controllers-networks.js`

### Описание

```javascript
// controllers-networks.js
// Сначала получает список ID (массив строк)
_nets = (await api.get('/local/controller/networks')||[]).map(id=>({id, _src:'local', name:''}));

// Затем делает N отдельных запросов — по одному на каждую сеть
for (let i = 0; i < _nets.length; i++) {
    try {
        const d = await api.get(`/local/controller/networks/${_nets[i].id}`);
        _nets[i] = {..._nets[i],...d,_src:'local'};
    } catch(e){}
}
```

При 10 сетях это 11 последовательных (не параллельных) запросов. Каждый запрос к бэкенду инициирует запрос к ZeroTier daemon.

### Почему это проблема

ZeroTier local API для контроллера не предоставляет эндпоинт "получить все сети с деталями" — только список ID. Тем не менее, N+1 запросы выполняются **последовательно**, а не параллельно.

### Рекомендация

```javascript
// Параллельная загрузка через Promise.allSettled
const ids = (await api.get('/local/controller/networks')) || [];
const details = await Promise.allSettled(
    ids.map(id => api.get(`/local/controller/networks/${id}`))
);
_nets = details
    .filter(r => r.status === 'fulfilled')
    .map(r => ({ ...r.value, _src: 'local' }));
```

---

## 11. MEDIUM — Хардкод CSS-значений в `log-panel.js`

**Приоритет:** 🟡 Medium  
**Файл:** `www/src/js/log-panel.js`

### Описание

```javascript
// log-panel.js, функция _injectStyles()
s.textContent = `
  #log-panel { background:var(--bg-secondary,#1a1a2e); border-top:1px solid var(--border,#2a2a4a); }
  #log-bar   { background:var(--bg-tertiary,#111122); }
  // ...
  .log-badge { background:var(--accent,#6366f1); }
`;
```

Используются CSS-переменные с fallback-значениями: `var(--bg-secondary,#1a1a2e)`, `var(--border,#2a2a4a)`, `var(--accent,#6366f1)`. Но в `variables.css` используется другая система именования: `--c-surface`, `--c-border`, `--c-primary`. Переменные не совпадают, поэтому **всегда** применяются хардкоданные fallback-цвета, а не токены из design system.

| log-panel.js использует | variables.css определяет |
|--------------------------|--------------------------|
| `--bg-secondary` | `--c-surface` |
| `--border` | `--c-border` |
| `--accent` | `--c-primary` |
| `--text` | `--c-text` |
| `--bg-hover` | `--c-surface2` |

### Рекомендация

Либо вынести стили log-panel в `www/src/css/components.css` с правильными переменными, либо исправить имена:

```javascript
// Использовать реальные переменные из variables.css
`#log-panel { background: var(--c-surface); border-top: 1px solid var(--c-border); }`
```

---

## 12. MEDIUM — Неиспользуемый тип-алиас `PhysNetStateArc`

**Приоритет:** 🟡 Medium  
**Файл:** `src/server/handlers/physnet.rs`

### Описание

```rust
// physnet.rs, строка 15
pub type PhysNetStateArc = Arc<RwLock<PhysNetState>>;
```

Этот тип-алиас нигде не используется — ни внутри модуля, ни в других файлах. Аналогичный алиас определён в конце `relay.rs`:

```rust
// relay.rs, последняя строка
pub type RelayRemoteState = RwLock<Option<RemoteRelayInfo>>;
```

`RelayRemoteState` тоже нигде не используется. В `state.rs` поля определены напрямую без этих алиасов.

### Рекомендация

Удалить оба неиспользуемых алиаса. Если они планировались как публичный API для тестов — добавить `#[cfg(test)]` или документацию.

---

## 13. MEDIUM — `PeersPage` определена inline в `shell.html`

**Приоритет:** 🟡 Medium  
**Файл:** `www/src/html/shell.html`

### Описание

Все страницы приложения имеют отдельные файлы в `www/src/js/pages/`. Но `PeersPage` определена прямо в `shell.html` в блоке `<script>`:

```html
<!-- shell.html, строки внутри <script> блока -->
const PeersPage = {
  init() {
    const peers = State.get('peers')||[];
    const rows = peers.map(p => `<tr>...`).join('');
    document.getElementById('content').innerHTML = `...`;
  }
};
```

Это нарушает единообразие структуры проекта и усложняет поддержку. Более того, `PeersPage` использует кэшированные данные из `State.get('peers')` — если переходить на `/peers` напрямую без предварительного посещения Dashboard, список окажется пустым (данные не загружены).

### Рекомендация

Создать `www/src/js/pages/peers.js` с полноценной загрузкой данных:

```javascript
const PeersPage = (() => {
  return {
    async init() {
      document.getElementById('content').innerHTML = '<div class="page"><div class="loading-row">...</div></div>';
      try {
        const peers = await api.get('/local/peers');
        State.set('peers', peers);
        // render...
      } catch(e) { Toast.error(e.message); }
    }
  };
})();
```

---

## 14. MEDIUM — Семантическая ошибка в поле `zt_network_id`

**Приоритет:** 🟡 Medium  
**Файл:** `src/exitnode/mod.rs`

### Описание

```rust
// exitnode/mod.rs
#[derive(Debug, Clone, Serialize, Default)]
pub struct ExitNodeState {
    pub zt_network_id: Option<String>,  // ← поле называется "network_id"
    // ...
}

// Но при включении в него записывается имя ИНТЕРФЕЙСА, а не ID сети:
pub async fn enable(&self, zt_iface: String, ...) {
    let new_state = ExitNodeState {
        zt_network_id: Some(zt_iface),  // ← здесь "ztabcd1234e", не "8056c2e21c000001"
        // ...
    };
}
```

Поле называется `zt_network_id`, но содержит имя ZeroTier-интерфейса (например, `ztabcd1234e`), а не 16-символьный ID сети. Это приводит к путанице в API-ответе и в логах.

### Рекомендация

```rust
pub struct ExitNodeState {
    pub zt_interface: Option<String>,  // переименовать для ясности
    // ...
}
```

И обновить соответствующий JSON-ответ в `handlers/exitnode.rs` (`"zt_network_id"` → `"zt_interface"`).

---

## 15. MEDIUM — Отсутствует кнопка сворачивания боковой панели

**Приоритет:** 🟡 Medium  
**Файл:** `www/src/html/shell.html`, `www/src/css/layout.css`

### Описание

Sidebar не имеет кнопки скрыть/показать. Это критично для мобильных устройств (где sidebar занимает половину экрана) и полезно на десктопе для работы с широкими таблицами.

### Рекомендация

Добавить toggle-кнопку в шапку sidebar и обработчик в JS:

```html
<!-- В shell.html, внутри #sidebar -->
<button id="sidebar-toggle" onclick="document.getElementById('sidebar').classList.toggle('collapsed')"
        title="Toggle sidebar" aria-label="Toggle navigation">
  <svg width="20" height="20" viewBox="0 0 20 20">...</svg>
</button>
```

```css
/* layout.css */
#sidebar.collapsed {
    width: 52px;
    overflow: hidden;
}
#sidebar.collapsed .nav-section-label,
#sidebar.collapsed .sidebar-logo-text,
#sidebar.collapsed .sidebar-logo-sub,
#sidebar.collapsed .nav-item > span { display: none; }
#sidebar.collapsed + #content { margin-left: 52px; }
```

---

## 16. MEDIUM — `settings-global.js` не предоставляет UI для `metricstoken_file`

**Приоритет:** 🟡 Medium  
**Файл:** `www/src/js/pages/settings-global.js`, `src/server/handlers/config.rs`

### Описание

Бэкенд поддерживает настройку `metricstoken_file` (путь к `metricstoken.secret` для авторизации на ZeroTier metrics endpoint):

```rust
// config/schema.rs
pub struct MetricsConfig {
    pub metricstoken_file: std::path::PathBuf,
    // ...
}
```

Но в UI для метрик отображаются только `prometheus_url` и `poll_interval_seconds`. Поле `metricstoken_file` нельзя настроить через интерфейс. Пользователь не может задать кастомный путь к файлу токена, не редактируя `config.yml` вручную.

### Рекомендация

Добавить поле в секцию Metrics страницы Global Settings:

```javascript
// В settings-global.js
`<div class="field">
  <label class="field-label">Metrics Token File</label>
  <input class="input" id="s-metrics-token" 
         value="${cfg.metrics?.metricstoken_file||'/var/lib/zerotier-one/metricstoken.secret'}">
  <div class="text-dim text-sm">Path to metricstoken.secret (leave default if unsure)</div>
</div>`
```

И включить в тело запроса `PUT /api/settings/config`:

```javascript
metrics: {
    enabled: ...,
    prometheus_url: ...,
    poll_interval_seconds: ...,
    metricstoken_file: document.getElementById('s-metrics-token')?.value?.trim(),
},
```

---

## 17. ✅ RESOLVED — `Modal.prompt` вызывается с optional chaining без реализации

**Приоритет:** 🟢 Resolved (2026-04)  
**Файл:** `www/src/js/pages/relay.js`, `www/src/js/components/modal.js`

### Статус

`Modal.prompt()` реализован в `www/src/js/components/modal.js` (строка 35). Optional chaining `?.` теперь является лишь защитным слоем, но метод гарантированно существует. Проблема закрыта.

---

## 18. MEDIUM — Состояние bridge/physnet/relay не сохраняется между перезапусками

**Приоритет:** 🟡 Medium  
**Файл:** `src/server/state.rs`

### Описание

```rust
// state.rs
pub struct AppState {
    pub physnet_state: Arc<RwLock<PhysNetState>>,   // in-memory
    pub bridge_state:  Arc<RwLock<BridgeState>>,    // in-memory
    pub relay_remote:  Arc<RwLock<Option<RemoteRelayInfo>>>, // in-memory
    // ...
}
```

Все три состояния хранятся только в памяти. После перезапуска `ztnet-box`:
- UI покажет "Bridge: Disabled" и "PhysNet: Inactive", даже если правила iptables/iproute2 фактически активны.
- Кнопка "Disable" не появится — пользователь не сможет корректно выключить активную конфигурацию через UI.
- Для relay: ссылка на удалённый relay теряется, и кнопки "Verify" / "Remove" не будут доступны.

### Рекомендация

Сохранять состояния в `config.yml` при изменении, или создать отдельный файл состояния (например, `/var/lib/ztnet-box/state.json`), загружаемый при старте. Для physnet/bridge дополнительно можно зондировать реальное состояние системы (например, `ip link show br0`) при инициализации.

---

## 19. MEDIUM — Внешние зависимости: `ssh`, `sshpass` CLI-инструменты

**Приоритет:** 🟡 Medium  
**Файл:** `src/relay/ssh.rs`

### Описание

```rust
// ssh.rs
let ssh = which::which("ssh").map_err(|e| SshError::NotFound(e.to_string()))?;
// ...
let sshpass = which::which("sshpass").map_err(...)?;
```

Функциональность relay deploy зависит от внешних системных утилит `ssh` и `sshpass`. Проблемы:

1. `sshpass` нет по умолчанию на большинстве систем — требует отдельной установки.
2. `sshpass` передаёт пароль через переменную среды или аргумент командной строки, что делает его видимым через `ps aux`.
3. Комментарий в коде сам объясняет причину выбора `ssh` вместо Rust SSH crate: _"Avoids heavy crypto dependencies"_. Но это решение ценой безопасности и надёжности.

### Рекомендация

Рассмотреть `russh` или `ssh2` crate для нативной SSH-поддержки. Если оставлять `ssh`-бинарь — убрать поддержку паролей через `sshpass` и требовать исключительно ключевую аутентификацию.

---

## 20. LOW — CSP разрешает `unsafe-inline` для скриптов и стилей

**Приоритет:** 🟢 Low  
**Файл:** `src/server/router.rs`

### Описание

```rust
// router.rs
"default-src 'self'; \
 script-src 'self' 'unsafe-inline'; \   // ← XSS-риск
 style-src 'self' 'unsafe-inline'; \    // ← XSS-риск
 img-src 'self' data:; \
 connect-src 'self' *",                 // ← connect-src 'self' * — очень широко
```

`'unsafe-inline'` для `script-src` сводит на нет большую часть защиты от XSS, которую даёт CSP. Поскольку весь JS встроен в один HTML-файл (архитектура `build.rs`), использование nonce или hash было бы правильным решением.

`connect-src *` разрешает AJAX-запросы к любому домену — это может быть использовано при XSS для эксфильтрации данных.

### Рекомендация

В `build.rs` генерировать случайный nonce при каждой сборке и прописывать его в CSP-заголовок через `SetResponseHeaderLayer`. Либо перейти на `script-src 'self'` с вынесением JS в отдельный файл (не inline).

---

## 21. LOW — `danger_accept_invalid_certs` в ZtLocalClient

**Приоритет:** 🟢 Low  
**Файл:** `src/zerotier/local/client.rs`

### Описание

```rust
// local/client.rs
let http = Client::builder()
    .danger_accept_invalid_certs(true) // ZT One uses self-signed cert
    .build()
    .expect("reqwest client");
```

Принятие самоподписанных сертификатов обосновано для локального ZeroTier daemon (по умолчанию `http://127.0.0.1:9993` — без TLS вообще). Однако если пользователь настраивает `api_url` на удалённый хост, эта опция снимает всю защиту TLS.

### Рекомендация

Отключать `danger_accept_invalid_certs` если `api_url` не является localhost/loopback, или добавить опцию в конфигурацию:

```rust
let is_loopback = api_url.contains("127.0.0.1") || api_url.contains("localhost") || api_url.contains("[::1]");
Client::builder()
    .danger_accept_invalid_certs(is_loopback)
    .build()
```

---

## 22. LOW — Дублирование функции `_esc()` в нескольких JS-модулях

**Приоритет:** 🟢 Low  
**Файл:** `www/src/js/log-panel.js`, `www/src/js/pages/settings-ztnode.js`

### Описание

Идентичная функция экранирования HTML определена как минимум дважды:

```javascript
// log-panel.js
function _esc(s) {
    return String(s)
        .replace(/&/g, '&amp;').replace(/</g, '&lt;')
        .replace(/>/g, '&gt;').replace(/"/g, '&quot;');
}

// settings-ztnode.js
function _esc(s) {
    return String(s)
        .replace(/&/g, '&amp;').replace(/</g, '&lt;')
        .replace(/>/g, '&gt;');
    // ← без &quot; — незначительное расхождение
}
```

Глобальная область видимости (все модули через IIFE) позволяет вынести утилиту в общий файл.

### Рекомендация

Добавить в `www/src/js/state.js` или создать `www/src/js/utils.js`:

```javascript
const Utils = (() => {
    function esc(s) {
        return String(s)
            .replace(/&/g, '&amp;').replace(/</g, '&lt;')
            .replace(/>/g, '&gt;').replace(/"/g, '&quot;');
    }
    return { esc };
})();
```

---

## 23. LOW — Backend-эндпоинты без покрытия во Frontend

**Приоритет:** 🟢 Low  
**Файл:** различные

### Описание

Следующие API-эндпоинты реализованы на бэкенде, но не вызываются фронтендом:

| Эндпоинт | Назначение | Статус во Frontend |
|----------|-----------|-------------------|
| `GET /api/system/zt-status` | Детектирование установки ZeroTier | ❌ Не используется |
| `POST /api/system/zt-install` | Установка ZeroTier | ❌ Не используется |
| `GET /api/central/status` | Статус Central API аккаунта | ❌ Не используется |
| `GET /api/central/user` | Информация о пользователе Central | ❌ Не используется |
| `GET /api/metrics/raw` | Raw Prometheus метрики | ❌ Не используется |
| `GET /api/local/networks/:id/localconf` | Per-network local.conf (allowDefault и др.) | ❌ Не используется |
| `PUT /api/local/networks/:id/localconf` | Обновление per-network local.conf | ❌ Не используется |
| `GET /api/logs/level` | Текущий уровень логирования | ❌ Только PUT используется |
| `GET /api/local/config` | Настройки ZeroTier node (local.conf) | ✅ Покрыт `settings-ztnode.js` |
| `PUT /api/local/config` | Обновление настроек ZeroTier node | ✅ Покрыт `settings-ztnode.js` |
| `GET /api/settings/config` | Настройки ztnet-box (config.yml) | ✅ Покрыт `settings-global.js` |
| `PUT /api/settings/config` | Обновление настроек ztnet-box | ✅ Покрыт `settings-global.js` |
| `GET /api/exitnode/ndp/status` | Статус ndppd | ✅ Покрыт `exitnode.js` |
| `POST /api/exitnode/ndp/install` | Установка ndppd | ✅ Покрыт `exitnode.js` |
| `POST /api/exitnode/ndp/enable` | Включение NDP Proxy | ✅ Покрыт `exitnode.js` |
| `POST /api/exitnode/ndp/disable` | Отключение NDP Proxy | ✅ Покрыт `exitnode.js` |

Особенно важны `zt-status`/`zt-install` — они были бы полезны на Dashboard для помощи пользователям без установленного ZeroTier.

---

## 24. LOW — `#[allow(clippy::derivable_impls)]` на `Default` для `Config`

**Приоритет:** 🟢 Low  
**Файл:** `src/config/schema.rs`

### Описание

```rust
#[allow(clippy::derivable_impls)]
impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig::default(),
            zerotier: ZeroTierConfig::default(),
            metrics: MetricsConfig::default(),
            exitnode: ExitNodeConfig::default(),
        }
    }
}
```

Clippy предупреждает, что `Default` можно вывести через `#[derive(Default)]`. Вместо исправления добавлен `allow`. То же для `ZeroTierConfig`. Если кастомная логика в `Default` не требуется — следует использовать `#[derive(Default)]`.

---

## 25. LOW — Заглушка в `exitnode_manager.enable()`: `zt_network_id` не передаётся

**Приоритет:** 🟢 Low  
**Файл:** `src/server/handlers/exitnode.rs`, `src/exitnode/mod.rs`

### Описание

Хендлер `enable` получает опциональный `network_id` для проверки `allowDefault` и проверок, но не передаёт его в `ExitNodeManager::enable()`:

```rust
// handlers/exitnode.rs
let state = s.exitnode_manager
    .enable(
        req.zt_interface,
        req.wan_interface,
        req.enable_ipv6,
        req.ipv6_prefix,
        // ← req.network_id НЕ передаётся
    ).await?;
```

```rust
// exitnode/mod.rs — сигнатура метода
pub async fn enable(
    &self,
    zt_iface: String,
    wan_iface: String,
    enable_ipv6: bool,
    ipv6_prefix: Option<String>,
    // ← network_id отсутствует
) -> Result<ExitNodeState, ApiError>
```

В результате `ExitNodeState.zt_network_id` всегда содержит имя интерфейса (см. п. 14), и реальный `network_id` нигде не сохраняется для дальнейшего использования.

---

## Итоговая таблица

| # | Приоритет | Компонент | Проблема | Статус |
|---|-----------|-----------|----------|--------|
| 1 | ✅ Resolved | Build | `log-panel.js` не включался в бандл → перемещён в `components/` | ✅ `d53a1c2` |
| 2 | ✅ Resolved | Rust | Rate limiter не работал (permit сразу дропался) — исправлен `.forget()` | ✅ `d53a1c2` |
| 3 | ✅ Resolved | JS ↔ API | POST вместо PUT для update controller network → исправлено | ✅ `d53a1c2` |
| 4 | ✅ Resolved | Security | SSH: `StrictHostKeyChecking=no` → `accept-new` | ✅ `6d4c76f` |
| 5 | ✅ Resolved | Security | Docker install via `curl \| sh` → apt/dnf/pacman | ✅ `6d4c76f` |
| 6 | ✅ Resolved | Security | SSH-пароль: добавлено UI-предупреждение об HTTPS | ✅ `6d4c76f` |
| 7 | ✅ Resolved | Frontend | Нет мобильной адаптации (`@media` queries) | ✅ `85bc5a2` |
| 8 | ✅ Resolved | Rust | `rand_byte()` → `getrandom::getrandom()` — 6 байт, без fallback | ✅ `c474bb1` |
| 9 | ✅ Resolved | Rust | `update_token` уничтожал UUID — исправлено через `TokenStore::update()` | ✅ `c474bb1` |
| 10 | ✅ Resolved | JS | N+1 последовательных запросов в controllers-networks | ✅ `502a8aa` |
| 11 | ✅ Resolved | CSS/JS | Хардкод CSS-значений в `log-panel.js` (несовместимые переменные) | ✅ `85bc5a2` |
| 12 | ✅ Resolved | Rust | Неиспользуемый тип-алиас `PhysNetStateArc` | ✅ `c474bb1` |
| 13 | ✅ Resolved | Frontend | `PeersPage` inline в `shell.html`, не загружает данные при прямом переходе | ✅ `502a8aa` |
| 14 | ✅ Resolved | Rust | Поле `zt_network_id` содержало имя интерфейса — переименовано в `zt_interface` | ✅ `c474bb1` |
| 15 | ✅ Resolved | Frontend | Нет кнопки toggle sidebar (мобильная и десктоп UX) | ✅ `85bc5a2` |
| 16 | ✅ Resolved | Frontend | `metricstoken_file` добавлено в Settings UI | ✅ `6d4c76f` |
| 17 | ✅ Resolved | Frontend | `Modal.prompt?.()` — тихий fail при отсутствии метода | ✅ Решён (2026-04) |
| 18 | 🟡 Medium | Rust | bridge/physnet/relay state in-memory, теряется при перезапуске | ❌ Открыт |
| 19 | 🟡 Medium | Rust | Внешние зависимости от `ssh`/`sshpass` CLI-утилит | ❌ Открыт |
| 20 | ✅ Resolved | Security | CSP: `connect-src *` → `connect-src 'self'` | ✅ `6d4c76f` |
| 21 | ✅ Resolved | Security | `danger_accept_invalid_certs(true)` безусловно — исправлено is_loopback | ✅ `85bc5a2` |
| 22 | ✅ Resolved | Frontend | Дублирование функции `_esc()` — вынесена в `Utils.esc()` | ✅ `c474bb1` |
| 23 | 🟢 Low | Frontend | 8 backend-эндпоинтов без UI-покрытия (8 из 16 покрыто после feat(ndp)+feat(settings)) | ❌ Частично |
| 24 | ✅ Resolved | Rust | `#[allow(clippy::derivable_impls)]` → `#[derive(Default)]` | ✅ `c474bb1` |
| 25 | ✅ Resolved | Rust | `network_id` теперь передаётся в `ExitNodeManager::enable()` | ✅ `502a8aa` |
| 26 | 🔴 High | CI/Cross | `ndp.rs`: unused params на не-Linux → `-D warnings` → ошибка сборки macOS/Windows | ✅ Исправлен `d39f17b` |

---

## Рекомендации по архитектуре

### 1. Персистентность состояния

Добавить `StateStore` — тонкий слой поверх `config.yml` или отдельного `state.json` для bridge/physnet/relay. При старте приложения зондировать реальное состояние ОС (например, `ip link show br0 2>/dev/null`) и синхронизировать с сохранённым.

### 2. Сборка фронтенда

`build.rs` собирает весь фронтенд в одну строку внутри Rust-компиляции. Это нестандартно и хрупко (баг с `log-panel.js` — наглядный пример). Рассмотреть:
- Именованный список всех JS-файлов в `build.rs` вместо glob (явное лучше неявного).
- Или вынести сборку в `Makefile`/`justfile` с простым `cat` для сохранения текущей архитектуры.

### 3. Relay deploy

Relay deploy через SSH-бинарь — самое хрупкое место проекта. Если эта функциональность важна, стоит:
- Реализовать через нативный Rust SSH (crate `russh` или `ssh2`).
- Или задокументировать как "advanced feature" с чётким предупреждением о безопасности.
- Убрать поддержку пароля через `sshpass`.

### 4. Rate Limiter

После исправления бага (п. 2) рассмотреть использование `governor` crate вместо самописного семафора — это зрелая реализация token bucket с поддержкой burst.

### 5. Мобильность UI

Структурный подход к мобильной адаптации:
1. Сначала реализовать `@media (max-width: 768px)` в `layout.css` (скрыть sidebar).
2. Добавить кнопку toggle и оверлей в `shell.html`.
3. Проверить overflow для каждой страницы (особенно таблицы в controllers-networks, dashboard peers).

---

## 26. ✅ RESOLVED — Cross-platform: unused params в cfg-гейтированных ndp-функциях

**Приоритет:** 🔴 High (CI breakage)  
**Файл:** `src/exitnode/ndp.rs`  
**Коммит исправления:** `d39f17b` (2026-04-13)

### Описание

Функции `enable(cfg: &NdpConfig)` и `disable(remove_config: bool)` использовали паттерн с `#[cfg(not(target_os = "linux"))] return Err(...)` + `#[cfg(target_os = "linux")] { ... use params ... }`. На не-Linux (macOS, Windows) параметры `cfg` и `remove_config` оставались неиспользуемыми — `warning: unused variable` → ошибка компиляции из-за `RUSTFLAGS="-D warnings"`.

Тот же баг был в `install()` — потенциальный `clippy::needless_return` когда linux-блок убирается.

### Исправление

Разделены на два отдельных `#[cfg]`-гейтированных определения функции:
- `#[cfg(target_os = "linux")]` — полная реализация с правильными именами параметров.
- `#[cfg(not(target_os = "linux"))]` — заглушка с `_cfg` / `_remove_config`.

Паттерн соответствует уже существующему в `src/bridge/deps.rs` (`_prefer_remove_conflicts`).

---

## Список задач (TODO)

Приоритизированный список открытых проблем для устранения:

### 🔴 Critical / High — исправить в первую очередь

- [ ] **#1** `build.rs`: добавить `"log-panel"` в список core-скриптов (`["api", "state", "router", "log-panel"]`) — 1 строка, сразу устраняет `LogPanel is not defined` в production
- [ ] **#2** `src/zerotier/central/client.rs`: исправить `RateLimiter::acquire()` — хранить `SemaphorePermit` до конца запроса (заменить `let _ = ...` на `let _permit = ...` и вернуть permit из функции)
- [ ] **#3** `www/src/js/pages/controllers-config.js:190`: заменить `api.post(...)` на `api.put(...)` для обновления сети контроллера
- [ ] **#4** `src/relay/ssh.rs`: убрать `StrictHostKeyChecking=no`; использовать `accept-new` или задокументировать требование pre-approve host в UI
- [ ] **#5** `src/relay/deploy.rs`: заменить `curl -fsSL https://get.docker.com | sh` на установку через пакетный менеджер (`apt-get install -y docker.io` или официальный APT-репозиторий)
- [ ] **#6** `src/relay/mod.rs`: не включать `password` в трассировку; добавить UI-предупреждение что парольная аутентификация небезопасна без TLS
- [ ] **#8** `src/zerotier/local/client.rs`: заменить самописный `rand_byte()` на `getrandom` crate или `rand::random::<u8>()` — убрать fallback-константу `0xAB`

### 🟡 Medium — исправить после критических

- [ ] **#9** `src/server/handlers/tokens.rs`: исправить `update_token` — не уничтожать оригинальный UUID при PATCH
- [ ] **#10** `www/src/js/pages/controllers-networks.js`: заменить последовательные N+1 запросы на `Promise.allSettled([...])`
- [ ] **#11** `www/src/js/log-panel.js`: убрать хардкод CSS-строк (`height: 220px` и др.), использовать CSS custom properties
- [ ] **#12** `src/server/handlers/physnet.rs`: удалить неиспользуемый `pub type PhysNetStateArc`
- [ ] **#13** `www/src/html/shell.html`: вынести `PeersPage` в `www/src/js/pages/peers.js` с полноценным `api.get('/local/peers')`
- [ ] **#14** `src/exitnode/mod.rs`: переименовать `zt_network_id` → `zt_interface` в `ExitNodeState`; обновить JSON-ответ хендлера
- [ ] **#15** `www/src/html/shell.html` + `www/src/css/layout.css`: добавить toggle-кнопку для sidebar
- [ ] **#16** `www/src/js/pages/settings-global.js`: добавить UI-поле для `metricstoken_file`
- [ ] **#18** `src/server/state.rs`: добавить персистентность для bridge/physnet/relay state (файл состояния или config.yml)
- [ ] **#19** `src/relay/ssh.rs`: рассмотреть переход на `ssh2`/`russh` crate вместо `ssh`+`sshpass` CLI

### 🟢 Low — улучшения

- [ ] **#20** `src/server/router.rs`: убрать `'unsafe-inline'` из CSP (или сгенерировать nonce в build.rs)
- [ ] **#21** `src/zerotier/local/client.rs`: сделать `danger_accept_invalid_certs` зависимым от is_loopback
- [ ] **#22** `www/src/js/`: вынести общую `_esc()` в `www/src/js/state.js` или новый `utils.js`
- [ ] **#23** Frontend: реализовать UI для `GET/POST /api/system/zt-status`/`zt-install` на Dashboard; покрыть `GET /api/local/networks/:id/localconf` в network detail view
- [ ] **#24** `src/config/schema.rs`: заменить `#[allow(clippy::derivable_impls)] impl Default` на `#[derive(Default)]`
- [ ] **#25** `src/server/handlers/exitnode.rs` + `src/exitnode/mod.rs`: передавать `network_id` в `ExitNodeManager::enable()`, хранить в `ExitNodeState`

### 📋 Мобильная адаптация (отдельная задача, #7 + #15)

- [ ] Добавить `@media (max-width: 768px)` в `layout.css`: скрыть sidebar, сделать контент full-width
- [ ] Добавить hamburger-кнопку в шапку и overlay для закрытия sidebar на мобильном
- [ ] Проверить и исправить горизонтальный overflow в таблицах (controllers-networks, dashboard peers, physnet)
