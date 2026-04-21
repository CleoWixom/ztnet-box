# Changelog

All notable changes to ztnet-box are documented here.

Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).  
Versioning follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html).  
Version bumps are automated via [Conventional Commits](.github/COMMIT_CONVENTION.md).

---

## [Unreleased]

## [0.9.3] — 2026-04-21


## [0.9.2] — 2026-04-20

### Bug Fixes
- replace networkidle with fixed 2s wait — prevents hang (2f63be4)

### Chores
- docs(audit): audit-3 — screenshots workflow root causes documented (SCR-1..7) (e2fb0a0)
- Updated New tasks (6a67a03)
- chore(ci): update actions/checkout, cache, setup-node v4→v5 (893aa93)


## [0.9.1] — 2026-04-18

### Chores
- Enhance manual-testing.yml with base_url input (eac411f)


## [0.9.0] — 2026-04-18

### Bug Fixes
- 6 root causes fixed — ZT daemon, wait logic, sudo, mobile, routing (44f452c)

### Chores
- Modify ZeroTier service management in workflow (41f8250)
- Update workflow to install and start ZeroTier (c8574da)
- Merge pull request #19 from CleoWixom/chore/screenshots-20260417-233429 (da0a1e4)
- docs(screenshots): update WebUI screenshots [skip ci] (5b4b38e)
- Disable cargo caching in screenshots workflow (9fa9c82)
- Merge pull request #18 from CleoWixom/chore/screenshots-20260417-205959 (e13bae3)
- docs(screenshots): update WebUI screenshots [skip ci] (079659d)


## [0.8.2] — 2026-04-17

### Chores
- docs: rewrite README + actualize AUDIT.md (v0.8.0) (73c0f7b)


## [0.8.1] — 2026-04-17


## [0.8.0] — 2026-04-17

### Security
- SSH relay: removed `sshpass` dependency — key-based auth only (`BatchMode=yes` enforces it)
- SSH relay: `StrictHostKeyChecking=no` → `accept-new` (MITM protection after first connect)
- Docker install: replaced `curl -fsSL https://get.docker.com | sh` with apt/dnf/pacman
- CSP: tightened `connect-src *` → `connect-src 'self'`
- ZT local client: `danger_accept_invalid_certs` is now conditional on is_loopback

### Features
- Mobile UI: `@media (max-width:768px)` responsive layout, off-canvas sidebar, hamburger toggle
- Dashboard: ZeroTier install detection banner + Install button (uses `/api/system/zt-status`)
- Settings: `metricstoken_file` path now configurable in Global Settings UI
- Peers page: full standalone page with live `api.get('/local/peers')` (was inline stub)
- Exit node: `zt_network_id` properly stored in `ExitNodeState` (separate from `zt_interface`)
- State persistence: bridge/physnet/relay state survives restarts via `runtime_state.rs`
- Rate limiter: `RateLimiter::acquire()` now uses `.forget()` — true token-bucket semantics

### Bug Fixes
- `log-panel.js` moved to `components/` so it's included in the build bundle (was missing)
- `controllers-config.js`: `api.post` → `api.put` for local controller network update
- `update_token` handler: UUID preserved on rename (was destroy+recreate)
- `rand_byte()`: replaced `/dev/urandom` + `0xAB` fallback with `getrandom::getrandom()`
- N+1 requests in controllers-networks: parallel `Promise.allSettled()`
- `_esc()` deduplication: single `Utils.esc()` in `state.js`; log-panel CSS vars aligned
- NDP `install/enable/disable`: split into `#[cfg(target_os="linux")]` overloads (cross-platform CI)
- Relay handler: removed stale `cfg.password` check after password field removal

### Chores
- `PhysNetStateArc` dead type alias removed
- `#[allow(clippy::derivable_impls)]` → `#[derive(Default)]` for `Config` + `ZeroTierConfig`
- Log panel: all CSS variables aligned with `variables.css` (`--c-*` names)


## [0.7.6] — 2026-04-12

### Features
- feat(ndp): `src/exitnode/ndp.rs` — `check_status()`, `install()`, `enable(cfg)`, `disable(remove_config)`
- feat(ndp): generates `/etc/ndppd.conf` with proxy rule for zt+ interface; `systemctl enable --now ndppd`
- feat(ndp): apt/dnf/pacman package manager detection for install; all linux-only fns under `#[cfg(linux)]`
- feat(ndp): 4 REST endpoints: GET status, POST install, POST enable, POST disable
- feat(frontend): NDP Proxy section in Exit Node page — status card, install/enable/disable, WAN+prefix form

### Tests
- test(ndp): check_status_does_not_panic, ndp_config_fields, unsupported platform (non-linux)
- test(ndp): ndp_status_returns_structure, ndp_enable_invalid_cidr_returns_422

## [0.7.5] — 2026-04-12

### Features
- feat(screenshots): `.github/workflows/screenshots.yml` — Playwright captures 10 pages × 2 viewports; opens PR on changes
- feat(screenshots): `docs/screenshots/README.md` — screenshot index table

## [0.7.4] — 2026-04-12

### Features
- feat(pkg): `.github/workflows/packages.yml` — builds .deb/.rpm/.pkg.tar.zst/.msi on version tags
- feat(pkg): `pkg/debian/postinst` + `prerm` — systemd service lifecycle scripts
- feat(pkg): `pkg/lib/systemd/system/ztnet-box.service` — hardened systemd unit
- feat(pkg): `pkg/homebrew/ztnet-box.rb` — Homebrew formula for macOS/Linux
- feat(pkg): `Cargo.toml` `[package.metadata.deb]` and `[package.metadata.generate-rpm]` metadata

## [0.7.2] — 2026-04-12

### Features
- feat(relay): `RelayStatus`, `LocalRelayConfig`, `RemoteRelayInfo`, `RelayDeployConfig` types
- feat(relay): `SshClient` — thin wrapper around system `ssh`/`sshpass` binary
- feat(relay): `deploy::deploy()` — SSH → install Docker → stop UFW → run pylon reflect container
- feat(relay): `deploy::remove()` — stop/rm pylon container via SSH
- feat(relay): `deploy::verify()` — TCP connect reachability check
- feat(relay): 5 REST handlers: GET status, PUT local (ip/port validation), POST deploy (spawn_blocking), GET verify, DELETE remote
- feat(relay): auto-update `local.conf` tcp_fallback_relay after deploy/remove
- feat(relay): `relay_remote: Arc<RwLock<Option<RemoteRelayInfo>>>` in AppState
- feat(frontend): `relay.js` — local config form, SSH deploy form, remote status card with Verify/Remove

### Tests
- test(relay): ssh_client_fields, verify_unreachable_host, relay_deploy_config_defaults
- test(relay): relay_status_returns_structure, relay_local_invalid_endpoint_returns_422, relay_deploy_missing_host_returns_422, relay_verify_no_relay_returns_not_reachable


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
