# Changelog

All notable changes to ztnet-box are documented here.

Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).  
Versioning follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html).  
Version bumps are automated via [Conventional Commits](.github/COMMIT_CONVENTION.md).

---

## [Unreleased]

## [0.4.0] — 2026-04-07

### Chores
- docs(plan): PART-2 complete — all 5 branches merged, v0.3.0 (0ac1a72)


## [0.3.0] — 2026-04-06

### Chores
- docs(plan): update PART-2 — mark metrics/central/tokens complete, detail exitnode todo (f14d5ea)


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
