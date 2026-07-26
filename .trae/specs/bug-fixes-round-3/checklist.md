# TradingAgents Skill Bug 修复 Round 3 - 验证检查点

## BUG 1: `--start`/`--end` required vs 文档默认值
- [x] Checkpoint 1.1: `fetch_stock_data.py` argparse 中 `--start` 不再是 `required=True`
- [x] Checkpoint 1.2: `fetch_stock_data.py` argparse 中 `--end` 不再是 `required=True`
- [x] Checkpoint 1.3: `--start`/`--end` 缺省时脚本用今天往前 365 天 / 今天填充
- [x] Checkpoint 1.4: `uv run python fetch_stock_data.py --symbol AAPL`（不传 --start/--end）不报 argparse 错误
- [x] Checkpoint 1.5: SKILL.md §6 L242 补充"若未传则脚本默认取今天往前 1 年"说明
- [x] Checkpoint 1.6: 两份拷贝 `fetch_stock_data.py` diff 为空

## BUG 2: `fetch_cn_sentiment` 裸 `<unavailable>`
- [x] Checkpoint 2.1: `fetch_sentiment.py` `fetch_cn_sentiment` 函数不再 `return "<unavailable>"`
- [x] Checkpoint 2.2: 失败路径返回已构建的 `sections`（含 `# A股情绪分析` 头部 + 错误块）
- [x] Checkpoint 2.3: `uv run python -c "import ast; ast.parse(open('fetch_sentiment.py', encoding='utf-8').read())"` 通过
- [x] Checkpoint 2.4: 两份拷贝 `fetch_sentiment.py` diff 为空

## BUG 3: `data-sources.md` CN 新闻降级链
- [x] Checkpoint 3.1: `data-sources.md` L106 顺序为 `AKShare news → Google News (Chinese)`
- [x] Checkpoint 3.2: 与 `fetch_news.py` `fetch_cn_news` 实际行为一致（AKShare 优先）
- [x] Checkpoint 3.3: 两份拷贝 `data-sources.md` diff 为空

## BUG 4: SKILL.md §3 News Analyst 过度声称
- [x] Checkpoint 4.1: SKILL.md §3 表格 News Analyst 行不再含 "FRED indicators"
- [x] Checkpoint 4.2: SKILL.md §3 表格 News Analyst 行不再含 "prediction markets"
- [x] Checkpoint 4.3: SKILL.md §3 表格 News Analyst 行不再含 "global macro"
- [x] Checkpoint 4.4: 两份拷贝 SKILL.md diff 为空

## BUG 5: `install.mjs` 过时注释
- [x] Checkpoint 5.1: install.mjs 不再含 "若不存在则回退到 ~/.agents/skills"
- [x] Checkpoint 5.2: install.mjs 仍含 `// 默认 Claude Code` 注释

## BUG 6: SKILL.md §3 Sentiment Analyst "news headlines"
- [x] Checkpoint 6.1: SKILL.md §3 表格 Sentiment Analyst 行不再含 "news headlines"
- [x] Checkpoint 6.2: 两份拷贝 SKILL.md diff 为空

## BUG 7: SKILL.md §6 示例时间窗口
- [x] Checkpoint 7.1: SKILL.md §6 L237 示例 `--start` 到 `--end` 跨度 ≥ 10 个月
- [x] Checkpoint 7.2: 两份拷贝 SKILL.md diff 为空

## BUG 8: 负数 `--tail` 崩溃
- [x] Checkpoint 8.1: `fetch_stock_data.py` `build_compact_report` 入口钳制 `tail = max(0, int(tail))` 或等价
- [x] Checkpoint 8.2: `--tail -5` 不抛 ValueError（返回正常报告）
- [x] Checkpoint 8.3: 两份拷贝 `fetch_stock_data.py` diff 为空

## BUG 9: `prompts/README.md` 工具名 override 文档
- [x] Checkpoint 9.1: `prompts/README.md` 含 "get_stock_data" 或 "tool-name override" 说明
- [x] Checkpoint 9.2: `market_analyst.md` 等 prompt 文件未被修改（git diff 为空）
- [x] Checkpoint 9.3: 两份拷贝 `prompts/README.md` diff 为空

## 全局验证
- [x] Checkpoint 10.1: 4 个 Python 脚本通过 ast.parse 语法检查
- [x] Checkpoint 10.2: `diff -r tradingagents-analysis/ skills/tradingagents-analysis/` 为空
- [x] Checkpoint 10.3: install.mjs 功能测试（--dir 校验、~ 展开、__pycache__ 过滤）通过
- [x] Checkpoint 10.4: `package.json` version == 1.3.2
- [x] Checkpoint 10.5: CHANGELOG.md 含 1.3.2 段落，覆盖 9 个 BUG
- [x] Checkpoint 10.6: 无新依赖引入（仅 yfinance/akshare/requests/pandas）
- [x] Checkpoint 10.7: 代码风格一致（中文注释、函数式、无 class）
