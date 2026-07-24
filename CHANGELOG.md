# Changelog

All notable changes to this project will be documented in this file.

## [1.3.0] - 2026-07-24

### Added
- Standard installation via [vercel-labs/skills](https://github.com/vercel-labs/skills) CLI: `npx skills add halfoffive/trad-skill` now works across 70+ coding agents (Claude Code, Cursor, Windsurf, Trae, OpenCode, Codex, etc.).
- `skills/tradingagents-analysis/` directory: the standard location for skills CLI discovery. Contains a full copy of the skill (identical to the root copy).
- "For AI Agents: Installation Guide" section at the top of SKILL.md with explicit install commands, path locations, and dependency notes.
- "For AI Agents" subsection in both READMEs with agent-specific install instructions.
- Manual installation instructions using raw.githubusercontent.com URLs (curl-based copy-paste commands).

### Changed
- READMEs (English + Chinese): Installation section restructured — `npx skills add` is now the recommended method, custom `npx halfoffive/trad-skill` installer retained as fallback.
- `package.json`: `files` array now includes `skills/` directory for proper npm/npx packing.
- AGENTS.md: updated structure diagram and install command docs to reflect dual-location layout.

### Notes
- Backward compatible: the existing `npx halfoffive/trad-skill` custom installer continues to work unchanged.
- Root `tradingagents-analysis/` is preserved as an identical copy for the custom installer; both locations are maintained.
- Python scripts, prompts, and core pipeline logic are unchanged.

## [1.2.0] - 2026-07-23

### Changed — Deep token-cost reduction (let the LLM use python scripts)

- **Indicator computation moved into the script.** `fetch_stock_data.py` now pre-computes SMA(50/200), EMA(10), MACD/signal/hist, RSI(14), Bollinger(20,2), ATR(14), VWMA(20), MFI(14) with pure pandas (no new deps) and prints a compact indicator snapshot (latest values + trend signals: golden/death cross, overbought/oversold, band position). The Market Analyst now **interprets** pre-computed values instead of doing arithmetic over a 250-row CSV. Resolves the SKILL.md §6 "Market Analyst computes indicators" cost center.
- **Data output compacted across all scripts** to shrink the payloads that enter analyst reports and then get re-injected downstream:
  - `fetch_stock_data.py`: default output is OHLCV **tail** (default `--tail 30`, was full range) + indicators + optional `--stats`; `--raw` preserves the legacy full-range CSV.
  - `fetch_fundamentals.py`: curated **compact key-metrics table** (revenue, net income, EPS, FCF, debt, equity, OCF, margins + YoY) replaces `to_markdown()` dumps of all 4-year × 3-statement line items.
  - `fetch_news.py`: default `--limit 8` per source (was 20×2=40); all summaries truncated to 200 chars (was US-only); per-item format slimmed to title + source + one-line summary.
  - `fetch_sentiment.py`: default `--limit 15` (was 30); displayed messages 15→8; Reddit posts 20→8.
- **SKILL.md re-injection discipline (biggest lever).** Analyst reports must be concise (≤~400 words) and lead with a `## Key Signals` digest (5–8 bullets). The Stage 2 & 5 debate prompts now bind the four analyst reports to their **Key Signals digests** instead of full bodies (the verbatim prompt bodies are unchanged — only what is bound to the `{*_report}` variables changes). The Stage 6 Portfolio Manager still receives full reports + transcript once.
- `SKILL.md §4` spawn template now tells sub-agents the python script IS the data source / "verified snapshot" and not to attempt nonexistent tool names (`get_stock_data` / `get_indicators` / `get_verified_market_snapshot`), cutting wasted tool-call round-trips.
- `SKILL.md §7` final reasoning capped to 3–4 concise paragraphs (cite, don't re-narrate).
- `references/indicators.md`: notes that indicators are pre-computed by the script; the "Verified Market Snapshot" section now points at the script output instead of the nonexistent `get_verified_market_snapshot` tool.

### Added
- `fetch_stock_data.py`: `--tail`, `--indicators` / `--no-indicators`, `--stats`, `--raw` flags; `compute_indicators()` and `compute_stats()` helpers; `build_compact_report()` default entry; `_normalize_ohlcv()` shared normalizer.
- `fetch_fundamentals.py`: `_build_us_metric_table()` curated key-metrics extractor with YoY.
- `fetch_news.py`: `_truncate()` helper; `--limit` flag (default 8); slim `_format_news_item()`.

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
