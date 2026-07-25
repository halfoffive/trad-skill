## Summary

Round 2 of the iterative bug-fix loop on the TradingAgents skill. Five parallel review sub-agents (Python scripts, SKILL.md, installer, docs, cross-cutting consistency) identified **12 confirmed BUGs** and **11 high-impact ISSUEs**. This PR fixes all of them in 9 surgical commits, each focused on one area.

Spec: `.trae/specs/bug-fixes-round-2/` (spec.md / tasks.md / checklist.md).

## What's fixed

### Python scripts (4 scripts × 2 copies, kept byte-identical)
- **`fetch_fundamentals.py` `_yoy()` direction reversed** — yfinance `financials` columns are descending (most recent first), but the code used `iloc[-2]`/`iloc[-1]` (oldest two years). Now uses `iloc[0]`/`iloc[1]` (most recent year vs prior year). Docstring updated.
- **A-share fundamentals/sentiment sections silently failing** — `df.to_markdown()` lazily imports `tabulate`, which is not a declared dependency. Without `tabulate`, the entire A-share table output became `> 获取失败: Import tabulate failed.`. Replaced all `to_markdown(index=False)` with `to_string(index=False)` (pure pandas, no new dep) in `fetch_fundamentals.py` and `fetch_sentiment.py`.
- **`fetch_stock_data.py` `build_compact_report` duplicate network request** — on failure, it called `fetch_stock_data` again just to get the error string. Refactored to single call + local CSV parse.
- **Removed unused `import sys`** from all 4 scripts.

### Installer (`install.mjs`) + npm packing
- **`fs.cpSync` copied `__pycache__/*.pyc` to user dir** — added `filter` to exclude `__pycache__`.
- **`--dir` / `--agent` missing value silently fell through to default path** — now `fail()` with friendly message. Supports `--dir=PATH` / `--agent=NAME` `=` syntax. Unknown args `fail()` instead of being silently ignored.
- **`--dir ~/foo` didn't expand `~`** — PowerShell/cmd don't auto-expand `~`, so a literal `~` directory was created in CWD. Now expands via `os.homedir()`.
- **`cpSync` / `rmSync` had no error handling** — wrapped in `try/catch` calling `fail()`.
- **Added `.npmignore`** as a backstop (excludes `__pycache__/`, `*.pyc`, `.omo/`, `.codegraph/`, `.trae/`, etc.).

### `SKILL.md`
- **Spawn template hardcoded `fetch_stock_data.py` + `--start/--end`** — caused News/Fundamentals/Sentiment analyst scripts to throw `unrecognized arguments`. Template now uses `{script_name}` / `{script_args}` placeholders with a clear note to substitute both per §6.
- **`--start` / `--end` default window undefined** — agents didn't know how much history to fetch. §6 now says: default `--start` = trade date − 1 year, `--end` = trade date (≥ 200 trading days for SMA200).
- **CN-specific prompts existed but were never referenced** — `china_market_analyst.md` and `cn_news_analyst.md` are A-share/HK-specific prompts covering T+1, price limits, northbound capital, etc. §4 now documents the CN market prompt swap.
- **A-share detection rule mismatch** — docs said `.SS`/`.SZ` suffix → A-share, but scripts only recognize 6-digit pure numbers. Docs unified to "6 位纯数字 (e.g., 600519, 000858) → A-share".
- **§3 grammar** — `Stages 1` → `Stage 1`.
- **§6 overclaims** — `fetch_news.py` description dropped "macro", `fetch_sentiment.py` dropped "headline analysis" (scripts don't do these).

### Docs (`README.md`, `README_CN.md`, `references/*.md`, `AGENTS.md`)
- **OpenCode install path wrong in READMEs** — `~/.opencode/skills` → `~/.config/opencode/skills` (matches `install.mjs` and `SKILL.md`).
- **`README_CN.md` A-share data source priority fiction** — `Tushare → AKShare → Baostock` → `AKShare → yfinance` (actual script behavior).
- **Project Structure tree** — added missing `skills/` directory; fixed consecutive `└──` formatting.
- **`references/data-sources.md` downgrade chains** — A-share: `MongoDB cache → Tushare → AKShare → Baostock → TDX` → `AKShare → yfinance`; US: dropped Alpha Vantage; US news: dropped Alpha Vantage News. Configuration section annotated as original-framework-only. Unimplemented sources marked "not wired in scripts".
- **`references/prompts/README.md`** — replaced dev-machine absolute paths (`D:\niaod\...`) with GitHub links; added MFI to indicator list; "Tushare data" → "akshare data"; noted `Decision = Research Manager + Trader` for the 5-stage overview.
- **`AGENTS.md`** — Gotchas now documents `.trae/specs/` as tracked spec-workflow state (distinct from `.omo/`).
- **`.gitignore`** — added `.codegraph/` and `node_modules/` defensively.

### Release hygiene
- **`.gitattributes`** — `* text=auto eol=lf` to prevent CRLF/LF drift between the two `tradingagents-analysis/` copies.
- **`package.json`** — version `1.3.0` → `1.3.1`.
- **`CHANGELOG.md`** — new `[1.3.1] - 2026-07-25` entry covering both round 1 (commit `7c958ec`, previously undocumented) and round 2 fixes.

## Verification

22 automated checks + 4 install.mjs functional tests all pass:

- 8 Python scripts pass `uv run python -c "import ast; ast.parse(...)"` (both copies)
- `git diff --no-index tradingagents-analysis skills/tradingagents-analysis` empty (byte-identical copies)
- `node --check install.mjs` OK
- `npm pack --dry-run` tarball clean of `__pycache__` / `.pyc`
- `--dir` missing value → `fail()` (exit 1)
- `--foo` unknown arg → `fail()` (exit 1)
- `--dir=$TEMP/x` → installs correctly, no `__pycache__` in target
- `--dir ~/x` → expands to `<home>/x`, no literal `~` dir in CWD
- grep checks: no `import sys`, no `to_markdown`, no `~/.opencode/skills`, no `Stages 1`, no macro overclaim, no Tushare priority fiction, no MongoDB in data-sources.md, no dev-machine paths in prompts/README.md
- presence checks: `{script_name}`/`{script_args}` in SKILL.md, CN prompts referenced, 6-digit A-share rule, AKShare→yfinance in data-sources.md, MFI listed, `.trae/specs/` in AGENTS.md, `.codegraph/` in `.gitignore`, `.gitattributes` exists, version 1.3.1, CHANGELOG `[1.3.1]` entry

## Out of scope (deferred to next round)

- `days` parameter filtering logic in `fetch_yfinance_news` / `fetch_cn_news` / `fetch_reddit_sentiment` (behavior change, needs separate discussion)
- Bilingual drift (TDX row, A-share special notes only in CN README)
- Alpha Vantage / FRED / Tushare data source wiring (would be a feature, not a fix)
- The 14 verbatim prompt files in `references/prompts/` are untouched (per AGENTS.md policy)

## Loop status

This is round 2 of the iterative review→fix→verify loop. Round 1 spec: `.trae/specs/bug-fixes/` (merged in commit `7c958ec`). After this PR merges, round 3 will re-review for any remaining or newly introduced issues.
