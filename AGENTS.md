# AGENTS.md

## What this repo is

An AI agent **skill** (not an application). It teaches an AI agent to run the TradingAgents multi-agent stock analysis pipeline. Installable via `npx halfoffive/trad-skill`.

The core deliverable is `skills/tradingagents-analysis/` (SKILL.md + references/ + bin/). Rust source lives in `crates/trad-data/` with CI (cargo fmt/clippy/test + 6-platform cross-build) in `.github/workflows/ci.yml`.

## Structure

```
trad-skill/                        # repo root (meta files + installer)
├── package.json                  # npx entry point (name: trad-skill)
├── install.mjs                   # zero-dependency installer
├── bin/trad-data-wrapper.js      # cross-platform binary wrapper
├── README.md / README_CN.md       # bilingual docs with language switch
├── CHANGELOG.md                   # version history
├── AGENTS.md                      # this file
├── LICENSE                        # Apache 2.0
├── .github/workflows/ci.yml      # CI: fmt + clippy + test + 6-platform build
├── crates/trad-data/              # Rust binary source (trad-data)
└── skills/
    └── tradingagents-analysis/    # the installable skill
        ├── SKILL.md               # skill entry point (YAML frontmatter)
        ├── references/prompts/    # role prompts for each analyst
        ├── references/data-sources.md
        └── references/indicators.md
```

## Install commands

Recommended (universal, 70+ agents via [vercel-labs/skills](https://github.com/vercel-labs/skills)):
```bash
npx skills add halfoffive/trad-skill --skill tradingagents-analysis -g -y
```

Custom installer (legacy, Claude Code default):
```bash
npx halfoffive/trad-skill
```

## Installer (`package.json` + `install.mjs`)

- `bin.trad-skill` → `install.mjs`, a zero-dependency Node ESM script.
- Copies `skills/tradingagents-analysis/` into the target agent's skills dir. Default target: `~/.claude/skills/tradingagents-analysis` (Claude Code). Flags: `--dir <path>`, `--agent claude|agents|opencode`.
- Also copies `bin/` (trad-data Rust binary) into the skill directory.
- Idempotent (removes existing dir first). Prints next steps on success.
- The `files` field in `package.json` controls what npx packs: `install.mjs`, `bin/`, `skills/`, and the doc/LICENSE files.

## Source repos (read-only reference)

- `../TradingAgents` — TauricResearch original (pipeline, prompts, dataflows)
- `../TradingAgents-CN` — China market fork (A股/港股 analysts, Tushare/AKShare)

Prompts and methodology are distilled from these. When updating prompts, re-extract verbatim from source — never rewrite from memory.

## Gotchas

- `.omo/` is gitignored orchestration state — never commit it.
- `.codegraph/` exists for indexing — ignore it.
- `.trae/specs/` is the trae agent spec workflow state (spec.md / tasks.md / checklist.md), tracked in git; unlike `.omo/`, do NOT gitignore.
- `uv` is available; use `uv run python` for quick script checks if needed.
- Skill files live in `skills/tradingagents-analysis/` — this is the single source of truth.
- Rust source lives in `crates/trad-data/`. Run `cargo fmt`, `cargo clippy`, `cargo test` before committing.
- **`SKILL.md §2` makes the agent ask for the ticker first** when the user hasn't named one. Keep this prerequisite step.
