# Changelog

All notable changes to this project will be documented in this file.

## [1.3.1] - 2026-07-25

### Fixed (round 1, commit 7c958ec — previously undocumented)

- **`install.mjs` OpenCode 安装路径不匹配**：`--agent opencode` 实际安装到 `~/.config/opencode/skills`，但旧代码装到 `~/.opencode/skills`。已对齐 SKILL.md 文档。
- **`fetch_sentiment.py` 美股情绪报告裸露 `<unavailable>` 占位符**：当 StockTwits/Reddit 不可用时直接拼接裸字符串。已改为类似 `fetch_news.py` 的友好错误提示块（`> 数据源不可用`）。
- **`fetch_fundamentals.py` `_fmt_num()` 返回类型不一致**：类型注解为 `str`，但对数值返回 `float`。已统一返回 `str`。
- **`fetch_stock_data.py` docstring 拼写错误**：`_normalize_ohlcv` 中 "olumnolume" → "Volume"。
- **A股函数缺少显式 `ak is not None` 检查**：`fetch_cn_fundamentals` 和 `fetch_cn_sentiment` 在调用 akshare API 前未显式检查，与其他脚本模式不一致。已补齐。

### Fixed (round 2, this release)

- **`fetch_fundamentals.py` `_yoy()` 计算方向反了**：yfinance `financials` 列为降序（最近年在前），但代码用 `iloc[-2]`/`iloc[-1]` 取最旧两年。改为 `iloc[0]`/`iloc[1]` 取最近一年同比。同步修 docstring。
- **A 股基本面/情绪章节静默失败**：`df.to_markdown()` 惰性依赖未声明的 `tabulate` 包，未安装时 A 股整套表格输出变为 `> 获取失败: Import tabulate failed.`。`fetch_fundamentals.py` 和 `fetch_sentiment.py` 中所有 `to_markdown(index=False)` 改为 `to_string(index=False)`（纯 pandas，无新依赖）。
- **`fetch_stock_data.py` `build_compact_report` 失败路径重复网络请求**：失败时为拿错误字符串再次调用 `fetch_stock_data`。重构为单次调用 + 本地 CSV 解析。
- **4 个脚本冗余 `import sys`**：全程未引用，已删除。
- **`install.mjs` 把 `__pycache__/*.pyc` 复制到用户目录**：`fs.cpSync` 加 `filter` 排除 `__pycache__`。
- **`install.mjs` `--dir`/`--agent` 缺值时静默走默认路径**：缺值或值以 `--` 开头时 `fail()` 并打印友好错误。支持 `--dir=PATH`/`--agent=NAME` 等号语法。未知参数 `fail()`。
- **`install.mjs` `--dir ~/foo` 不展开 `~`**：在 PowerShell/cmd 中 `~` 不会自动展开，会在当前目录创建名为 `~` 的垃圾目录。已用 `os.homedir()` 展开 `~` 和 `~/`。
- **`install.mjs` `cpSync`/`rmSync` 无错误处理**：失败时抛原始 Node 堆栈。已包 `try/catch` 走 `fail()`。
- **SKILL.md spawn 模板硬编码脚本名和 `--start/--end`**：导致 News/Fundamentals/Sentiment 分析师脚本报 `unrecognized arguments`。模板改为 `{script_name}`/`{script_args}` 占位符，注释明确按 §6 替换脚本名**和**参数。
- **SKILL.md 未定义 `--start`/`--end` 默认窗口**：代理不知道取多长历史窗口。§6 加指引：默认 trade date 前 1 年到当天（至少 200 个交易日才能算 SMA200）。
- **CN 专用 prompt 文件存在但 SKILL.md 完全没引用**：`china_market_analyst.md` 和 `cn_news_analyst.md` 是 A股/港股 专用 prompt，但 SKILL.md 没说何时切换。§4 加 CN market prompt swap 说明。
- **A股自动检测规则文档与脚本不符**：README/SKILL.md 写 `.SS`/`.SZ` 后缀 → A股，但脚本只识别 6 位纯数字。文档统一改为「6 位纯数字（如 600519、000858）→ A股」。
- **SKILL.md §3 语法错误**：`Stages 1 uses parallel sub-agents` → `Stage 1 uses parallel sub-agents`。
- **SKILL.md §6 表格过度声称**：`fetch_news.py` 描述去掉 "macro"，`fetch_sentiment.py` 描述去掉 "headline analysis"（脚本不做这些）。
- **README OpenCode 安装路径错误**：`--agent opencode` 注释从 `~/.opencode/skills` 改为 `~/.config/opencode/skills`，与 install.mjs 一致。
- **README_CN A股数据源优先级错误**：`Tushare → AKShare → Baostock` → `AKShare → yfinance`（脚本实际行为）。
- **README Project Structure tree 缺 `skills/` 目录且有 `└──` 重复**：补 `skills/` 目录行，修掉连续 `└──`。
- **`references/data-sources.md` 降级链与脚本不符**：A股 `MongoDB cache → Tushare → AKShare → Baostock → TDX` → `AKShare → yfinance`；美股 `yfinance → Alpha Vantage` → `yfinance`；美股新闻 `Yahoo Finance News → Alpha Vantage News → Google News` → `yfinance + Google News`。Configuration 章节注明属原始框架，本 skill 不读取。未实现的源（Alpha Vantage/FRED/Polymarket/Tushare/Baostock/TDX）标注 "not wired in scripts"。
- **`references/prompts/README.md` 含开发机绝对路径**：`D:\niaod\RustroverProjects\trad\...` 改为 GitHub 仓库链接。
- **`references/prompts/README.md` 漏列 MFI 指标**：Market Analyst 指标列表补 MFI。
- **`references/prompts/README.md` "with Tushare data" 误导**：改为 `with akshare data (Tushare referenced in prompt but not wired in scripts)`。
- **`references/prompts/README.md` 5 阶段 vs 6 阶段不一致**：加注 `Decision = Research Manager + Trader; see SKILL.md §3 for the full 6-stage flow`。
- **第一轮修复未入 CHANGELOG，version 未 bump**：本次 1.3.1 补齐第一轮和第二轮所有 Fixed 条目。

### Added

- `.npmignore`：排除 `__pycache__/`、`*.pyc`、`*.pyo`、`.omo/`、`.codegraph/`、`.trae/`、`*.log`、`.vscode/`，作为 `package.json` `files` allowlist 的兜底。
- `.gitattributes`：`* text=auto eol=lf`，统一行尾符，防止两份 `tradingagents-analysis/` 拷贝之间 CRLF/LF 漂移。
- `AGENTS.md` Gotchas：补充 `.trae/specs/` 是 spec 工作流状态（已跟踪，与 `.omo/` 不同）的说明。
- `.gitignore`：补 `.codegraph/`（防御性，不依赖嵌套 .gitignore）和 `node_modules/`（future-proof）。
- SKILL.md §6：`fetch_stock_data.py` 行后加 `--start`/`--end` 默认窗口指引。
- SKILL.md §4：CN market prompt swap 说明（A股/港股时用 `china_market_analyst.md` / `cn_news_analyst.md`）。

### Changed

- `package.json` `version`: `1.3.0` → `1.3.1`。

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
