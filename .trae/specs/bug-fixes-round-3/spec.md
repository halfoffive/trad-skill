# TradingAgents Skill Bug 修复 - Round 3 规范

## 背景

Round 2 PR #3 已提交（MERGEABLE/CLEAN），修复了 12 个 BUG + 11 个文档/契约问题。
Round 3 通过 5 个并行子代理多角度复审当前状态（含 Round 2 修复），找出残留或新引入的 bug。

## 审查方法

- 5 个并行 Explore 子代理：Python 脚本、install.mjs、SKILL.md 契约、文档一致性、跨拷贝一致性
- 子代理返回的发现质量参差（多处误报，如声称 `_yoy()` 未处理 NaN 但实际有 `.dropna()`）
- 对每条发现通过 Read 工具读取实际代码行验证，剔除误报
- 跨拷贝一致性子代理确认两份 `tradingagents-analysis/` 字节一致 ✅

## 确认的 BUG（9 个）

### BUG 1 [HIGH] — `fetch_stock_data.py` `--start`/`--end` required=True 与 SKILL.md 文档默认值矛盾

- **文件**: `tradingagents-analysis/scripts/fetch_stock_data.py:467-477`（`required=True`）；`tradingagents-analysis/SKILL.md:242`（"默认 `--start` 取 trade date 前 1 年、`--end` 取 trade date 当天"）
- **现状**: 脚本 `--start`/`--end` 是 `required=True`，缺失时 argparse 报错退出（exit code 2），子代理看到 `error: the following arguments are required: --start, --end`。
- **契约矛盾**: SKILL.md §6 L242 明确说"默认 `--start` 取 trade date 前 1 年、`--end` 取 trade date 当天"，暗示脚本有默认值。代理若按文档省略这两个参数，脚本直接拒绝运行。
- **修复**: 将 `--start`/`--end` 改为可选（`required=False`），默认 `--end` = 今天、`--start` = 今天往前 365 天。SKILL.md L242 补一句"若未传则脚本默认取今天往前 1 年；如需分析历史交易日，请显式传 --start/--end"。
- **验证**: `uv run python fetch_stock_data.py --symbol AAPL`（不传 --start/--end）应正常返回，不报 argparse 错误。

### BUG 2 [MEDIUM] — `fetch_cn_sentiment` 全部数据源失败时返回裸 `<unavailable>`

- **文件**: `tradingagents-analysis/scripts/fetch_sentiment.py:247-248`
- **现状**: `fetch_cn_sentiment` 在 `has_data == False` 时 `return "<unavailable>"`，丢弃已构建的 `sections`（含 `> akshare 未安装，跳过` / `> 获取失败` / `> 无数据` 等友好错误块）。
- **回归**: Round 1 修复（CHANGELOG L10）的目标是"消除裸 `<unavailable>` 占位符，改为友好错误提示块"。US 分支已在 `fetch_sentiment` L278/L286 包裹；**CN 分支被遗漏** —— `fetch_sentiment` L269-270 直接 `return fetch_cn_sentiment(symbol)`，未包裹。
- **修复**: `fetch_cn_sentiment` L247-248 改为 `sections.append("\n> A 股情绪数据源全部不可用\n"); return "\n".join(sections)`，保留已构建的结构化错误报告。
- **验证**: `uv run python fetch_sentiment.py --symbol 600519`（在 akshare 不可用的环境）应返回带 `## 个股评论\n\n> akshare 未安装，跳过` 等结构化块的报告，而非裸 `<unavailable>`。

### BUG 3 [MEDIUM] — `data-sources.md` L106 CN 新闻降级链顺序反了

