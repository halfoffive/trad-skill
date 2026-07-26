# Round 5 — Bug Fixes Spec

**Branch**: `fix/round5-bugs` (stacked on `fix/round4-bugs`)
**Date**: 2026-07-26
**Review method**: 4 parallel sub-agents (Python scripts / docs / verbatim prompts / install.mjs) + manual verification of every finding against the clean committed state and source repos.

## Summary

14 confirmed bugs: **0 CRITICAL, 2 HIGH, 4 MEDIUM, 8 LOW**.

> **Note on R5-1 (skills/ stale) — removed as false positive.** The initial Python-script and install.mjs sub-agents reported `skills/tradingagents-analysis/` as stale vs root `tradingagents-analysis/`. Manual verification showed this was a **working-tree artifact**: uncommitted changes from a previous interrupted session had reverted the skills/ copy on disk. After `git checkout -- .` (discarding the dirty working tree), `git diff --no-index --stat tradingagents-analysis skills/tradingagents-analysis` shows ONLY `__pycache__/*.pyc` differences (binary, .gitignored) — all source files are identical in the committed state. The real on-disk issue is `__pycache__/*.pyc` being included by `npm pack` (R5-3 below).

## Confirmed BUGs

### BUG R5-1 [HIGH] — 9 non-CN prompts missing `{get_language_instruction()}` (verbatim violation + functional)

- **Files**: 9 prompt files in `tradingagents-analysis/references/prompts/` (+ skills copy):
  `market_analyst.md`, `news_analyst.md`, `fundamentals_analyst.md`, `bull_researcher.md`, `bear_researcher.md`, `research_manager.md`, `aggressive_risk.md`, `conservative_risk.md`, `neutral_risk.md`
