# AGENTS.md

## What this repo is

An AI agent **skill** (not an application). It teaches an AI agent to run the TradingAgents multi-agent stock analysis pipeline. Installable via `bunx trad-skill` (default target: `~/.agents/skills`); `npx trad-skill` works as a fallback.

The core deliverable is `skills/tradingagents-analysis/` (SKILL.md + references/ + bin/). Rust source lives in `crates/trad-data/` — the binary serves as both the **data tool** and the **installer** (`trad-data install` subcommand) — with CI (cargo fmt/clippy/test + 7-platform cross-build) in `.github/workflows/ci.yml`.

## Structure

```
trad-skill/                        # repo root (meta files + installer)
├── package.json                  # npm entry point (name: trad-skill)
├── bin/trad-skill.js             # thin JS launcher -> Rust installer
├── bin/trad-data-wrapper.js      # cross-platform binary wrapper (runtime)
├── README.md / README_CN.md       # bilingual docs with language switch
├── CHANGELOG.md                   # version history
├── AGENTS.md                      # this file
├── LICENSE                        # Apache 2.0
├── .github/workflows/ci.yml      # CI: fmt + clippy + test + 7-platform build
├── crates/trad-data/              # Rust binary source (trad-data)
└── skills/
    └── tradingagents-analysis/    # the installable skill
        ├── SKILL.md               # skill entry point (YAML frontmatter)
        ├── references/prompts/    # role prompts for each analyst
        ├── references/data-sources.md
        └── references/indicators.md
```

## Install commands

Recommended (Rust installer via bunx; default target `~/.agents/skills`):
```bash
bunx trad-skill                  # install to ~/.agents/skills (default)
bunx trad-skill --agent claude   # install to ~/.claude/skills
bunx trad-skill --dry-run        # print plan, write nothing
```

Fallback (no `bun`): `npx trad-skill` behaves identically.

Data tool without installing: `bunx trad-data stock --symbol AAPL` (also `news` / `fundamentals` / `sentiment`).

Deprecated: the third-party `npx skills add halfoffive/trad-skill ...` flow (vercel-labs/skills) still works but is no longer recommended.

## Installer (`package.json` + `bin/trad-skill.js` + `trad-data install`)

The installer logic is **Rust**, in the `trad-data install` subcommand (`crates/trad-data/src/install.rs`). The npm `trad-skill` bin entry is a zero-logic CJS launcher (`bin/trad-skill.js`) that resolves the platform binary (same two-strategy logic as `bin/trad-data-wrapper.js`) and execs `trad-data install --skills-dir <pkgRoot>/skills/tradingagents-analysis`.

- Copies `skills/tradingagents-analysis/` into the target agent's skills dir. **Default target: `~/.agents/skills`** (generic agent directory). Flags: `--dir <path>`, `--agent claude|agents|opencode` (mutually exclusive).
- Copies `bin/trad-data-wrapper.js` into the skill's `bin/`, and copies the platform binary into `bin/<node-platform-key>/` via **self-copy** (`std::env::current_exe()`). `--bin-path` / `--no-bin` override.
- Idempotent (removes existing dir first). `--dry-run` prints the plan without writing.
- `node_platform_key()` in `install.rs` maps rustc target consts → node platform/arch naming; this key MUST match `bin/trad-data-wrapper.js` strategy-1 path. Keep them in sync (unit-tested).
- The `files` field in `package.json` controls what bunx/npx packs: `bin/`, `skills/`, and the doc/LICENSE files.

## Source repos (read-only reference)

- `../TradingAgents` — TauricResearch original (pipeline, prompts, dataflows)
- `../TradingAgents-CN` — China market fork (A股/港股 analysts, Tushare/AKShare)

Prompts and methodology are distilled from these. When updating prompts, re-extract verbatim from source — never rewrite from memory.

## Coding Standards

### Rust
- All code must pass `cargo fmt --check`, `cargo clippy -- -D warnings`, and `cargo test` before committing.
- Use `anyhow` for error handling. No `unwrap()`/`expect()` in non-test code.
- All HTTP clients must use `rustls-tls`. Never introduce `native-tls` or `openssl` (breaks cross-compilation).
- No `cfg(target_os)` platform-specific code unless covering all 7 build targets.
- No C-dependent system crates (e.g. `openssl-sys`, `sqlite3-sys`).

### Cross-compilation compatibility
New dependencies must support all 7 CI targets:
- linux-{gnu,musl} x {x86_64,aarch64}
- apple-darwin aarch64
- windows-msvc x {x86_64,aarch64}

### Build targets (7)
x86_64-unknown-linux-gnu, aarch64-unknown-linux-gnu, x86_64-unknown-linux-musl, aarch64-unknown-linux-musl, aarch64-apple-darwin, x86_64-pc-windows-msvc, aarch64-pc-windows-msvc.

When adding/removing targets, update: `.github/workflows/ci.yml`, `.github/workflows/release.yml`, `bin/trad-data-wrapper.js`.

### Release
- Version lives in **7 files that must stay in sync** (a desync silently breaks platform package install):
  `package.json` (+ its 5 `optionalDependencies` `@trad-skill/*` pins), `crates/trad-data/Cargo.toml`, and all 5 `npm/<platform>/package.json`.
- Update `CHANGELOG.md` before release.
- Push tag `vX.Y.Z` to trigger the release workflow (`.github/workflows/release.yml`). CI builds the same 7 binaries; the `install` subcommand ships with them — no new artifacts.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for the branching model (GitHub Flow), Conventional Commits conventions, PR checklist, and the release 7-file version-bump procedure.

## Gotchas

- `.omo/` is gitignored orchestration state — never commit it.
- `.codegraph/` exists for indexing — ignore it.
- `.trae/` (Trae IDE state) is gitignored and not tracked — do not commit it.
- `uv` is available; use `uv run python` for quick script checks if needed.
- Skill files live in `skills/tradingagents-analysis/` — this is the single source of truth.
- Rust source lives in `crates/trad-data/`. Run `cargo fmt`, `cargo clippy`, `cargo test` before committing.
- **`SKILL.md §2` makes the agent ask for the ticker first** when the user hasn't named one. Keep this prerequisite step.
