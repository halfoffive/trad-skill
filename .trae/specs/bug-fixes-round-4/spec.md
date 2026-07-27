# Round 4 — Bug-Fix Spec

**Status**: in-progress
**Branch**: `fix/round4-bugs` (stacked on `fix/round3-bugs`)
**Date**: 2026-07-25
**Review method**: 3 parallel sub-agents (regression audit, residual scan, edge-case/contract audit with runtime evidence)

## Context

Round 3 修复了 9 个 bug（见 `../bug-fixes-round-3/spec.md`），41/41 检查点通过。Round 4 通过 3 个并行子代理对当前状态做回归 + 残留 + 边缘 case 审计，发现 7 个确认 bug（1 CRITICAL / 1 HIGH / 3 MEDIUM / 2 LOW），其中 2 个是 Round 3 修复不完整（BUG 7、BUG 9），5 个是新发现。

## Confirmed Bugs (7)

### BUG R4-1 [CRITICAL] — `fetch_fundamentals("")` / `fetch_fundamentals(None)` 抛异常

**证据**（运行时）:
```
>>> fetch_fundamentals("")
ValueError: Empty ticker name
  File ".../fetch_fundamentals.py", line 143, in fetch_us_fundamentals
    ticker = yf.Ticker(symbol)         # ← 在 try/except 之外
  File ".../yfinance/base.py", line 98, in __init__
    raise ValueError("Empty ticker name")

>>> fetch_fundamentals(None)
AttributeError: 'NoneType' object has no attribute 'isdigit'
  File ".../fetch_fundamentals.py", line 299, in fetch_fundamentals
    if symbol.isdigit() and len(symbol) == 6:
```

**根因**:
- `fetch_us_fundamentals` L143 `ticker = yf.Ticker(symbol)` 在 try/except **之外**。yfinance 对空串抛 `ValueError`，直接穿透到调用方。对比 `fetch_us_stock_data` L244 同样调用 `yf.Ticker(symbol)` 但包在 `try/except Exception` 里。
- `fetch_fundamentals` L299 `symbol.isdigit()` 对 None 抛 `AttributeError`。对比 `fetch_stock_data` L394 `symbol = symbol.strip()` 会先对 None 抛错，但 `fetch_stock_data` 的调用方都经 CLI 不会传 None；`fetch_fundamentals` 同样如此，但契约要求"never raises"对任意输入成立。

**影响**: 违反 AGENTS.md 契约"Every function returns a formatted string, never raises"。虽然 CLI 入口 `args.symbol` 是 str 且 `required=True`，但函数是 public API（其他代码可能 `import` 调用），且 `fetch_stock_data` 已经对空串做了优雅降级，`fetch_fundamentals` 应保持一致。

**修复目标**: `fetch_fundamentals` 入口对空串 / None / 非字符串返回错误字符串；`fetch_us_fundamentals` 的 `yf.Ticker(symbol)` 移入 try/except。

**修复内容**:
1. `fetch_fundamentals(symbol)` 入口加：
   ```python
   if not isinstance(symbol, str):
       return f"错误: 无效的股票代码 {symbol!r}"
   symbol = symbol.strip()
   if not symbol:
       return "错误: 股票代码不能为空"
   ```
2. `fetch_us_fundamentals(symbol)` 入口加 `symbol = symbol.strip()`（防御性），并把 L143 `ticker = yf.Ticker(symbol)` 移入紧随其后的 try/except（与 `fetch_us_stock_data` 一致）：
   ```python
   try:
       ticker = yf.Ticker(symbol)
       info = ticker.info
       sections.append("## 公司概况\n")
       ...
   except Exception as e:
       sections.append(f"## 公司概况\n\n> 获取失败: {e}\n")
   ```
   注意：当前 L146-165 的 try/except 已经覆盖 `ticker.info`，只需把 `ticker = yf.Ticker(symbol)` 一行移进去即可。

**影响范围**: `tradingagents-analysis/scripts/fetch_fundamentals.py` + `skills/tradingagents-analysis/scripts/fetch_fundamentals.py`（双拷贝同步）。

---

### BUG R4-2 [MEDIUM] — `compute_stats` 输出 "年化波动率: nan%" / "区间收益率: nan%"

**证据**（运行时，单行 DataFrame）:
```
## 区间统计
- 区间收益率: 0.0%
- 年化波动率: nan%        # ← 误导：看起来像数字，实际是 NaN
- 日均成交量: 1000
- 52周(或区间)高/低: 100.0 / 100.0
```