- **Problem**: Source repo `TradingAgents/tradingagents/agents/**/*.py` has `+ get_language_instruction()` at end of all 12 non-CN prompt strings. trad-skill converted 3 of them (`sentiment_analyst.md`, `trader.md`, `portfolio_manager.md`) to `{get_language_instruction()}` template var, but stripped it from the other 9. CN source (`TradingAgents-CN`) does NOT have `get_language_instruction`, so CN prompts correctly omit it.
- **Impact**: `output_language` config only affects 3/12 non-CN sub-agents. 9 sub-agents (Market/News/Fundamentals Analysts, Bull/Bear Researchers, Research Manager, 3 Risk Analysts) never receive the "Respond in <language>" instruction. Verbatim violation per AGENTS.md "Do NOT paraphrase".
- **Verification**: Confirmed via source repo grep — all 12 non-CN source .py files have `get_language_instruction`; only 3 trad-skill .md files have `{get_language_instruction()}` (verified in clean committed state).
- **Fix**: Add `{get_language_instruction()}` at end of prompt body (before closing ``` fence) in each of the 9 files, matching the placement in `sentiment_analyst.md:64` / `portfolio_manager.md:35`. Update each file's "Template variables" front-matter line to include `{get_language_instruction()}`.

### BUG R5-2 [HIGH] — `fetch_news(None)` / `fetch_stock_data(None,...)` / `fetch_sentiment(None)` raise AttributeError

- **Files**: `tradingagents-analysis/scripts/fetch_news.py:231`, `fetch_stock_data.py:402`, `fetch_sentiment.py:269` (+ skills copy)
- **Problem**: Round 4 fixed `fetch_fundamentals(None)` (added `isinstance(symbol, str)` guard), but missed the other 3 public functions. Each does `symbol.strip()` or `symbol.isdigit()` at entry, raising `AttributeError: 'NoneType' object has no attribute 'strip'` / `'isdigit'` for None input. Violates AGENTS.md "never raises" contract.
- **Verification**: `uv run --with yfinance,pandas,akshare,requests python -c "..."` confirmed all 3 raise in clean committed state.
- **Fix**: Add `if not isinstance(symbol, str): return "错误: 无效的股票代码 ..."` + `symbol = symbol.strip()` + empty-string guard at each entry, mirroring `fetch_fundamentals` round-4 fix.

### BUG R5-3 [MEDIUM] — `npm pack` includes `__pycache__/*.pyc` (~58.6kB garbage)

- **Files**: `package.json`, source `tradingagents-analysis/scripts/__pycache__/*.pyc` (4 files on disk, .gitignored but present)
- **Problem**: `npm pack --dry-run` includes 4 `.pyc` files (~58.6kB). `package.json` `files` allowlist includes `tradingagents-analysis/` directory; `.npmignore` cannot exclude files inside a `files`-allowlisted directory. The on-disk `__pycache__/` exists because `uv run` / verification scripts compiled the .py files.
- **Fix**: Add `prepublishOnly` npm script that removes `__pycache__/` from both source trees before publish.

### BUG R5-4 [MEDIUM] — `install.mjs:130` prints relative script paths when `--dir` is relative

- **Files**: `install.mjs:130`
- **Problem**: `path.join(scriptsDir, s)` preserves relative-ness of `--dir`. If user runs `node install.mjs --dir ./my-skills`, printed script paths are `my-skills/tradingagents-analysis/scripts/...` (relative). Sub-agent CWD ≠ install CWD, so relative paths break. AGENTS.md: "Script paths must be absolute."
- **Fix**: Change `path.join(scriptsDir, s)` → `path.resolve(scriptsDir, s)` (or `path.resolve(parentDir)` early). `path.resolve` always returns absolute.

### BUG R5-5 [MEDIUM] — `README_CN.md:129` residual `.SS`/`.SZ` description

- **Files**: `README_CN.md:129`
- **Problem**: L128 says "6位纯数字格式（如 600519、000858）" but L129 still says "自动识别上海/深圳市场（`.SS` 后缀为上海，`.SZ` 后缀为深圳）". Contradicts the 6-digit rule and the script behavior (scripts only accept 6-digit, internally append `.SS`/`.SZ` for yfinance fallback). Round 2 fixed README.md and SKILL.md but missed this README_CN line.
- **Fix**: Replace L129 with "脚本内部根据 6 位代码前缀自动判断交易所（6 开头 → 上海 .SS；0/3 开头 → 深圳 .SZ），用户只需提供 6 位纯数字".

### BUG R5-6 [MEDIUM] — `install.mjs:104` `mkdirSync` outside try/catch

- **Files**: `install.mjs:104`
- **Problem**: `fs.mkdirSync(parentDir, { recursive: true })` is outside the try/catch (L107-117). Invalid path chars (e.g. `--dir C:\bad:path`) print raw Node stack instead of friendly `fail()` message.
- **Fix**: Move L104 into the existing try/catch block.

### BUG R5-7 [LOW] — `CHANGELOG.md:15` says "9 ghost tools" but actual is 11

- **Files**: `CHANGELOG.md:15`
- **Problem**: Round-4 CHANGELOG entry says "扩展章节覆盖全部 9 个 ghost tools". Actual count: Market Analyst (3) + News Analyst (4) + Fundamentals (4) = 11.
- **Fix**: `9` → `11`.

### BUG R5-8 [LOW] — `README.md:248` / `README_CN.md:256` broken relative link

- **Files**: `README.md:248`, `README_CN.md:256`
- **Problem**: `[references/data-sources.md](references/data-sources.md)` — from repo root, `references/data-sources.md` doesn't exist (it's at `tradingagents-analysis/references/data-sources.md`). GitHub renders 404.
- **Fix**: Change to `(tradingagents-analysis/references/data-sources.md)`.

### BUG R5-9 [LOW] — `prompts/README.md:104` `{investment_plan}` flow description wrong

- **Files**: `tradingagents-analysis/references/prompts/README.md:104` (+ skills copy)
- **Problem**: Says "Research Manager output → Trader input + Risk Debate input." But `{investment_plan}` only flows to Trader (`trader.md:18`). Risk Debate uses `{trader_decision}` (Trader output), not `{investment_plan}`. SKILL.md §3 confirms: Stage 3 (Research Manager) → Stage 4 (Trader) → Stage 5 (Risk Debate).
- **Fix**: Remove "+ Risk Debate input".

### BUG R5-10 [LOW] — `prompts/README.md:125,131` CN prompt misattribution

- **Files**: `tradingagents-analysis/references/prompts/README.md:125,131` (+ skills copy)
- **Problem**: Headers "Market Analyst tools (`market_analyst.md` / `china_market_analyst.md`)" and "News Analyst tools (`news_analyst.md` / `cn_news_analyst.md`)" imply CN prompts reference the same ghost tools. They don't — `china_market_analyst.md` references "Tushare数据接口", `cn_news_analyst.md` references no ghost tools. Source CN .py uses different toolkit names.
- **Fix**: Add a note under each header: "CN counterpart does not reference these ghost tools; the same `fetch_stock_data.py` / `fetch_news.py` script override still applies at spawn time."

### BUG R5-11 [LOW] — `_fmt_num(pd.NA)` returns `'<NA>'` not `'N/A'`

- **Files**: `tradingagents-analysis/scripts/fetch_fundamentals.py:30-37` (+ skills copy)
- **Problem**: Guard `isinstance(v, float) and pd.isna(v)` doesn't cover `pd.NA` (not a float subclass). `pd.NA` falls through to `round(float(pd.NA))` → TypeError → except returns `str(v)` = `'<NA>'`. Third NA sentinel alongside `'N/A'` and `'nan'`. Inconsistent with `compute_indicators._val` (uses `pd.notna(v)` uniformly).
- **Verification**: `_fmt_num(pd.NA)` → `'<NA>'`; `_fmt_num(None)` → `'N/A'`; `_fmt_num(np.nan)` → `'N/A'` (confirmed in clean state).
- **Fix**: Change guard to `if v is None or pd.isna(v): return "N/A"` (pd.isna covers None/np.nan/pd.NA). Drop `isinstance(v, float)` restriction.

### BUG R5-12 [LOW] — `fetch_stock_df(None,...)` raises AttributeError

- **Files**: `tradingagents-analysis/scripts/fetch_stock_data.py:371` (+ skills copy)
- **Problem**: Internal helper `fetch_stock_df` does `symbol.strip()` at entry, raises for None. Docstring says "内部用" but it's a public-named function (no `_` prefix).
- **Verification**: Confirmed raises in clean state.
- **Fix**: Add same None guard as R5-2.

### BUG R5-13 [LOW] — `.npmignore` missing `node_modules/` and `CLAUDE.md`

- **Files**: `.npmignore`
- **Problem**: `package.json` `files` allowlist already excludes these, but `.npmignore` is documented as "兜底" (fallback). Missing `node_modules/` and `CLAUDE.md` lines.
- **Fix**: Add `node_modules/` and `CLAUDE.md` to `.npmignore`.

### BUG R5-14 [LOW] — `install.mjs` `--dir` + `--agent` both given → `--agent` silently ignored

- **Files**: `install.mjs:87-99`
- **Problem**: `if (args.dir) ... else if (args.agent) ...` — when both given, `--agent` is silently ignored, install goes to `--dir` path. User may expect `--agent` subpath.
- **Fix**: `fail('不能同时指定 --dir 和 --agent')` when both given.

## Non-bugs (verified clean)

- `skills/` vs root sync: source files identical in committed state (only `__pycache__/*.pyc` differs on disk, gitignored).
- All 4 scripts: no `class` keyword, no tabulate residue, deps within whitelist, akshare soft-dep guarded, Chinese comments, argparse CLI present.
- `compute_indicators._val` NaN handling: uses `pd.notna`, outputs "N/A" for SMA200/RSI/MFI on short DataFrames — clean.
- Template variable count: 30 unique — matches README claim.
- Ghost tools count in `prompts/README.md`: 11 listed (only CHANGELOG says 9).
- Verbatim CN prompts: correctly omit `get_language_instruction` (source CN doesn't have it).
- Stage mapping in prompts/README Prompt Index: consistent with SKILL.md §3.
- `install.mjs` idempotency, `~` expansion, `--dir=PATH` syntax, unknown-arg fail: all pass.
- `.gitignore` / `.gitattributes`: correct.

## Out of scope

- PR #3 (fix/round2-bugs) is orphaned (not in our stack). Leave alone.
- PR #4 (fix/round3-bugs → main) and PR #5 (fix/round4-bugs → fix/round3-bugs) remain open; round-5 PR stacks on fix/round4-bugs.
