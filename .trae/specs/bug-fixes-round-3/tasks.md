# TradingAgents Skill Bug 修复 Round 3 - 任务列表

## [x] Task 1: 修复 `fetch_stock_data.py` `--start`/`--end` required 与文档矛盾 [BUG 1]
- **Priority**: high
- **Depends On**: None
- **Description**:
  - 将 `--start`/`--end` 改为 `required=False`，加默认值：`--end` 默认今天（`datetime.date.today().isoformat()`），`--start` 默认今天往前 365 天。
  - 在 `build_compact_report` / `fetch_stock_data` 入口处，若 `start_date`/`end_date` 为 None，用默认值填充。
  - 同步更新 SKILL.md §6 L242：补一句"若未传则脚本默认取今天往前 1 年；如需分析历史交易日，请显式传 --start/--end"。
  - 两份拷贝同步：`tradingagents-analysis/scripts/` 和 `skills/tradingagents-analysis/scripts/`。
- **Acceptance Criteria**:
  - `uv run python fetch_stock_data.py --symbol AAPL`（不传 --start/--end）不报 argparse 错误，正常返回报告。
  - `uv run python fetch_stock_data.py --symbol AAPL --start 2024-01-01 --end 2024-06-30` 仍正常工作（显式传参）。
  - SKILL.md §6 L242 文档与脚本行为一致。
  - `uv run python -c "import ast; ast.parse(open(f, encoding='utf-8').read())"` 通过。
- **Test**:
  - programmatic: `argparse` 不再 required=True for --start/--end
  - programmatic: 默认值逻辑正确（今天 / 今天-365d）
  - programmatic: 两份拷贝 diff 为空

## [x] Task 2: 修复 `fetch_cn_sentiment` 裸 `<unavailable>` [BUG 2]
- **Priority**: high
- **Depends On**: None
- **Description**:
  - `fetch_sentiment.py` L247-248：将 `if not has_data: return "<unavailable>"` 改为 `sections.append("\n> A 股情绪数据源全部不可用\n"); return "\n".join(sections)`。
  - 保留已构建的 `sections`（含 `## 个股评论\n\n> akshare 未安装，跳过` 等结构化错误块）。
  - 两份拷贝同步。
- **Acceptance Criteria**:
  - `fetch_cn_sentiment` 不再返回裸 `"<unavailable>"`。
  - 返回值含 `# A股情绪分析` 头部和结构化错误块。
  - `uv run python -c "import ast; ast.parse(open(f, encoding='utf-8').read())"` 通过。
- **Test**:
  - programmatic: grep `return "<unavailable>"` 在 `fetch_cn_sentiment` 函数内不再出现
  - programmatic: 两份拷贝 diff 为空

## [x] Task 3: 修复 `data-sources.md` CN 新闻降级链顺序 [BUG 3]
- **Priority**: medium
- **Depends On**: None
- **Description**:
  - `data-sources.md` L106：`Unified news tool (auto-detects market type) → Google News (Chinese) → AKShare news` 改为 `AKShare news → Google News (Chinese)`（与 `fetch_cn_news` 实际行为一致）。
  - 两份拷贝同步。
- **Acceptance Criteria**:
  - L106 顺序为 AKShare 在前、Google News 在后。
  - 与 `fetch_news.py` `fetch_cn_news` L190-212 行为一致。
- **Test**:
  - programmatic: grep 确认 L106 含 "AKShare news → Google News"
  - programmatic: 两份拷贝 diff 为空

## [x] Task 4: 修复 SKILL.md §3 News Analyst 过度声称 [BUG 4]
- **Priority**: medium
- **Depends On**: None
- **Description**:
  - SKILL.md L85 "Key inputs" 列：去掉 "global macro, FRED indicators, prediction markets"，改为 `Company news via \`scripts/fetch_news.py\``（FRED / Polymarket / macro 由 web-search fallback，不在脚本内）。
  - 两份拷贝同步。
- **Acceptance Criteria**:
  - §3 表格 News Analyst 行不再声称脚本提供 FRED / prediction markets / macro。
  - 与 §6 L238 描述（"Company news (US: yfinance + Google News RSS; A股: 东方财富/akshare)"）一致。
- **Test**:
  - programmatic: grep SKILL.md §3 确认无 "FRED indicators" / "prediction markets" 在 News Analyst 行
  - programmatic: 两份拷贝 diff 为空

## [x] Task 5: 修复 `install.mjs` L97 过时注释 [BUG 5]
- **Priority**: low
- **Depends On**: None
- **Description**:
  - install.mjs L97：`// 默认 Claude Code；若不存在则回退到 ~/.agents/skills` 改为 `// 默认 Claude Code`。
- **Acceptance Criteria**:
  - install.mjs 不再含"回退"字样（无对应代码）。