**根因**: `fetch_stock_data.py` L207-210:
```python
ret = (last / first - 1) * 100 if first else float("nan")
daily_ret = close.pct_change().dropna()
vol = daily_ret.std() * (252 ** 0.5) * 100 if len(daily_ret) > 1 else float("nan")
```
当 `first=0`（或 NaN）→ `ret=nan`；当 `len(daily_ret) <= 1`（单行）→ `vol=nan`。L218-219 直接 `round(float(vol), 2)` 输出 `"nan%"`。对比 L220 `avg_vol` 已用 `pd.notna(avg_vol)` 守卫输出 "N/A"。

**影响**: LLM 看到 `"nan%"` 可能误判为数值（"nan" 看起来像变量名或缩写），不像 "N/A" 那样明确表示缺失。Market Analyst 据 §3 表格"interprets pre-computed values"，误读会污染下游分析。

**修复目标**: `ret` / `vol` / `hi` / `lo` 为 NaN 时输出 "N/A"，与 `avg_vol` 保持一致。

**修复内容**: L216-222 改为：
```python
def _num(v, nd=2):
    """格式化数值，NaN → 'N/A'。"""
    try:
        return round(float(v), nd) if pd.notna(v) else "N/A"
    except (TypeError, ValueError):
        return "N/A"

lines = [
    "## 区间统计\n",
    f"- 区间收益率: {_num(ret)}%",
    f"- 年化波动率: {_num(vol)}%",
    f"- 日均成交量: {int(avg_vol) if pd.notna(avg_vol) else 'N/A'}",
    f"- 52周(或区间)高/低: {_num(hi, 4)} / {_num(lo, 4)}",
]
```

**影响范围**: `tradingagents-analysis/scripts/fetch_stock_data.py` + `skills/` 双拷贝。

---

### BUG R4-3 [MEDIUM] — README.md / README_CN.md 示例日期仍是 `2024-01-01`（Round 3 BUG 7 未同步）

**证据**:
- `README.md:260`: `python scripts/fetch_stock_data.py --symbol AAPL --start 2024-01-01 --end 2024-06-30 --tail 30 --stats`（6 个月）
- `README.md:266`: `python scripts/fetch_stock_data.py --symbol AAPL --start 2024-01-01 --end 2024-01-31 --raw`（1 个月）
- `README_CN.md:268,271,277`: 同样 `2024-01-01 --end 2024-06-30`
- `README_CN.md:274`: `2024-01-01 --end 2024-01-31`
- 对比 `SKILL.md:237`（Round 3 已修）: `--start 2023-07-01 --end 2024-06-30`（12 个月）
- `SKILL.md:242`: "至少需 200 个交易日才能算 SMA200"

**根因**: Round 3 BUG 7 只改了 `SKILL.md §6`，漏改两份 README。用户复制 README 示例运行，SMA200 会显示 "N/A"。

**修复目标**: 两份 README 的 `fetch_stock_data.py` 示例日期与 SKILL.md §6 一致（≥ 10 个月窗口）。

**修复内容**: 把 README.md L260/L266 和 README_CN.md L268/L271/L274/L277 中所有 `--start 2024-01-01 --end 2024-06-30` 改为 `--start 2023-07-01 --end 2024-06-30`；`--start 2024-01-01 --end 2024-01-31`（raw 示例）改为 `--start 2024-01-01 --end 2024-06-30`（保留 raw 的短窗口演示性质，但扩到 6 个月避免 SMA200 完全为空——raw 不算指标，仅作 CSV 长度演示，6 个月足够）。

**影响范围**: `README.md` + `README_CN.md`。

---

### BUG R4-4 [MEDIUM] — `sentiment_analyst.md` 期望 `{news_block}` 但脚本不抓新闻

**证据**:
- `sentiment_analyst.md:7`: 模板变量列表含 `{news_block}`, `{stocktwits_block}`, `{reddit_block}`
- `sentiment_analyst.md:19-21`: `<start_of_news>{news_block}<end_of_news>` — 期望预取的新闻数据块
- `sentiment_analyst.md:26-28`: `<start_of_stocktwits>{stocktwits_block}<end_of_stocktwits>`
- `sentiment_analyst.md:33-35`: `<start_of_reddit>{reddit_block}<end_of_reddit>`
- `SKILL.md:84`（Round 3 BUG 6 修复后）: `Sentiment Analyst | Social sentiment → composite score | StockTwits, Reddit (US) / akshare 个股评论+机构参与度 (CN) via scripts/fetch_sentiment.py` — 无新闻
- `SKILL.md:154` spawn 模板: 只让跑一个脚本 `fetch_sentiment.py`，其输出含 StockTwits+Reddit 但**无新闻**

