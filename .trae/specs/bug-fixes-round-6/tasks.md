# Round 6 — Tasks

**Branch**: `fix/round6-bugs` (base: `fix/round5-bugs`)

## Task 1: HIGH fixes — Python (R6-1, R6-2) [HIGH]

- [ ] R6-1: `fetch_fundamentals.py` `fetch_us_fundamentals` ticker 作用域 — 预声明 `ticker = None`，三大报表前加 `if ticker is None: financials = balance = cashflow = None` 短路
- [ ] R6-2: `fetch_news.py` `fetch_yfinance_news` days 参数 — 循环内对 `publishTime`/`providerPublishTime` 做 `>= now - timedelta(days=days)` 过滤；保留 `from datetime import datetime, timedelta` import（R6-13 联动）
- [ ] 同步 skills/ 副本
- [ ] 验证：`uv run python -c "..."` mock yf.Ticker 抛异常，确认三大报表不抛 NameError；mock news 含时间字段，确认 days 过滤生效
- [ ] Commit: `fix(scripts): fetch_us_fundamentals ticker scope + fetch_yfinance_news days filter (R6-1, R6-2)`

## Task 2: HIGH fixes — Prompts (R6-3, R6-4) [HIGH]

- [ ] R6-3: `prompts/README.md` L69-72 + `SKILL.md` L163 Quick reference — 把 `{target_label}`/`{asset_label}`/`{fundamentals_label}` 替换规则改为与源仓库一致（stock/asset, company/asset, Company fundamentals report/...）
- [ ] R6-4: 6 个 prompt front-matter "Template variables" 改为只列 body 实际变量：
  - `china_market_analyst.md` → "(none — body is static text)"
  - `cn_news_analyst.md` → "(none — body is static text)"
  - `market_analyst.md` → `{get_language_instruction()}`
  - `fundamentals_analyst.md` → `{get_language_instruction()}`
  - `news_analyst.md` → `{asset_label}`, `{get_language_instruction()}`
  - `sentiment_analyst.md` → `{get_language_instruction()}` (verify body vars)
- [ ] R6-4 Note: `prompts/README.md` 加 Note 说明 `{tool_names}/{current_date}/{instrument_context}/{system_message}` 由 SKILL.md §4 spawn 模板在外层处理（联动 R6-12）
- [ ] 同步 skills/ 副本
- [ ] Commit: `fix(prompts): correct 3 var substitution rules + 6 front-matter phantom vars (R6-3, R6-4)`

## Task 3: MEDIUM fixes — Python (R6-6, R6-7, R6-8) [HIGH]

- [ ] R6-6: `fetch_stock_data.py` `fetch_cn_stock_data` / `fetch_hk_stock_data` 顶部加日期守卫 `if not isinstance(start_date, str) or not isinstance(end_date, str): return "错误: 日期参数无效"`
- [ ] R6-7: `fetch_stock_data.py` `compute_indicators` RSI — 处理 `avg_loss == 0` 返回 100：
  ```python
  rs = avg_gain / avg_loss  # 不 replace 0
  rsi = pd.Series(np.nan, index=close.index)  # 需 import numpy as np
  mask_zero = avg_loss == 0
  rsi[mask_zero] = 100.0
  mask_nonzero = ~mask_zero
  rsi[mask_nonzero] = 100 - 100 / (1 + rs[mask_nonzero])
  ```
- [ ] R6-8: `fetch_stock_data.py` Bollinger — `close.rolling(20).std(ddof=0)`
- [ ] R6-5 文档化: 同一函数 RSI 注释改为"RSI(14)：Wilder 平滑法的 pandas ewm 简化实现，与标准 Wilder（SMA 种子）偏差约 1pp"
- [ ] 同步 skills/ 副本
- [ ] 验证：mock 持续上涨序列确认 RSI=100；mock 一般序列确认 Bollinger 与 ddof=0 一致
- [ ] Commit: `fix(scripts): date guards + RSI/Bollinger correctness + Wilder note (R6-5, R6-6, R6-7, R6-8)`

## Task 4: MEDIUM fixes — Prompts/SKILL (R6-9, R6-10, R6-11, R6-12) [HIGH]

- [ ] R6-9: `SKILL.md` §4 "Re-injection discipline" Stage 6 措辞改为 "append the full reports as out-of-template context (e.g., prepend them to the prompt as a '## Analyst Reports' section)"
- [ ] R6-10: `SKILL.md` L130 CN swap — "其余 3 个分析师" → "其余 2 个分析师（Sentiment / Fundamentals）保持不变；Stage 2 及之后的 researcher/manager/risk debator 等角色不受 market 影响"
- [ ] R6-11: `prompts/README.md` L86 加 Note — "trad-skill uses a compact one-liner instead of the source's full paragraph for token efficiency..."
- [ ] R6-12: `prompts/README.md` L62 加注 — "Of these 30, 3 (`{current_date}`, `{tool_names}`, `{system_message}`) appear only in the source's outer ChatPromptTemplate..."
- [ ] 同步 skills/ 副本
- [ ] Commit: `fix(docs): SKILL re-injection + CN swap count + 2 README notes (R6-9, R6-10, R6-11, R6-12)`

## Task 5: LOW fixes — Python batch 1 (R6-13 ~ R6-18) [MEDIUM]

