# Round 6 — Tasks

**Branch**: `fix/round6-bugs` (base: `fix/round5-bugs`)
**Status**: ✅ ALL TASKS COMPLETE (32/32 verify_round6.py checks pass)

## Task 1: HIGH fixes — Python (R6-1, R6-2) [HIGH] ✅

- [x] R6-1: `fetch_fundamentals.py` `fetch_us_fundamentals` ticker 作用域 — 预声明 `ticker = None`，三大报表前加 `if ticker is None: financials = balance = cashflow = None` 短路
- [x] R6-2: `fetch_news.py` `fetch_yfinance_news` days 参数 — 新增 `_parse_news_time` 辅助函数，循环内对 `pub_time < cutoff` 过滤；保留 `from datetime import datetime, timedelta, timezone` import（R6-13 联动）
- [x] 同步 skills/ 副本
- [x] 验证：mock yf.Ticker 抛异常确认三大报表不抛 NameError；mock news 含时间字段确认 days 过滤生效
- [x] Commit: `7f2034f fix(scripts): fetch_us_fundamentals ticker scope + fetch_yfinance_news days filter (R6-1, R6-2)`

## Task 2: HIGH fixes — Prompts (R6-3, R6-4) [HIGH] ✅

- [x] R6-3: `prompts/README.md` L69-72 + `SKILL.md` L163 Quick reference — 三个变量替换规则改为与源仓库一致
- [x] R6-4: 6 个 prompt front-matter "Template variables" 改为只列 body 实际变量
- [x] R6-4 Note: `prompts/README.md` 加 Note 说明 phantom variables（联动 R6-12）
- [x] 同步 skills/ 副本
- [x] Commit: `0206354 fix(prompts): correct 3 var substitution rules + 6 front-matter phantom vars (R6-3, R6-4, R6-11, R6-12, R6-24, R6-26)`

## Task 3: MEDIUM fixes — Python (R6-5, R6-6, R6-7, R6-8) [HIGH] ✅

- [x] R6-6: `fetch_stock_data.py` `fetch_cn_stock_data` / `fetch_hk_stock_data` 顶部加日期守卫
- [x] R6-7: `fetch_stock_data.py` `compute_indicators` RSI — 处理 `avg_loss == 0` 返回 100
- [x] R6-8: `fetch_stock_data.py` Bollinger — `close.rolling(20).std(ddof=0)`
- [x] R6-5 文档化: RSI 注释改为 "Wilder 平滑法的 pandas ewm 简化实现，偏差约 1pp"
- [x] 同步 skills/ 副本
- [x] 验证：mock 持续上涨序列确认 RSI=100；mock 一般序列确认 Bollinger 与 ddof=0 一致
- [x] Commit: `457dab7 fix(scripts+skill): round-6 MEDIUM fixes (R6-5/6/7/8/9/10)`

## Task 4: MEDIUM fixes — Prompts/SKILL (R6-9, R6-10, R6-11, R6-12) [HIGH] ✅

- [x] R6-9: `SKILL.md` §4 Stage 6 措辞改为 "append the full reports as out-of-template context"
- [x] R6-10: `SKILL.md` L130 CN swap — "3 个" → "2 个"
- [x] R6-11: `prompts/README.md` 加 Note（instrument_context token 效率取舍）
- [x] R6-12: `prompts/README.md` 加 Note（phantom variables）— 最初漏加，在 verify_round6.py 检出后补加
- [x] 同步 skills/ 副本
- [x] Commit: `457dab7` (R6-9/10) + R6-12 补加在后续提交

## Task 5: LOW fixes — Python batch 1 (R6-13 ~ R6-18) [MEDIUM] ✅

