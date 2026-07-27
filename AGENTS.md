# AGENTS.md

## What this repo is

An AI agent **skill** (not an application). It teaches an AI agent to run the TradingAgents multi-agent stock analysis pipeline. Installable via `npx halfoffive/trad-skill`.

The core deliverable is `tradingagents-analysis/` (SKILL.md + references/ + bin/). There is no build, no test suite, no CI.

## Structure

```
trad-skill/                        # repo root (meta files + installer)
├── package.json                  # npx entry point (name: trad-skill)
├── install.mjs                   # zero-dependency installer
├── README.md / README_CN.md       # bilingual docs with language switch
├── CHANGELOG.md                   # version history
├── AGENTS.md                      # this file
├── LICENSE                        # Apache 2.0
├── skills/                        # ← vercel-labs/skills standard location
│   └── tradingagents-analysis/    # ← the installable skill (standard path)
└── tradingagents-analysis/        # ← the installable skill (root copy, backward compat)
    ├── SKILL.md                   # skill entry point (YAML frontmatter). Keep under 500 lines.
    ├── references/prompts/        # VERBATIM prompts from source repos. Do NOT paraphrase.
    ├── references/data-sources.md # data source catalog
    ├── references/indicators.md   # technical indicator reference
    └── bin/                       # trad-data Rust binary (platform-specific)
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
- Copies the root `tradingagents-analysis/` copy into the target agent's skills dir. Default target: `~/.claude/skills/tradingagents-analysis` (Claude Code). Flags: `--dir <path>`, `--agent claude|agents|opencode`.
- Idempotent (removes existing dir first). Prints next steps on success.
- The `files` field in `package.json` controls what npx packs: `install.mjs`, both `tradingagents-analysis/` locations (`skills/` + root copy), and the doc/LICENSE files. `.omo/` and `.codegraph/` are never packed.
- Note: The standard `npx skills add` (vercel-labs/skills) is now the recommended universal method. The custom installer (`npx halfoffive/trad-skill`) is preserved for backward compatibility. Both locations (`skills/tradingagents-analysis/` and root `tradingagents-analysis/`) contain identical copies — the npm `files` field packs both.

## Source repos (read-only reference)

- `../TradingAgents` — TauricResearch original (pipeline, prompts, dataflows)
- `../TradingAgents-CN` — China market fork (A股/港股 analysts, Tushare/AKShare)

Prompts and methodology are distilled from these. When updating prompts, re-extract verbatim from source — never rewrite from memory.

## Gotchas

- `.omo/` is gitignored orchestration state — never commit it.
- `.codegraph/` exists for indexing — ignore it.
- `.trae/specs/` is the trae agent spec workflow state (spec.md / tasks.md / checklist.md), tracked in git; unlike `.omo/`, do NOT gitignore.
- `uv` is available with bundled Python; use `uv run python` for script checks.
- No `requirements.txt` by design — deps are documented in README only.
- Skill files live in `tradingagents-analysis/` subfolder — repo root is for meta files only.
- Skill files now also live in `skills/tradingagents-analysis/` for vercel-labs/skills CLI discovery; root copy kept for install.mjs backward compat.
- **`SKILL.md §2` makes the agent ask for the ticker first** when the user hasn't named one. Keep this prerequisite step.