- [ ] R6-13: `fetch_news.py` 死代码 import — R6-2 修复后 `timedelta` 已被使用，确认保留；`datetime` 若仍无引用则删除
- [ ] R6-14: `fetch_stock_data.py` `_val` inf 处理 — `if not pd.notna(v) or not np.isfinite(v): return "N/A"`（需 `import numpy as np` 或用 `math.isfinite(float(v))`）
- [ ] R6-15: `fetch_sentiment.py` `fetch_stocktwits` docstring "默认 30" → "默认 15"
- [ ] R6-16: `fetch_news.py` / `fetch_sentiment.py` 各入口加 `limit = max(0, int(limit))` 和 `days = max(1, int(days))` 钳制
- [ ] R6-17: `fetch_stock_data.py` `compute_stats` 注释 "日对数收益" → "日百分比收益"
- [ ] R6-18: `fetch_stock_data.py` `compute_indicators` MFI — `pos_sum==0 & neg_sum==0` 时 mfi=50
- [ ] 同步 skills/ 副本
- [ ] Commit: `fix(scripts): low-priority cleanup — dead import, inf, docstring, negative clamp, comment, MFI flat (R6-13~R6-18)`

## Task 6: LOW fixes — Python batch 2 (R6-19, R6-20, R6-21, R6-22) [MEDIUM]

- [ ] R6-19: `fetch_sentiment.py` symbol URL 编码 — 代码注释加说明（不改代码）
- [ ] R6-20: `fetch_news.py` `fetch_yfinance_news` 循环内 `try/except` 包裹每条 item 解析，`continue` 跳过坏数据
- [ ] R6-21: `fetch_stock_data.py` `build_compact_report` `tail=None` 守卫 — `tail = max(0, int(tail)) if tail is not None else 0`
- [ ] R6-22: `fetch_fundamentals.py` / `fetch_stock_data.py` 北交所/B 股注释 — 代码注释加说明（不改代码）
- [ ] 同步 skills/ 副本
- [ ] Commit: `fix(scripts): robustness — news item guard, tail=None, 2 limitation notes (R6-19~R6-22)`

## Task 7: LOW fixes — Prompts/SKILL (R6-23 ~ R6-27) [MEDIUM]

- [ ] R6-23: `indicators.md` 加 Note — "源仓库 market_analyst.py 的指标列表未列 MFI，但 fetch_stock_data.py 会预计算 MFI..."
- [ ] R6-24: `prompts/README.md` 加 Note — "whitespace before `{get_language_instruction()}` normalized to one blank line for readability"
- [ ] R6-25: `SKILL.md` §5 Stage 4 加 Note — "trader.md has separate System Message and User Message blocks..."
- [ ] R6-26: `prompts/README.md` L87 — English 时空字符串，非 English 时 `Write your entire response in {lang}.`
- [ ] R6-27: `SKILL.md` §6 表格 fetch_stock_data.py 行补 Flags 列表
- [ ] 同步 skills/ 副本
- [ ] Commit: `fix(docs): 5 prompt/SKILL notes — MFI, whitespace, trader 2-block, lang instr, §6 flags (R6-23~R6-27)`

## Task 8: LOW fixes — Docs/Installer (R6-28, R6-29, R6-30) [MEDIUM]

- [ ] R6-28: `install.mjs` L107 `path.join(parentDir, SKILL_NAME)` → `path.resolve(parentDir, SKILL_NAME)`
- [ ] R6-29: `README.md` L144 `...` → `—`
- [ ] R6-30: `README_CN.md` L136 "5位数字" → "4-5 位数字 + `.HK` 后缀（如 0700.HK 或 00700.HK）"
- [ ] 验证：`node --check install.mjs`；`node install.mjs --dir ./foo`（在 %TEMP%）确认 destDir 显示绝对路径
- [ ] Commit: `fix(install+docs): destDir path.resolve + README ...→— + README_CN 港股位数 (R6-28~R6-30)`

## Task 9: Verify + spec + changelog + version [HIGH]

- [ ] 写 `verify_round6.py` — 逐 bug 验证脚本（目标 ≥30 项检查）
- [ ] 跑 `uv run --with yfinance,pandas,akshare,requests,numpy python .trae/specs/bug-fixes-round-6/verify_round6.py`，确认全过
- [ ] `git diff --no-index --stat tradingagents-analysis skills/tradingagents-analysis` 确认仅 __pycache__ 差异
- [ ] `CHANGELOG.md` 加 `## [1.3.6] - 2026-07-26` 章节
- [ ] `package.json` version 1.3.5 → 1.3.6
- [ ] 标记 spec/tasks 完成
- [ ] Commit: `chore(release): bump to 1.3.6 with round-6 changelog + verify_round6.py`
- [ ] Push branch
- [ ] 创建 PR #7 (base: fix/round5-bugs)

## Dependency order

Task 1 → Task 2 → Task 3 → Task 4 → Task 5 → Task 6 → Task 7 → Task 8 → Task 9

Task 1 和 Task 2 可并行（不同文件），但为提交清晰度串行。Task 3 依赖 Task 1（同文件 fetch_stock_data.py 的 compute_indicators 与 fetch_us_fundamentals 不冲突，但同提交批次清晰）。Task 5/6 都改 Python 脚本，需串行。
