# ZeroBox WebUI — Plan Overview

**ZeroBox** — локальный web-интерфейс для управления ZeroTier: создание/подключение/настройка сетей, управление участниками, Exit Node (VPN), root-серверы.

> **Стек:** Rust (backend) · HTML5/CSS3/JS (frontend, собирается в один `index.html`, встроен в бинарник)  
> **Конфиг:** `config.yml` + ENV override  
> **Авторизация:** отсутствует — локальный интерфейс, безопасность обеспечивается привязкой к `127.0.0.1` по умолчанию

---

## Архитектура (Bird's Eye)

```
┌─────────────────────────────────────────────────────┐
│                  Browser (localhost)                 │
│           Single-page App (index.html)               │
│   Hash-router · fetch API · no framework             │
└────────────────────┬────────────────────────────────┘
                     │ HTTP/REST JSON
┌────────────────────▼────────────────────────────────┐
│              Rust Binary (Axum)                      │
│  ┌──────────┐ ┌────────────┐ ┌────────────────────┐ │
│  │  Config  │ │ ZT Detect  │ │   Static (embed)   │ │
│  │  Module  │ │  & Install │ │   GET / → index    │ │
│  └──────────┘ └────────────┘ └────────────────────┘ │
│  ┌──────────────────────────────────────────────┐    │
│  │              REST API /api/*                  │    │
│  │  /local/*  /central/*  /metrics  /exitnode   │    │
│  │  /settings/tokens  /settings/config           │    │
│  └──────┬───────────────────┬───────────────────┘    │
└─────────┼───────────────────┼─────────────────────── ┘
          │                   │
┌─────────▼──────┐   ┌────────▼──────────────────────┐
│ zerotier-one   │   │   api.zerotier.com             │
│ :9993          │   │   (Central API, per-token)      │
│ Local Service  │   │   Multiple tokens supported     │
└────────────────┘   └───────────────────────────────┘
```

---

## Принципиальные решения

| Аспект | Решение | Обоснование |
|---|---|---|
| Авторизация | Нет | Локальный UI, bind `127.0.0.1` по умолчанию |
| Хранение токенов | `config.yml` + ENV override | Нет внешних БД, файл на сервере закрыт правами ОС |
| Несколько Central токенов | Список в конфиге, один активный | Поддержка нескольких аккаунтов ZT Central |
| Встраивание фронта | `include_str!` в бинарнике | Один исполняемый файл без зависимостей |
| ZT установка | Нативный Rust, без curl/shell | Требование: нет внешних CLI-инструментов |
| Metrics парсинг | Собственный парсер Prometheus | Нет избыточных зависимостей |
| Exit Node | Нативный Linux (iptables/nftables via Rust) | Кроссплатформенная проверка + graceful fallback |

---

## Структура репозитория

```
ztnet-box/
├── Cargo.toml
├── build.rs                    # сборка фронтенда → www/build/index.html
├── config.yml.example
├── CHANGELOG.md
├── README.md
├── plan/
│   ├── README.md               # (этот файл)
│   ├── PART-1-infrastructure.md
│   ├── PART-2-backend.md
│   ├── PART-3-frontend.md
│   └── PART-4-integration.md
├── src/
│   ├── main.rs
│   ├── config/                 # загрузка, схема, сохранение config.yml
│   │   ├── mod.rs
│   │   ├── schema.rs
│   │   └── env.rs
│   ├── zerotier/
│   │   ├── detection.rs        # поиск/установка zerotier-one, zerotier-idtool
│   │   ├── local/              # ZT One Service API клиент
│   │   │   ├── client.rs
│   │   │   └── types.rs
│   │   └── central/            # ZT Central API клиент
│   │       ├── client.rs
│   │       ├── token_store.rs  # управление набором токенов
│   │       └── types.rs
│   ├── server/
│   │   ├── router.rs
│   │   ├── state.rs
│   │   ├── error.rs
│   │   └── middleware.rs
│   ├── metrics/
│   │   ├── collector.rs
│   │   ├── parser.rs
│   │   └── cache.rs
│   └── exitnode/
│       ├── platform.rs
│       ├── deps.rs
│       ├── interfaces.rs
│       └── rules.rs
└── www/
    ├── src/
    │   ├── html/               # шаблоны страниц
    │   ├── css/                # стили (модульно)
    │   └── js/                 # логика страниц + компоненты
    └── build/
        └── index.html          # артефакт сборки (gitignore)
```

---

## Ветки и последовательность

```
main
 ├── feat/part1-scaffold              ─┐
 ├── feat/part1-config                 │ PART 1
 ├── feat/part1-zt-detection           │ Infrastructure
 └── feat/part1-http-server           ─┘
      ├── feat/part2-zt-local-api     ─┐
      ├── feat/part2-zt-central-api    │ PART 2
      ├── feat/part2-token-store       │ Backend
      ├── feat/part2-metrics           │ Modules
      └── feat/part2-exitnode         ─┘
           ├── feat/part3-build-pipeline ─┐
           ├── feat/part3-ui-shell        │ PART 3
           ├── feat/part3-ui-dashboard    │ Frontend
           ├── feat/part3-ui-networks     │
           ├── feat/part3-ui-controllers  │
           ├── feat/part3-ui-exitnode     │
           └── feat/part3-ui-settings    ─┘
                ├── feat/part4-integration ─┐
                ├── feat/part4-security     │ PART 4
                └── feat/part4-release     ─┘
                     └── release/v0.1.0
```

Каждая ветка завершается PR в `main`. Ветки внутри одной части последовательны.

---

## Документы плана

| Файл | Содержание |
|---|---|
| [PART-1-infrastructure.md](./PART-1-infrastructure.md) | Scaffold, Config, ZT Detection, HTTP Server |
| [PART-2-backend.md](./PART-2-backend.md) | ZT Local API, Central API, Token Store, Metrics, Exit Node |
| [PART-3-frontend.md](./PART-3-frontend.md) | Build pipeline, все страницы UI |
| [PART-4-integration.md](./PART-4-integration.md) | Тесты, безопасность, сборка, релиз |
