# Changelog

All notable changes to ztnet-box are documented here.

Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).  
Versioning follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html).  
Version bumps are automated via [Conventional Commits](.github/COMMIT_CONVENTION.md).

---

## [Unreleased]

## [0.7.1] — 2026-04-12

### Features
- feat(bridge): `BridgeConfig` + `BridgeState` types in `src/bridge/`
- feat(bridge): `deps::check()` — iproute2, systemd-networkd, dhcpcd/ifupdown conflict detection; `install()` removes conflicts and enables networkd
- feat(bridge): `rules::apply()` — `ip link` bridge setup + systemd-networkd `.netdev`/`.network` unit files for persistence
- feat(bridge): `rules::remove()` — detach members, delete bridge, remove unit files; all linux helpers gated under `#[cfg(target_os = "linux")]`
- feat(bridge): 6 REST handlers: platform, deps, deps/install, status, enable, disable
- feat(bridge): `bridge_state: Arc<RwLock<BridgeState>>` in AppState; physnet conflict check now uses real bridge state
- feat(frontend): `bridge.js` — full UI with deps checklist, config form, status card, ZT Central instructions

### Tests
- test(bridge): 4 unit tests (config roundtrip, no-addr config, unsupported platform)
- test(bridge): 4 integration tests (platform, deps, status structure, invalid network_id 422)

## [0.7.0] — 2026-04-11

### Features
- feat(logs): `LogCollector` — in-process ring buffer (500 entries) + `broadcast::Sender` (256 cap)
- feat(logs): `CollectorLayer` — custom `tracing::Layer` wired into `tracing_subscriber::registry()` in main.rs
- feat(logs): `GET /api/logs` — buffered entries with `?level=` and `?limit=` query params
- feat(logs): `GET /api/logs/stream` — SSE live stream via `BroadcastStream`
- feat(logs): `GET/PUT /api/logs/level` — read/set minimum capture level at runtime
- feat(logs): `DELETE /api/logs` — clear ring buffer
- feat(logs): `LogPanel` frontend sidebar — toggle open/close, SSE start/stop, substring filter, level selector, colour-coded rows

### Tests
- test(logs): 7 unit tests in `log_collector.rs` (push, filter, clear, ring eviction, entry filter, parse, subscribe)
- test(logs): 5 integration tests (GET array, GET/PUT level, PUT invalid 422, DELETE)

## [0.6.5] — 2026-04-11

### Features
- feat(exitnode): IPv6 ip6tables support — `enable_ipv6` + `ipv6_prefix` fields in `ExitNodeRules`, `ExitNodeState`, `EnableRequest`
- feat(exitnode): `with_ipv6(enable, prefix)` builder on `ExitNodeRules` — backward-compatible
- feat(exitnode): `enable_ipv6_forward()` — writes `/proc/sys/net/ipv6/conf/all/forwarding` + sysctl.conf persist
- feat(exitnode): `apply_ipv6_forwarding()` — ip6tables stateful FORWARD + NAT MASQUERADE rules
- feat(exitnode): `remove_ipv6_rules()` — clean removal of ip6tables rules on disable
- feat(exitnode): `ip6tables` path + `ipv6_forward_enabled` added to `DepsStatus`
- feat(exitnode): CIDR validation for `ipv6_prefix` in handler; IPv6-specific warnings in enable response
- feat(frontend): ip6tables dep step in Exit Node checklist; IPv6 checkbox + prefix field; IPv6 row in Status card

### Tests
- test(exitnode): `with_ipv6_builder`, `with_ipv6_no_prefix`, `ipv6_forward_disabled_by_default` unit tests
- test(exitnode): `exitnode_deps_returns_structure` extended with `ip6tables`/`ipv6_forward_enabled` assertions
- test(exitnode): `exitnode_enable_with_invalid_ipv6_prefix_returns_422` integration test
- test(exitnode): `exitnode_status_includes_ipv6_fields` integration test

## [0.6.3] — 2026-04-08

### Chores
- Merge pull request #12 from CleoWixom/feat/part4-security (f29c75b)
- docs(plan): add updates.md — comprehensive feature backlog from ZT docs audit (74e50b8)


## [0.6.2] — 2026-04-08


## [0.6.1] — 2026-04-08


## [0.6.0] — 2026-04-08


### Security
- fix: complete Content-Security-Policy — add `img-src 'self' data:` (required for QR canvas) and `connect-src 'self'`
- feat: add `Referrer-Policy: no-referrer` security header to all responses
- feat: log `WARN` at startup when server is bound to a non-loopback address
- feat: validate all path parameters (network_id: 16 hex, node_id: 10 hex, world_id: 1–16 hex) — invalid values return 422
- feat: limit request body to 64 KB (`DefaultBodyLimit`) — oversized bodies return 413
- docs: document security model in README (no-auth rationale, bind to localhost, CSRF not applicable)

### Features
- feat: `CentralToken.token` annotated — never serialized into API responses (only via `masked_token()`)
- feat: `validate` module with `network_id`, `node_id`, `world_id`, `ip_addr`, `cidr` helpers

### Tests
- test: `invalid_network_id_returns_422` — bad path param rejected before ZT call
- test: `invalid_node_id_in_peer_returns_422`
- test: `oversized_body_returns_413` — 65 KB body exceeds 64 KB limit
- test: `Referrer-Policy` header asserted in security header tests
- test: CSP `img-src` and `connect-src` directives asserted

### Docs
- docs: write full README with installation, configuration table, security model, Central API tokens, Exit Node sections
- docs: fill CHANGELOG [Unreleased] with all PART 4 changes

## [0.5.1] — 2026-04-07



## [0.2.0] — 2026-04-06

### Features
- Prometheus parser, typed cache, REST API + fix(ci) version.yml (#8) (48959a0)
- Central API token management CRUD (#7) (250267b)
- ZeroTier Central API client & REST handlers (#6) (901a1c6)
- ZeroTier local service client & REST handlers (#5) (0d50100)
- complete HTTP server with tests (#4) (8264480)
- complete ZeroTier detection & install module (#3) (5395bf8)
- REST API for config read/update (#2) (77a1256)
- initial project scaffold (#1) (f1e7019)

### Chores
- docs(plan): mark completed tasks in PART-1 and PART-2 (8d964ad)
- ci: add workflows for CI, version bump, release and PR validation (905abce)
- plan: restructure into 4 parts with task checklists, add token management, no-auth model (0b13056)
- plan: add full implementation plan with branch map (e83a396)
- Initial commit (49e9089)


<!-- Version sections are auto-inserted here by the version workflow -->

---

## Version Rules (automated)

| Commit type | Version bump |
|---|---|
| `feat!:` / `BREAKING CHANGE:` | Major (x.0.0) |
| `feat:` | Minor (0.x.0) |
| `fix:`, `perf:`, `refactor:`, `chore:`, `test:`, `build:` | Patch (0.0.x) |
| `docs:`, `ci:`, `style:`, `plan:` | No bump |
