# Round 4 — Checklist

**Spec**: `spec.md`
**Tasks**: `tasks.md`

## BUG R4-1 — fetch_fundamentals 空串/None 不抛异常

- [ ] `fetch_fundamentals.py` `fetch_fundamentals` 入口有 `isinstance(symbol, str)` 守卫
- [ ] `fetch_fundamentals.py` `fetch_fundamentals` 入口对空串返回错误字符串
- [ ] `fetch_fundamentals.py` `fetch_us_fundamentals` 的 `yf.Ticker(symbol)` 在 try/except 内
- [ ] `fetch_fundamentals("")` 返回字符串不抛异常（运行时验证）
- [ ] `fetch_fundamentals(None)` 返回字符串不抛异常（运行时验证）
- [ ] `fetch_fundamentals("AAPL")` 仍正常返回报告（回归）

## BUG R4-2 — compute_stats nan% → N/A

- [ ] `fetch_stock_data.py` `compute_stats` 有 `_num` helper 或等价 NaN 守卫
- [ ] `ret` 为 NaN 时输出 "N/A"
- [ ] `vol` 为 NaN 时输出 "N/A"
- [ ] `hi`/`lo` 为 NaN 时输出 "N/A"
- [ ] 单行 DataFrame 调用 `compute_stats` 输出含 "N/A" 不含 "nan%"
- [ ] 正常 DataFrame 仍输出数值（回归）

## BUG R4-3 — README 示例日期同步

- [ ] README.md 无 `--start 2024-01-01 --end 2024-06-30`（已改为 2023-07-01）
- [ ] README.md 无 `--start 2024-01-01 --end 2024-01-31`
- [ ] README_CN.md 无 `--start 2024-01-01 --end 2024-06-30`（已改为 2023-07-01）
- [ ] README_CN.md 无 `--start 2024-01-01 --end 2024-01-31`
- [ ] README.md / README_CN.md 示例窗口与 SKILL.md §6 一致（≥ 10 个月）

## BUG R4-4 — sentiment 数据块映射文档化

- [ ] prompts/README.md 含 `{stocktwits_block}` 映射说明
- [ ] prompts/README.md 含 `{reddit_block}` 映射说明
- [ ] prompts/README.md 含 `{news_block}` 映射说明（标注脚本不提供）

## BUG R4-5 — 30 个模板变量替换规则文档化

- [ ] prompts/README.md 有 "Template Variable Substitution" 章节
- [ ] 章节含 "Identity & labels" 子表（{ticker}/{target_label}/{asset_label}/{company_name}/{fundamentals_label}）
- [ ] 章节含 "Dates" 子表（{current_date}/{start_date}/{end_date}）
- [ ] 章节含 "Context & language" 子表（{instrument_context}/{get_language_instruction()}/{tool_names}/{system_message}/{NO_EXTERNAL_TOOLS}/{lessons_line}）
- [ ] 章节含 "Data reports" 子表（{market_research_report}/{sentiment_report}/{news_report}/{fundamentals_report}/{history}/{investment_plan}/{trader_decision}/{trader_plan}/{research_plan}/{current_response}/{current_aggressive_response}/{current_conservative_response}/{current_neutral_response}）
- [ ] 章节含 "Pre-fetched data blocks" 子表（{news_block}/{stocktwits_block}/{reddit_block}）
- [ ] SKILL.md §4 有指向 prompts/README.md 的指针

## BUG R4-6 — --no-stats 参数

- [ ] `fetch_stock_data.py` argparse 有 `--no-stats` (dest="stats", action="store_false")
- [ ] `python fetch_stock_data.py --symbol AAPL --no-stats --tail 5` 不报 unrecognized arguments（运行时）

## BUG R4-7 — Tool-Name Override 扩展

- [ ] prompts/README.md "Tool-Name Override" 含 `get_news` / `get_global_news` 映射
- [ ] prompts/README.md "Tool-Name Override" 含 `get_fundamentals` 映射
- [ ] prompts/README.md "Tool-Name Override" 含 `get_balance_sheet` / `get_cashflow` / `get_income_statement` 说明
- [ ] prompts/README.md "Tool-Name Override" 含 `get_macro_indicators` / `get_prediction_markets` 说明

## 双拷贝同步

- [ ] `git diff --no-index --quiet tradingagents-analysis skills/tradingagents-analysis` 退出码 0

## 版本 + CHANGELOG

- [ ] package.json version == 1.3.4
- [ ] CHANGELOG.md 含 `## [1.3.4]`
- [ ] CHANGELOG 1.3.4 章节含 7 个 Fixed 项

## 回归（Round 1-3 检查仍通过）

- [ ] 4 个脚本 ast.parse 通过
- [ ] 无 `class` 关键字
- [ ] 第三方依赖仅 yfinance/akshare/requests/pandas
- [ ] akshare 仍是软依赖（try/except ImportError）
- [ ] Round 3 的 9 个修复仍生效（41 检查点全通过）