- **Test**:
  - programmatic: grep install.mjs 确认无"若不存在则回退"

## [x] Task 6: 修复 SKILL.md §3 Sentiment Analyst 过度声称 [BUG 6]
- **Priority**: low
- **Depends On**: None
- **Description**:
  - SKILL.md L84 "Key inputs" 列：去掉 "news headlines"，改为 `StockTwits, Reddit (US) / akshare (CN) via \`scripts/fetch_sentiment.py\``。
  - 两份拷贝同步。
- **Acceptance Criteria**:
  - §3 表格 Sentiment Analyst 行不再声称 news headlines。
  - 与 §6 L240 描述一致。
- **Test**:
  - programmatic: grep SKILL.md §3 确认无 "news headlines" 在 Sentiment Analyst 行
  - programmatic: 两份拷贝 diff 为空

## [x] Task 7: 修复 SKILL.md §6 示例时间窗口 [BUG 7]
- **Priority**: low
- **Depends On**: None
- **Description**:
  - SKILL.md L237：示例 `--start 2024-01-01 --end 2024-06-30`（6 个月）改为约 1 年，如 `--start 2023-07-01 --end 2024-06-30`，与 L242 "至少需 200 个交易日才能算 SMA200" 一致。
  - 两份拷贝同步。
- **Acceptance Criteria**:
  - §6 示例日期跨度 ≥ 10 个月。
  - 与 L242 默认窗口指引一致。
- **Test**:
  - programmatic: 检查示例 --start 到 --end 跨度 ≥ 10 个月
  - programmatic: 两份拷贝 diff 为空

## [x] Task 8: 修复 `fetch_stock_data.py` 负数 `--tail` 崩溃 [BUG 8]
- **Priority**: low
- **Depends On**: None
- **Description**:
  - `fetch_stock_data.py` `build_compact_report` 入口：`tail = max(0, int(tail))` 钳制负数。
  - 或在 argparse 后 `args.tail = max(0, args.tail)`。
  - 两份拷贝同步。
- **Acceptance Criteria**:
  - `--tail -5` 不崩溃，返回正常报告（最近 0 行 OHLCV 段）。
  - `uv run python -c "import ast; ast.parse(open(f, encoding='utf-8').read())"` 通过。
- **Test**:
  - programmatic: `--tail -5` 不抛 ValueError
  - programmatic: 两份拷贝 diff 为空

## [x] Task 9: 文档化 `prompts/README.md` 工具名 override [BUG 9]
- **Priority**: low
- **Depends On**: None
- **Description**:
  - `prompts/README.md` 加一段 "Tool-name override" 说明：`market_analyst.md` 等提示词引用的 `get_stock_data` / `get_indicators` / `get_verified_market_snapshot` 在本 skill 中由 `scripts/fetch_stock_data.py` 替代（见 SKILL.md §4 与 `indicators.md`）。
  - 不修改 verbatim prompt 文件本身。
  - 两份拷贝同步。
- **Acceptance Criteria**:
  - `prompts/README.md` 含 "get_stock_data" 或 "tool-name override" 说明。
  - `market_analyst.md` 等 prompt 文件未被修改（保持 verbatim）。
- **Test**:
  - programmatic: grep prompts/README.md 确认含 override 说明
  - programmatic: git diff 确认 prompt .md 文件未变
  - programmatic: 两份拷贝 diff 为空

## [x] Task 10: CHANGELOG + version bump
- **Priority**: medium
- **Depends On**: [Task 1-9]
- **Description**:
  - `CHANGELOG.md` 追加 1.3.2 条目，列出 9 个 Fixed 项。
  - `package.json` version: 1.3.1 → 1.3.2。
- **Acceptance Criteria**:
  - CHANGELOG 含 1.3.2 段落，覆盖所有 9 个 BUG。
  - package.json version == 1.3.2。
- **Test**:
  - programmatic: grep CHANGELOG 确认 1.3.2 段落
  - programmatic: grep package.json 确认 "version": "1.3.2"

## [x] Task 11: 最终验证
- **Priority**: high
- **Depends On**: [Task 1-10]
- **Description**:
  - 运行完整验证套件，确认所有修复未引入新问题。
- **Acceptance Criteria**:
  - 4 个 Python 脚本通过 ast.parse。
  - 两份 `tradingagents-analysis/` 目录 diff 为空。
  - install.mjs 功能测试通过（参数校验、~ 展开、__pycache__ 过滤）。
  - `fetch_stock_data.py --symbol AAPL`（无 --start/--end）不报错。
  - `fetch_stock_data.py --tail -5` 不崩溃。
  - `fetch_sentiment.py --symbol 600519`（akshare 不可用时）不返回裸 `<unavailable>`。
- **Test**:
  - programmatic: 全部上述检查通过


## Status
全部完成。9 个任务已实施并验证通过（41/41 检查点）。
