# ADR-0009: Cargo disk budget

Status: accepted (2026-10-19). The 15 GiB cap is an estimate. Re-measure after the first full build of the workspace.

## Context

Cargo never deletes anything from `target/` on its own. Its built-in cleanup
(`cache.auto-clean-frequency`, stable since 1.88, default one day) only cleans
`$CARGO_HOME`, and the request to do the same for `target/` is still open
([cargo#13136](https://github.com/rust-lang/cargo/issues/13136)). So `target/`
only grows, and the project will build a large workspace with heavy
dependencies (Cranelift, wasmtime, optional LLVM).

Each of these multiplies the size:

- **Profiles and triples.** `dev` and `test` build into `debug/`, `release` and `bench` into `release/`, and every custom profile and `--target` triple gets another full tree.
- **Feature sets.** Cargo builds a crate again for every distinct feature set. The same dependency can appear several times.
- **Toolchains.** Artifacts of an old rustc are never reused and never removed.
- **Incremental caches.** Dev builds keep incremental state next to every crate.
- **Debuginfo.** Full debuginfo is the largest part of a dev binary and of each test binary.
- **Other consumers.** rust-analyzer and `cargo clippy` also fill `target/`.

See the [build cache](https://doc.rust-lang.org/cargo/reference/build-cache.html)
and [profiles](https://doc.rust-lang.org/cargo/reference/profiles.html) references.

## Decision

1. **Pin the toolchain.** `rust-toolchain.toml` fixes the channel to 1.99.1 with the `minimal` profile plus `rustfmt`, `clippy` and `rust-src`. One toolchain means one set of artifacts. A bump is a deliberate commit. LLD is the default linker on `x86_64-unknown-linux-gnu` since [Rust 1.90](https://blog.rust-lang.org/2025/09/18/Rust-1.90.0/), so no linker settings are needed.
2. **Profile block.** The workspace `Cargo.toml` gets this block when it is created:

   ```toml
   [profile.dev]
   debug = "line-tables-only"
   [profile.dev.package."*"]
   debug = false
   opt-level = 2
   [profile.dev.build-override]
   debug = false
   [profile.release]
   debug = "line-tables-only"
   strip = "debuginfo"
   ```

3. **Declare dependencies once.** Every dependency lives in `[workspace.dependencies]` with its features, and members use `dep.workspace = true`. This keeps one feature set per dependency. Cross-package feature unification (`resolver.feature-unification`, [cargo#14774](https://github.com/rust-lang/cargo/issues/14774)) is still nightly, so it is not used.
4. **No custom profiles.** Each custom profile is another full tree. Only `dev`, `release`, `test` and `bench` exist.
5. **LLVM is off by default.** The LLVM backend is an optional feature, and it links against the system LLVM instead of building it. See [ADR-0001](0001-implementation-language.md).
6. **rust-analyzer shares `target/`.** No separate target directory for the editor. `.cargo/config.toml` pins `build.target-dir` and `build.build-dir` to the repo's `target/` so a user-level config cannot move them. Set no `CARGO_TARGET_DIR`, because the variable overrides the file.
7. **Budget script and timer.** `tools/cargo-budget.sh <max-gib>` wipes `target/` when the rustc version differs from the stamp in `target/.rustc-version`, or when `target/` is larger than the cap. The cap is 15 GiB. A user timer runs it daily with `Persistent=true`, so a missed run happens at the next login.

   Script behavior worth knowing:
   - No `target/`: exit 0, nothing to do.
   - rustc not on `PATH`: only the size check runs. The stamp is neither read nor written, so the script never wipes on every run.
   - `target/` exists with no stamp: the stamp is written and nothing is wiped.
   - Wipes are complete. There is no partial cleaning, so a wipe costs one full rebuild.

   Install (run from the repo root):

   ```sh
   mkdir -p ~/.config/systemd/user
   sed "s|@REPO@|$PWD|" tools/systemd/verse-cargo-budget.service > ~/.config/systemd/user/verse-cargo-budget.service
   cp tools/systemd/verse-cargo-budget.timer ~/.config/systemd/user/
   systemctl --user daemon-reload
   systemctl --user enable --now verse-cargo-budget.timer
   ```

   `systemctl --user link` is not used. It symlinks the unit from the repo, so the `@REPO@` placeholder could not be filled without editing a tracked file. Rerun the install after moving the repo.

## Alternatives

| Option | For | Against |
|--------|-----|---------|
| **Pinned toolchain, trimmed profiles, size-cap script** | no dependencies; one directory to cap; fixes the causes and bounds the rest | wipe is all or nothing; the cap needs re-measuring |
| [cargo-sweep](https://github.com/holmgr/cargo-sweep) | removes only stale artifacts by age or toolchain | its README says it is unmaintained; one more tool to install |
| sccache | shares compiled output between projects | adds its own cache of about 10 GB; does not cache incremental builds; `target/` still grows |
| tmpfs for `target/` | no disk use; fast | uses RAM; every boot is a full rebuild |
| Shared global target dir | one cache for all projects | projects with different features and toolchains thrash each other; one lock for every build; harder to cap per project |
| cargo-hakari (workspace-hack crate) | unifies features across the workspace | extra generated crate and churn; not justified until the duplication is measured |
| `build.build-dir` relocation (stable since 1.91) | keeps intermediates apart from final artifacts | two directories to cap and clean; no gain until a measured need |

## Consequences

- Dev builds have less debuginfo. Backtraces still show file and line. Use a local override with `debug = true` for a debugger session, and expect a larger `target/` until the next wipe.
- Dependencies are built with `opt-level = 2` in dev, so the first build is slower and later builds are faster.
- A wipe or a toolchain bump costs one full rebuild.
- The cap must be re-measured after the first full build of all workspace targets, and `15` updated in the service unit.
- Each developer installs the timer once. Nothing else enforces the cap.

## References

- [Cargo config: cache](https://doc.rust-lang.org/cargo/reference/config.html#cache)
- [cargo#13136: clean `target/` automatically](https://github.com/rust-lang/cargo/issues/13136)
- [Cargo build cache](https://doc.rust-lang.org/cargo/reference/build-cache.html)
- [Cargo profiles](https://doc.rust-lang.org/cargo/reference/profiles.html)
- [cargo#14774: feature unification](https://github.com/rust-lang/cargo/issues/14774)
- [Rust 1.90.0 release notes](https://blog.rust-lang.org/2025/09/18/Rust-1.90.0/)
- [cargo-sweep](https://github.com/holmgr/cargo-sweep)