**根因**: verbatim 提示词假设框架预取三块数据（新闻 / StockTwits / Reddit）并注入模板变量。本技能的 spawn 模型是"子代理自己跑脚本"，而 `fetch_sentiment.py` 不抓新闻 → `{news_block}` 无数据源、无替换规则。`{stocktwits_block}` / `{reddit_block}` 数据在脚本输出里但映射未文档化。

**影响**: 子代理收到字面量 `{news_block}`（空块），可能困惑或忽略整个新闻段；或按 verbatim 提示词 L49 "If the sources are silent... flag this explicitly" 把 `{news_block}` 当成"数据源不可用"，降低 confidence。

**修复目标**: 在 `prompts/README.md` 的 Template Variable Substitution 章节（见 BUG R4-5）明确映射：
- `{stocktwits_block}` / `{reddit_block}` → 由 `fetch_sentiment.py` 输出的对应段落填充（US 分支）；CN 分支由 akshare 个股评论 + 机构参与度填充（语义等价）。
- `{news_block}` → 脚本不提供；子代理若需新闻上下文，用 web 搜索（fallback），或留空并标注"新闻块未预取"。

**影响范围**: `tradingagents-analysis/references/prompts/README.md` + `skills/` 双拷贝。

---

### BUG R4-5 [HIGH] — verbatim 提示词中 30 个模板变量未文档化替换规则

**证据**（全量枚举 14 个 prompt 文件的模板变量）:

子代理收到 verbatim 提示词后，以下变量若未替换，会以字面量 `{...}` 形式留在 prompt 里：

| 类别 | 变量 | 出现位置 | SKILL.md 是否文档化 |
|---|---|---|---|
| 数据报告 | `{market_research_report}` `{sentiment_report}` `{news_report}` `{fundamentals_report}` | bull/bear/risk/portfolio | §4 "Re-injection discipline" 部分文档化（Key Signals 摘要 vs 完整报告） |
| 数据报告 | `{history}` | 所有辩论阶段 | §4 "Re-injection discipline" 已文档化 |
| 数据报告 | `{investment_plan}` `{trader_decision}` `{trader_plan}` `{research_plan}` | trader/risk/portfolio | §5 Stage 描述用 prose 说 "Data: the investment plan"，未绑定变量名 |
| 数据报告 | `{current_response}` `{current_aggressive_response}` `{current_conservative_response}` `{current_neutral_response}` | bull/bear/risk 辩论 | 未文档化 |
| 预取数据块 | `{news_block}` `{stocktwits_block}` `{reddit_block}` | sentiment_analyst | 未文档化（见 BUG R4-4） |
| 标识 | `{ticker}` `{target_label}` `{company_name}` `{asset_label}` `{fundamentals_label}` | 各分析师 / 研究员 / trader | 未文档化 |
| 日期 | `{current_date}` `{start_date}` `{end_date}` | 各分析师 / sentiment | 未文档化 |
| 上下文 | `{instrument_context}` | 几乎所有 | 未文档化 |
| 语言 | `{get_language_instruction()}` | sentiment/trader/portfolio | 未文档化 |
| 工具 | `{tool_names}` `{system_message}` | 所有分析师 | 未文档化 |
| 工具 | `{NO_EXTERNAL_TOOLS}` | research_manager/trader/portfolio | 未文档化（且与 spawn 模板"web-search fallback"冲突） |
| 其他 | `{lessons_line}` | portfolio_manager | 未文档化 |

**根因**: SKILL.md §4 spawn 模板 L152 只追加 `"Analyze {ticker} as of {date}."`，未指导主代理替换 verbatim 提示词内部的 30 个模板变量。Round 3 BUG 9 只覆盖 3 个工具名（`get_stock_data` 等），未覆盖其余 27 个变量。

**影响**:
- `{current_date}` / `{start_date}` / `{end_date}` 未替换 → 子代理不知道分析日期窗口。
- `{get_language_instruction()}` 未替换 → 子代理不知道输出语言（虽然 §8 有默认，但提示词里字面量 `{get_language_instruction()}` 会困惑子代理）。
- `{instrument_context}` 未替换 → 子代理不知道市场上下文（A股/港股/US/Crypto）。
- `{NO_EXTERNAL_TOOLS}` 未替换且与 spawn 模板冲突 → research_manager/trader/portfolio 收到字面量 `{NO_EXTERNAL_TOOLS}`，但 spawn 模板说"fall back to web search"，语义冲突。
- `{target_label}` / `{company_name}` / `{asset_label}` 未替换 → 子代理不知道标的名称。
- `{tool_names}` / `{system_message}` 未替换 → 子代理看到字面量。

