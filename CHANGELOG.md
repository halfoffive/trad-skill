# Changelog

All notable changes to this project will be documented in this file.

## [1.1.0] - 2026-07-23

### Added
- `package.json` + `install.mjs`: a self-contained, zero-dependency npx installer. `npx halfoffive/trad-skill` copies the skill into the target agent's skills directory (default `~/.claude/skills/tradingagents-analysis`), with `--dir` and `--agent claude|agents|opencode` overrides. Idempotent.
- SKILL.md §2 "Before You Start — Confirm the Target": the agent now **asks the user which ticker to analyze** (and optionally date / debate rounds) before spawning any sub-agent, instead of assuming a ticker was given.
- SKILL.md §4: the main agent now resolves the absolute path to the skill's `scripts/` directory and embeds it in each sub-agent spawn, with a spawn template that requires running the script before writing the report.

### Fixed
- **Sub-agents never ran the bundled scripts** — SKILL.md told them to run `scripts/{script}.py` (a relative path), but a sub-agent's working directory is the user's project, not the skill folder, so the scripts never resolved. Now invoked by absolute path.
- Stale risk-debate prompt filenames in SKILL.md: `aggressive_analyst.md` / `conservative_analyst.md` / `neutral_analyst.md` → `aggressive_risk.md` / `conservative_risk.md` / `neutral_risk.md` (matching the actual files).
- `fetch_sentiment.py` ignored its `--limit` flag (parsed but never passed through). Now wired into `fetch_stocktwits`.
- `fetch_fundamentals.py` and `fetch_sentiment.py` imported `akshare` unconditionally, crashing on machines without it. Now wrapped in `try/except ImportError → ak = None` with graceful degradation, matching the other scripts.
- SKILL.md / README overclaims corrected: `fetch_stock_data.py` returns raw OHLCV (the Market Analyst computes indicators per `references/indicators.md`); `fetch_news.py` covers company + macro news (FRED / prediction-market claims removed since unimplemented).
- **`npx skill add …` never worked** — the third-party `skill` CLI (vercel-labs/codebuddy) has no `add` subcommand and no `owner/repo/subdir` support. Replaced by the custom `npx halfoffive/trad-skill` installer.

### Changed
- Scripts are now the **primary** data source; web search / browser tools are a fallback only for parts a script could not provide — no longer an easy excuse to skip the scripts.
- Default install target is `~/.claude/skills` (Claude Code); manual-install examples updated accordingly.
- SKILL.md sections renumbered (added §2, shifted the rest) with cross-references updated.

## [1.0.0] - 2026-07-22

### Added
- SKILL.md: Multi-agent trading analysis pipeline with sub-agent orchestration
- 14 verbatim agent role prompts in `references/prompts/` (from TradingAgents + TradingAgents-CN)
- Data source catalog (`references/data-sources.md`): 12 sources covering US, A-share, and HK markets
- Technical indicator reference (`references/indicators.md`): 13 indicators across 5 categories
- Python data-fetching scripts (functional style, Chinese comments):
  - `fetch_stock_data.py` — OHLCV via yfinance + akshare
  - `fetch_news.py` — news via yfinance + Google News RSS + akshare
  - `fetch_fundamentals.py` — financials via yfinance + akshare
  - `fetch_sentiment.py` — sentiment via StockTwits + Reddit + akshare
- Bilingual documentation: README.md (English) + README_CN.md (中文)
- AGENTS.md for AI agent onboarding
- Language switch buttons in both READMEs
- `npx skill add` one-click install support

### Design Decisions
- Skill files live in `tradingagents-analysis/` subfolder (clean repo root for meta files)
- Prompts are verbatim extracts — never paraphrased from memory
- Scripts return formatted strings (for LLM prompt injection), never raise exceptions
- No class-based Python; functional style throughout