- **文件**: `tradingagents-analysis/references/data-sources.md:106`
- **现状**: 文档写 `Unified news tool (auto-detects market type) → Google News (Chinese) → AKShare news`，即 Google News 优先、AKShare 兜底。
- **实际**: `fetch_news.py` `fetch_cn_news` L190-212 是 **AKShare 优先**（`ak.stock_news_em`），失败才降级到 Google News。Round 2 CHANGELOG L34 声称已修复 A 股降级链为 `AKShare → yfinance`，但 CN 新闻这条链反了。
- **修复**: L106 改为 `AKShare news → Google News (Chinese)`。
- **验证**: grep `data-sources.md` 确认 CN 新闻链顺序与 `fetch_cn_news` 实际行为一致。

### BUG 4 [MEDIUM] — SKILL.md §3 L85 News Analyst 过度声称（FRED / prediction markets）

- **文件**: `tradingagents-analysis/SKILL.md:85`
- **现状**: `| **News Analyst** | Company news, global macro, FRED indicators, prediction markets | News feeds via \`scripts/fetch_news.py\` |` —— "global macro, FRED indicators, prediction markets" 由 `fetch_news.py` 提供。
- **实际**: `fetch_news.py` 只抓公司新闻（yfinance + Google News），不抓 FRED / Polymarket。Round 2 CHANGELOG L30 声称"§6 表格过度声称：`fetch_news.py` 描述去掉 'macro'"，但只修了 §6，**§3 漏修**。
- **修复**: §3 L85 "Key inputs" 列改为 `Company news via \`scripts/fetch_news.py\` (FRED / Polymarket / macro: web-search fallback only)`，或直接去掉 "global macro, FRED indicators, prediction markets"。
- **验证**: grep SKILL.md §3 表格，确认 News Analyst 行不再声称脚本提供 FRED/prediction markets。

### BUG 5 [LOW] — `install.mjs` L97 过时注释（声称回退但无回退代码）

- **文件**: `install.mjs:97`
- **现状**: `// 默认 Claude Code；若不存在则回退到 ~/.agents/skills` —— 但 L98 仅 `parentDir = AGENT_DIRS.claude;`，无任何回退逻辑。
- **修复**: 删除"若不存在则回退到 ~/.agents/skills"半句，保留 `// 默认 Claude Code`。
- **验证**: grep install.mjs 确认无"回退"字样。

### BUG 6 [LOW] — SKILL.md §3 L84 Sentiment Analyst 过度声称 "news headlines"

- **文件**: `tradingagents-analysis/SKILL.md:84`
- **现状**: `| **Sentiment Analyst** | Social and headline sentiment → composite score | StockTwits, Reddit, news headlines via \`scripts/fetch_sentiment.py\` |` —— "news headlines via `scripts/fetch_sentiment.py`"。
- **实际**: `fetch_sentiment.py` 只抓 StockTwits + Reddit（US 分支）和 akshare 个股评论/机构参与度（CN 分支），**不抓 news headlines**。
- **修复**: L84 "Key inputs" 列去掉 "news headlines"，改为 `StockTwits, Reddit (US) / akshare (CN) via \`scripts/fetch_sentiment.py\``。
- **验证**: grep SKILL.md §3 确认 Sentiment Analyst 行不再声称 news headlines。

### BUG 7 [LOW] — SKILL.md §6 L237 示例时间窗口短于 SMA200 所需

- **文件**: `tradingagents-analysis/SKILL.md:237`
- **现状**: 示例 `--start 2024-01-01 --end 2024-06-30`（约 6 个月），但 L242 明确说"至少需 200 个交易日才能算 SMA200"（≈10 个月）。示例与指引自相矛盾。
- **修复**: 将示例窗口扩到约 1 年，如 `--start 2023-07-01 --end 2024-06-30`，与 L242 默认窗口指引一致。
- **验证**: 检查 SKILL.md §6 示例日期跨度 ≥ 10 个月。

### BUG 8 [LOW] — `fetch_stock_data.py` 负数 `--tail` 触发未捕获 ValueError