**修复目标**: 在 `prompts/README.md` 新增 "Template Variable Substitution" 章节，按类别列出所有 30 个变量的替换规则；SKILL.md §4 加一行指针指向该章节。

**修复内容**: `prompts/README.md` 在现有 "Tool-Name Override" 章节后新增 "Template Variable Substitution" 章节，包含：

```markdown
## Template Variable Substitution

The verbatim role prompts contain LangChain-style template variables (`{ticker}`, `{current_date}`,
`{instrument_context}`, `{get_language_instruction()}`, etc.) that the original TradingAgents framework
substituted at runtime. This skill does not use LangChain — the main agent must substitute these
variables **before** spawning each sub-agent. The table below defines the substitution for every
variable found across `references/prompts/*.md`.

### Identity & labels
| Variable | Substitute with |
|---|---|
| `{ticker}` `{target_label}` `{asset_label}` | The ticker symbol (e.g. `AAPL`, `600519`). |
| `{company_name}` | The company name if known (from `fetch_fundamentals.py` profile); else the ticker. |
| `{fundamentals_label}` | The literal string `Fundamentals` (section label in bull/bear prompts). |

### Dates
| Variable | Substitute with |
|---|---|
| `{current_date}` | Today's date (YYYY-MM-DD). |
| `{start_date}` `{end_date}` | The analysis window start/end (default: today-365 to today; see §6). |

### Context & language
| Variable | Substitute with |
|---|---|
| `{instrument_context}` | A one-line market context: `Market: <US / A股 / 港股 / Crypto>; Ticker: <symbol>; Trade date: <date>`. |
| `{get_language_instruction()}` | `Respond in <English / 中文> per output_language.` |
| `{tool_names}` | Empty string, or `python <script>` for the assigned script. The script is the only tool. |
| `{system_message}` | Empty string (no system message; the role prompt is the system message). |
| `{NO_EXTERNAL_TOOLS}` | Empty string — **not** set. The spawn template permits web-search fallback when a script fails. |
| `{lessons_line}` | Empty string (no lessons-learned line in this skill). |

### Data reports (bound to stage outputs)
| Variable | Bound to |
|---|---|
| `{market_research_report}` `{sentiment_report}` `{news_report}` `{fundamentals_report}` | The four analyst reports. In debate stages (2 & 5), bind to each report's `## Key Signals` digest only; in Portfolio Manager, bind full reports (see SKILL.md §4 "Re-injection discipline"). |
| `{history}` | The running debate transcript. |
| `{investment_plan}` | Research Manager output → Trader + Risk Debate input. |
| `{trader_decision}` `{trader_plan}` | Trader output → Risk Debate + Portfolio Manager input. |
| `{research_plan}` | Research Manager output → Portfolio Manager input. |
| `{current_response}` | The previous bull/bear argument in the current debate round. |
| `{current_aggressive_response}` `{current_conservative_response}` `{current_neutral_response}` | The previous risk arguments in the current risk-debate round. |

### Pre-fetched data blocks (Sentiment Analyst only)
| Variable | Substitute with |
|---|---|
| `{stocktwits_block}` `{reddit_block}` | The corresponding sections of `fetch_sentiment.py` output (US branch). For CN, use the akshare 个股评论 / 机构参与度 sections. |
| `{news_block}` | **Not provided by `fetch_sentiment.py`.** The sentiment analyst should use web search for news context, or leave empty and note "news block not pre-fetched". The News Analyst separately covers news; sentiment should focus on social signals. |
```

SKILL.md §4 在 spawn 模板后加一行：
```
> **Template variables.** The verbatim role prompts contain `{ticker}`, `{current_date}`,
> `{instrument_context}`, `{get_language_instruction()}`, and ~25 other LangChain-style variables.
> Before spawning, substitute them per the table in `references/prompts/README.md`
> (§ "Template Variable Substitution"). The most common: `{ticker}`/`{target_label}`/`{asset_label}`
> → ticker; `{current_date}` → today; `{instrument_context}` → market context line;
> `{get_language_instruction()}` → `Respond in <language>`.
```

**影响范围**: `tradingagents-analysis/references/prompts/README.md` + `tradingagents-analysis/SKILL.md` + `skills/` 双拷贝。

---

### BUG R4-6 [LOW] — `--no-stats` 参数未定义（与 `--no-indicators` 不对称）

**证据**:
```
$ python fetch_stock_data.py --symbol AAPL --no-indicators --no-stats --tail 5
usage: fetch_stock_data.py [-h] --symbol SYMBOL [--start START] [--end END]
                           [--tail TAIL] [--indicators] [--no-indicators]
                           [--stats] [--raw]
fetch_stock_data.py: error: unrecognized arguments: --no-stats
```
`--indicators` (L490-495) 配对了 `--no-indicators` (L496-501)，但 `--stats` (L502-508) 只有 store_true，无配对的 `--no-stats`。

**根因**: argparse 对称性缺失。用户看到 `--no-indicators` 自然会尝试 `--no-stats`。

**修复目标**: 补 `--no-stats`（dest="stats", action="store_false"），与 `--no-indicators` 对称。

**修复内容**: `fetch_stock_data.py` L502-508 后追加：
```python
parser.add_argument(
    "--no-stats",
    dest="stats",
    action="store_false",
    help="关闭区间统计（与 --stats 对称，显式关闭）",
)
```

**影响范围**: `tradingagents-analysis/scripts/fetch_stock_data.py` + `skills/` 双拷贝。

---

### BUG R4-7 [LOW] — `prompts/README.md` Tool-Name Override 只覆盖 market_analyst（Round 3 BUG 9 未完成）

**证据**:
- `prompts/README.md` "Tool-Name Override" 章节只列 `get_stock_data` / `get_indicators` / `get_verified_market_snapshot` → `fetch_stock_data.py`
- `news_analyst.md:12` 引用 `get_news`, `get_global_news`, `get_macro_indicators`, `get_prediction_markets`（4 个，无脚本支撑）
- `fundamentals_analyst.md:12` 引用 `get_fundamentals`, `get_balance_sheet`, `get_cashflow`, `get_income_statement`（4 个，仅 `get_fundamentals` 语义映射到 `fetch_fundamentals.py`）

**根因**: Round 3 BUG 9 只扫描了 `market_analyst.md`，未扫描其他分析师 prompt。

**修复目标**: 扩展 "Tool-Name Override" 章节覆盖所有 9 个 ghost tools。

**修复内容**: 在 `prompts/README.md` "Tool-Name Override" 章节追加：
```markdown
- `get_news` / `get_global_news` → `python "<skill>/scripts/fetch_news.py" --symbol <ticker> --days 7`
- `get_fundamentals` → `python "<skill>/scripts/fetch_fundamentals.py" --symbol <ticker>`
- `get_balance_sheet` / `get_cashflow` / `get_income_statement` → 无独立脚本；这些报表已由 `fetch_fundamentals.py` 抽取为关键指标表（营收/净利润/EPS/总资产/总负债/经营现金流/自由现金流）。如需完整报表，用 web-search fallback。
- `get_macro_indicators` / `get_prediction_markets` → 无脚本；FRED / Polymarket 数据本技能未实现，用 web-search fallback。
```

**影响范围**: `tradingagents-analysis/references/prompts/README.md` + `skills/` 双拷贝。

---

## Non-Goals (本轮不修)

- **VWMA `volume.rolling(20).sum()` 未用 `.replace(0, pd.NA)`**：Agent 3 标记为"风格不一致"，但运行时验证 `0/0 → NaN`，`_val` 返回 "N/A"，无实际 bug。本轮不改（避免无收益的 churn）。
- **`fetch_stock_data` 函数入口未填充默认日期**：spec 偏差（spec 说在函数入口填充，实现只在 CLI 填充）。当前所有调用路径经 CLI，None 不会泄漏到 `start_date.replace("-", "")`。非可用路径上的真实 bug，本轮不改。
- **verbatim 提示词内容本身**：AGENTS.md 禁止改写 verbatim prompt。所有修复通过 SKILL.md / prompts/README.md 的 override 文档实现，不改 prompt 文件。

## Verification Plan

每个 bug 对应的检查点见 `checklist.md`。修复后运行 `trad-r4-verify.py`（扩展自 round-3 suite，新增 7 个 bug 的检查 + 回归检查）。

## Out of Scope

- 功能增强（新指标、新数据源、新市场）
- 性能优化（token 量已在 round 1-2 优化）
- 提示词重写（verbatim 保留）
