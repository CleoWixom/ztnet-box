# Changelog

All notable changes to ztnet-box are documented here.

Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).  
Versioning follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html).  
Version bumps are automated via [Conventional Commits](.github/COMMIT_CONVENTION.md).

---

## [Unreleased]

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
