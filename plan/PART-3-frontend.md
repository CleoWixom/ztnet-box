# PART 3 — Frontend

> Ветки: `feat/part3-build-pipeline` (объединённая, содержит всё)

---

## feat/part3-build-pipeline ✅ merged #10 (v0.4.0)

**Все PART 3 ветки объединены в один PR.**

### build.rs ✅
- [x] Читает CSS в порядке: `variables` → `reset` → `layout` → `components` → `pages`
- [x] Читает JS: `components/` → `pages/` → `api.js`, `state.js`, `router.js`
- [x] Оборачивает CSS в `<style>`, JS в `<script>`, встраивает `shell.html`
- [x] Генерирует `www/build/index.html` (96 KB)
- [x] `cargo:rerun-if-changed=www/src` — пересборка при изменении источников
- [x] `GET /` → встроенный `index.html` без внешних зависимостей
- [x] Итоговый файл < 500 KB ✅ (96 KB)
- [ ] `TEMPLATES` JS-объект — не реализован (страницы рендерят HTML напрямую через template literals)

### CSS (5 файлов) ✅
- [x] `variables.css` — тёмная тема, CSS custom properties
- [x] `reset.css` — box-sizing, scrollbar
- [x] `layout.css` — sidebar (220px fixed), #content, cards, page-header
- [x] `components.css` — кнопки, формы, тоглы, таблицы, badges, tabs, toast, modal, spinner, banner
- [x] `pages.css` — dashboard, networks, controllers, exitnode, settings, side-panel

### feat/part3-ui-shell ✅
- [x] `shell.html` — `#app`, `#sidebar`, `#content`, `#toast-container`, `#modal-overlay`
- [x] Навигация: Node (Dashboard, Peers), My Networks, Controllers (Networks, Members), Exit Node, Settings (Global, Root Servers, API Tokens)
- [x] Активный пункт подсвечивается через `data-route` + `.active`
- [x] `router.js` — hash-роутер с params, cleanup callbacks, `Router.navigate()`, `Router.on()`
- [x] `api.js` — fetch wrapper, бросает `{error, code}` на 4xx/5xx, нет axios
- [x] `state.js` — in-memory pub/sub, нет localStorage
- [x] `toast.js` — success/error/info, 4s, slide animation
- [x] `modal.js` — `Modal.confirm()` + `Modal.prompt()` → Promise

### feat/part3-ui-dashboard ✅
- [x] Node status bar: ID, online badge, version, world_id, tcp_fallback
- [x] Metrics cards: RX/TX traffic (fmtBytes), packets, latency (цвет), errors, last_updated
- [x] Banner при `metrics.enabled = false`
- [x] Peers table: Role badge (PLANET/MOON/LEAF), latency с цветом, paths count, version
- [x] Auto-refresh каждые 10s, destroy() + clearInterval при уходе

### feat/part3-ui-networks ✅
- [x] Объединение local + central networks в один список
- [x] Tabs: All | My Networks | Central
- [x] Join (input + POST) / Leave (confirm + DELETE)
- [x] Network Detail: tabs Details/Config, QR код (нативный Canvas), toggles
- [x] `qrcode.js` — Reed-Solomon V1-4, нет внешних либ

### feat/part3-ui-controllers ✅
- [x] Controller Networks: таблица, create (local/central), delete
- [x] Controller Members: таблица, side panel, auth toggle, IP assignments, advanced
- [x] Controller Config: Basics, Access, IPv4 (24 пула), IPv6 (rfc4193/6plane), Multicast, DNS, Add Member, Delete

### feat/part3-ui-exitnode ✅
- [x] Platform check → banner если не Linux
- [x] Deps step-list (nftables/iptables/is_root) + Install кнопка
- [x] Network + WAN interface selects
- [x] Enable/Disable с confirm
- [x] Status block

### feat/part3-ui-settings ✅
- [x] Global: Server (host/port), ZT Local (url/token_file), Metrics (toggle+url+interval), Exit Node (nftables toggle)
- [x] Root Servers: moons таблица, orbit форма, remove
- [x] API Tokens: token cards, verify-then-add flow, activate, delete

### Замечания
- Внешние ресурсы в index.html: 0 CDN/fonts, 3 вхождения `http://127.0.0.1:9993` (дефолтные значения полей) и 1 ссылка на docs.zerotier.com в `settings-roots.js` (открывается в новой вкладке — OK)