- [x] R6-13: `fetch_news.py` 死代码 import — R6-2 修复后 `timedelta/timezone` 已被使用，import 保留
- [x] R6-14: `fetch_stock_data.py` `_val` inf 处理 — 用 `math.isfinite(float(v))` 排除 ±inf
- [x] R6-15: `fetch_sentiment.py` `fetch_stocktwits` docstring "默认 30" → "默认 15"
- [x] R6-16: `fetch_news.py` / `fetch_sentiment.py` 各入口加 `limit = max(0, int(limit))` 和 `days = max(1, int(days))` 钳制
- [x] R6-17: `fetch_stock_data.py` `compute_stats` 注释 "日对数收益" → "日百分比收益"
- [x] R6-18: `fetch_stock_data.py` `compute_indicators` MFI — 三种边缘情况（both_zero→50, neg_zero→100, pos_zero→0）
- [x] 同步 skills/ 副本
- [x] Commit: `3a37db0 fix(scripts): round-6 LOW Python fixes (R6-13/14/15/16/17/18/19/20/21/22)`

## Task 6: LOW fixes — Python batch 2 (R6-19, R6-20, R6-21, R6-22) [MEDIUM] ✅

- [x] R6-19: `fetch_sentiment.py` symbol URL 编码 — 代码注释加说明（不改代码）
- [x] R6-20: `fetch_news.py` `fetch_yfinance_news` content.get 类型守卫（R6-2 已加 `isinstance(content, dict)`）
- [x] R6-21: `fetch_stock_data.py` `build_compact_report` `tail=None` 守卫
- [x] R6-22: `fetch_fundamentals.py` / `fetch_stock_data.py` 北交所/B 股注释
- [x] 同步 skills/ 副本
- [x] Commit: `3a37db0`（与 Task 5 合并提交）

## Task 7: LOW fixes — Prompts/SKILL (R6-23 ~ R6-27) [MEDIUM] ✅

- [x] R6-23: `indicators.md` 加 Note（MFI verbatim-vs-script 不一致）+ R6-5/7/8 实现细节 Note
- [x] R6-24: `prompts/README.md` 加 Note（whitespace before get_language_instruction）
- [x] R6-25: `SKILL.md` §5 Stage 4 加 Note（trader.md 2-block 结构）
- [x] R6-26: `prompts/README.md` L87 — English 时空字符串，非 English 时 ` Write your entire response in <lang>.`
- [x] R6-27: `SKILL.md` §6 表格 fetch_stock_data.py 行补 Flags 列表
- [x] 同步 skills/ 副本
- [x] Commit: `c251e00 docs(skill+installer): round-6 LOW docs/installer fixes (R6-23/25/27/28/29/30)`

## Task 8: LOW fixes — Docs/Installer (R6-28, R6-29, R6-30) [MEDIUM] ✅

- [x] R6-28: `install.mjs` L107 `path.join` → `path.resolve`
- [x] R6-29: `README.md` L144 `...` → `—`
- [x] R6-30: `README_CN.md` L136 "5位数字" → "4-5 位数字 + `.HK` 后缀"
- [x] 验证：`node --check install.mjs` 通过
- [x] Commit: `c251e00`（与 Task 7 合并提交）

## Task 9: Verify + spec + changelog + version [HIGH] ✅

- [x] 写 `verify_round6.py` — 32 项检查（30 bug + AST + 副本一致性）
- [x] 跑 `uv run python verify_round6.py`，32/32 全过
- [x] `git diff --no-index --stat tradingagents-analysis skills/tradingagents-analysis` 确认仅 __pycache__ 差异
- [x] `CHANGELOG.md` 加 `## [1.3.6] - 2026-07-26` 章节
- [x] `package.json` version 1.3.5 → 1.3.6
- [x] 标记 spec/tasks 完成
- [x] Commit: `chore(release): bump to 1.3.6 with round-6 changelog + verify_round6.py`
- [ ] Push branch
- [ ] 创建 PR #7 (base: fix/round5-bugs)

## Dependency order

Task 1 → Task 2 → Task 3 → Task 4 → Task 5 → Task 6 → Task 7 → Task 8 → Task 9

Task 1 和 Task 2 可并行（不同文件），但为提交清晰度串行。Task 3 依赖 Task 1（同文件 fetch_stock_data.py 的 compute_indicators 与 fetch_us_fundamentals 不冲突，但同提交批次清晰）。Task 5/6 都改 Python 脚本，需串行。
