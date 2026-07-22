# AGENTS.md

## What this repo is

An AI agent **skill** (not an application). It teaches an AI agent to run the TradingAgents multi-agent stock analysis pipeline. Installable via `npx skill add`.

The core deliverable is `tradingagents-analysis/` (SKILL.md + references/ + scripts/). There is no build, no test suite, no CI.

## Structure

```
trad-skill/                        # repo root (meta files)
├── README.md / README_CN.md       # bilingual docs with language switch
├── CHANGELOG.md                   # version history
├── AGENTS.md                      # this file
├── LICENSE                        # Apache 2.0
└── tradingagents-analysis/        # ← the installable skill
    ├── SKILL.md                   # skill entry point (YAML frontmatter). Keep under 500 lines.
    ├── references/prompts/        # VERBATIM prompts from source repos. Do NOT paraphrase.
    ├── references/data-sources.md # data source catalog
    ├── references/indicators.md   # technical indicator reference
    └── scripts/*.py               # Python data-fetching helpers
```

## Install command

```bash
npx skill add halfoffive/trad-skill/tradingagents-analysis
```

## Python script conventions

- **Functional only** — no `class` keyword anywhere in `scripts/`.
- **Chinese comments** — all `#` comments and docstrings in Chinese.
- Dependencies: `yfinance`, `akshare`, `requests`, `pandas` (no other third-party).
- Every function returns a formatted string (for LLM prompt injection), never raises.
- Each script has an `argparse` CLI block under `if __name__ == "__main__":`.
- Syntax check: `uv run python -c "import ast; ast.parse(open(f, encoding='utf-8').read())"`

## Source repos (read-only reference)

- `../TradingAgents` — TauricResearch original (pipeline, prompts, dataflows)
- `../TradingAgents-CN` — China market fork (A股/港股 analysts, Tushare/AKShare)

Prompts and methodology are distilled from these. When updating prompts, re-extract verbatim from source — never rewrite from memory.

## Gotchas

- `.omo/` is gitignored orchestration state — never commit it.
- `.codegraph/` exists for indexing — ignore it.
- `uv` is available with bundled Python; use `uv run python` for script checks.
- No `requirements.txt` by design — deps are documented in README only.
- Skill files live in `tradingagents-analysis/` subfolder — repo root is for meta files only.
