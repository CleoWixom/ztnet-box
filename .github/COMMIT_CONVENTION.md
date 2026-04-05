# Conventional Commits — соглашение для проекта ztnet-box
#
# Формат:  <type>(<scope>): <description>
#          [пустая строка]
#          [body — опционально]
#          [пустая строка]
#          [footer — BREAKING CHANGE: ... или Closes #N]
#
# Примеры:
#   feat(backend): add ZT Central API token validation
#   fix(exitnode): correct nftables POSTROUTING rule on disable
#   perf(metrics): use RwLock instead of Mutex for cache reads
#   chore(deps): update axum to 0.7.5
#   docs: add Security Model section to README          ← не триггерит CI
#   feat!: redesign config schema                       ← MAJOR bump
#
# Types:
#   feat     → new feature                             (minor bump)
#   fix      → bug fix                                 (patch bump)
#   perf     → performance improvement                 (patch bump)
#   refactor → code change, no feature/fix             (patch bump)
#   test     → tests only                              (patch bump)
#   build    → build system, Cargo.toml changes        (patch bump)
#   ci       → CI/CD workflow changes                  (NO bump)
#   docs     → documentation only                      (NO bump — не триггерит CI)
#   style    → formatting, whitespace                  (NO bump)
#   chore    → maintenance, deps update                (patch bump)
#   revert   → revert a previous commit                (patch bump)
#
# Breaking change → MAJOR bump:
#   feat!: ...
#   fix!: ...
#   BREAKING CHANGE: <description> в footer
#
# Scopes (рекомендуемые):
#   backend    src/ общее
#   config     src/config/
#   server     src/server/
#   local-api  src/zerotier/local/
#   central    src/zerotier/central/
#   tokens     src/zerotier/central/token_store.rs
#   metrics    src/metrics/
#   exitnode   src/exitnode/
#   detection  src/zerotier/detection.rs
#   frontend   www/
#   build      build.rs
#   deps       Cargo.toml зависимости
#   release    версионирование, релиз
#   ci         .github/workflows/
#   plan       plan/ (не триггерит CI)
