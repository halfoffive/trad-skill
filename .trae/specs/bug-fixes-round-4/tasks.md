# Round 4 — Tasks

**Spec**: `spec.md`
**Branch**: `fix/round4-bugs` (stacked on `fix/round3-bugs`)

## Tasks

### Task 1: 修复 `fetch_fundamentals` 空串/None 抛异常 [BUG R4-1, CRITICAL]
- **Priority**: high
- **Depends on**: —
- **Files**: `tradingagents-analysis/scripts/fetch_fundamentals.py` (+ skills copy)
- **Description**:
  1. `fetch_fundamentals(symbol)` 入口加非字符串/空串守卫，返回错误字符串。
  2. `fetch_us_fundamentals(symbol)` 入口加 `symbol = symbol.strip()`，把 L143 `ticker = yf.Ticker(symbol)` 移入紧随其后的 try/except。
- **Test**: `fetch_fundamentals("")` 返回错误串不抛异常；`fetch_fundamentals(None)` 返回错误串不抛异常；`fetch_fundamentals("AAPL")` 正常工作。

### Task 2: 修复 `compute_stats` nan% 输出 [BUG R4-2, MEDIUM]
- **Priority**: high
- **Depends on**: —
- **Files**: `tradingagents-analysis/scripts/fetch_stock_data.py` (+ skills copy)
- **Description**: `compute_stats` 的 `ret`/`vol`/`hi`/`lo` 为 NaN 时输出 "N/A"，与 `avg_vol` 一致。引入内部 `_num(v, nd)` helper。
- **Test**: 单行 DataFrame 调用 `compute_stats` 输出含 "N/A" 不含 "nan%"；正常 DataFrame 输出数值。

### Task 3: 同步 README 示例日期 [BUG R4-3, MEDIUM]
- **Priority**: medium
- **Depends on**: —
- **Files**: `README.md`, `README_CN.md`
- **Description**: 把 `--start 2024-01-01 --end 2024-06-30` 改为 `--start 2023-07-01 --end 2024-06-30`；`--start 2024-01-01 --end 2024-01-31`（raw）改为 `--start 2024-01-01 --end 2024-06-30`。
- **Test**: README/README_CN 无 `2024-01-01 --end 2024-06-30` 之外的短窗口；grep `2024-01-01 --end 2024-01-31` 无结果。

### Task 4: 文档化 sentiment 数据块映射 [BUG R4-4, MEDIUM]
- **Priority**: medium
- **Depends on**: Task 5（同章节）
- **Files**: `tradingagents-analysis/references/prompts/README.md` (+ skills copy)
- **Description**: 在 "Template Variable Substitution" 章节的 "Pre-fetched data blocks" 子节明确 `{stocktwits_block}`/`{reddit_block}`/`{news_block}` 的映射。
- **Test**: prompts/README.md 含 `{news_block}` 映射说明。

### Task 5: 文档化 30 个模板变量替换规则 [BUG R4-5, HIGH]
- **Priority**: high
- **Depends on**: —
- **Files**: `tradingagents-analysis/references/prompts/README.md` (+ skills copy), `tradingagents-analysis/SKILL.md` (+ skills copy)
- **Description**:
  1. prompts/README.md 新增 "Template Variable Substitution" 章节（5 个子表：Identity / Dates / Context & language / Data reports / Pre-fetched blocks）。
  2. SKILL.md §4 spawn 模板后加一行指针指向 prompts/README.md。
- **Test**: prompts/README.md 含全部 30 个变量名；SKILL.md §4 含指针。

### Task 6: 补 `--no-stats` 参数 [BUG R4-6, LOW]
- **Priority**: low
- **Depends on**: —
- **Files**: `tradingagents-analysis/scripts/fetch_stock_data.py` (+ skills copy)
- **Description**: argparse 加 `--no-stats` (dest="stats", action="store_false")。
- **Test**: `python fetch_stock_data.py --symbol AAPL --no-stats --tail 5` 不报 unrecognized arguments。

### Task 7: 扩展 Tool-Name Override 覆盖 9 个 ghost tools [BUG R4-7, LOW]
- **Priority**: low
- **Depends on**: Task 5（同章节）
- **Files**: `tradingagents-analysis/references/prompts/README.md` (+ skills copy)
- **Description**: 扩展 "Tool-Name Override" 章节覆盖 `get_news`/`get_global_news`/`get_fundamentals`/`get_balance_sheet`/`get_cashflow`/`get_income_statement`/`get_macro_indicators`/`get_prediction_markets`。
- **Test**: prompts/README.md 含 `get_news` 和 `get_fundamentals` 映射。

### Task 8: 双拷贝同步
- **Priority**: high
- **Depends on**: Task 1-7
- **Files**: `skills/tradingagents-analysis/**`
- **Description**: 把 `tradingagents-analysis/` 的所有改动同步到 `skills/tradingagents-analysis/`。
- **Test**: `git diff --no-index --quiet tradingagents-analysis skills/tradingagents-analysis` 退出码 0。

### Task 9: 版本号 + CHANGELOG
- **Priority**: medium
- **Depends on**: Task 1-8
- **Files**: `package.json`, `CHANGELOG.md`
- **Description**: 版本 1.3.2 → 1.3.4（跳过 1.3.3 避免与 round-3 PR 冲突）；CHANGELOG 新增 `## [1.3.4]` 章节列 7 个修复。
- **Test**: package.json version == 1.3.4；CHANGELOG 含 `## [1.3.4]`。

### Task 10: 验证套件 + PR
- **Priority**: high
- **Depends on**: Task 1-9
- **Files**: `trad-r4-verify.py`（临时验证脚本，不入库）
- **Description**: 扩展 round-3 验证脚本为 round-4（新增 7 bug 检查 + 回归），运行通过后推送并创建 PR。
- **Test**: 41 round-3 检查 + N round-4 检查全部通过。

## Status

全部完成。7 个任务已实施并验证通过（66/66 检查点，含 41 round-3 回归 + 25 round-4 新增）。