- **文件**: `tradingagents-analysis/scripts/fetch_stock_data.py:448`
- **现状**: `build_compact_report` L448 `tail_df = df.tail(tail)` 不在 try/except 内。`--tail -5` → `df.tail(-5)` 抛 `ValueError`，向上传播到 CLI L519（也无 try/except），脚本崩溃。
- **契约违反**: AGENTS.md 规定"Every function returns a formatted string, never raises"。
- **修复**: `build_compact_report` 入口处 `tail = max(0, int(tail))` 钳制负数；或在 argparse 加 `tail = max(0, args.tail)` 后再传入。
- **验证**: `uv run python fetch_stock_data.py --symbol AAPL --start 2024-01-01 --end 2024-06-30 --tail -5` 不崩溃，返回正常报告（最近 0 行 OHLCV 段）。

### BUG 9 [LOW] — `prompts/README.md` 未文档化 market_analyst.md 中的不存在工具引用

- **文件**: `tradingagents-analysis/references/prompts/market_analyst.md:36,38`；`references/prompts/README.md`（缺说明）
- **现状**: `market_analyst.md` 是 verbatim 提取，L36 引用 `get_stock_data` / `get_indicators`，L38 引用 `get_verified_market_snapshot` —— 这些工具在本 skill 中不存在。SKILL.md §4 L155-159 和 `indicators.md` L6 已有 override 说明，但 `prompts/README.md` 没有提示读者。
- **约束**: AGENTS.md 规定 prompts 必须 verbatim，不能改写。所以修复点在 `prompts/README.md` 加注，不改 prompt 本身。
- **修复**: `prompts/README.md` 加一段 "Tool-name override" 说明：`market_analyst.md` 等提示词引用的 `get_stock_data` / `get_indicators` / `get_verified_market_snapshot` 在本 skill 中由 `scripts/fetch_stock_data.py` 替代（见 SKILL.md §4 与 `indicators.md`）。
- **验证**: grep prompts/README.md 确认含 "get_stock_data" 或 "tool-name override" 说明。

## 不修复的项（误报或设计选择）

- **`_yoy()` 未处理 NaN** — 误报。L43 `pd.to_numeric(series, errors="coerce").dropna()` 已处理。
- **`install.mjs` 路径遍历** — 设计选择。`--dir` 是用户显式输入，用户可写到任意目录，不是安全漏洞。
- **`install.mjs` Windows 路径分隔符** — 误报。`path.join` 跨平台处理。
- **`fetch_reddit_sentiment` `days` 参数未从 CLI 传入** — 设计选择。`days` 默认 7 天，CLI 不暴露 `--days` for sentiment，符合精简原则。
- **`fetch_yfinance_news` `days` 仅用于表头不用于过滤** — yfinance API 限制，不可修。
- **HK 5 位数字 ticker 走 US fundamentals 路径** — 边缘情况。用户应使用 `0700.HK` 格式，5 位纯数字非主流用法。
- **`compute_stats` `first = 0` 打印 nan%** — 股价不可能为 0，无实际影响。
- **跨拷贝一致性** — 子代理确认两份 `tradingagents-analysis/` 字节一致 ✅。

## 修复原则（karpathy-guidelines）

- 外科式修改：每条修复只动相关行，不顺手重构。
- 简洁优先：不引入新依赖、新抽象。
- 验证驱动：每条修复都有可执行的验证命令。
- 两份拷贝同步：`tradingagents-analysis/` 和 `skills/tradingagents-analysis/` 必须保持字节一致。

## 影响范围

- `tradingagents-analysis/scripts/fetch_stock_data.py`（+ `skills/` 同步）
- `tradingagents-analysis/scripts/fetch_sentiment.py`（+ `skills/` 同步）
- `tradingagents-analysis/SKILL.md`（+ `skills/` 同步）
- `tradingagents-analysis/references/data-sources.md`（+ `skills/` 同步）
- `tradingagents-analysis/references/prompts/README.md`（+ `skills/` 同步）
- `install.mjs`
- `CHANGELOG.md`（追加 1.3.2 条目）
- `package.json`（version bump 1.3.1 → 1.3.2）
