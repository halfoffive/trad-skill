# Round 6 — Bug Fixes Spec

**Branch**: `fix/round6-bugs` (stacked on `fix/round5-bugs`)
**Date**: 2026-07-26
**Status**: ✅ COMPLETE — 30/30 bugs fixed (24 code/doc changes + 6 documented as limitations); 32/32 verify_round6.py checks pass; PR #7 open.
**Review method**: 3 parallel general-purpose sub-agents (Python scripts / Prompts+SKILL / Docs+Installer), each read-only, with cross-validation against source repos `../TradingAgents` and `../TradingAgents-CN`.

## Summary

30 confirmed bugs: **0 CRITICAL, 4 HIGH, 8 MEDIUM, 18 LOW**.

Findings consolidated from three sub-agent reports. Bug IDs renumbered R6-1 ~ R6-30 for unified tracking. Source-of-finding tagged per bug (PY=Python scripts agent, PR=Prompts+SKILL agent, DI=Docs+Installer agent).

Of the 30:
- **24 will be fixed** (code or doc changes)
- **6 will be documented as known limitations** (verbatim constraints or low-risk edge cases where code change is risky/out-of-scope): R6-5, R6-13, R6-16, R6-21, R6-23, R6-24

---

## Confirmed BUGs

### HIGH (4)

#### BUG R6-1 [HIGH] — `fetch_us_fundamentals` ticker 作用域 bug，公司概况失败静默吞掉三大报表 [PY]

- **Files**: `tradingagents-analysis/scripts/fetch_fundamentals.py:155-189` (+ skills copy)
- **Problem**: `ticker = yf.Ticker(symbol)` 在 try 块内，若抛异常（网络瞬断/rate limit/无效 symbol），`ticker` 永不赋值。后续三大报表 try 块引用 `ticker.financials` / `ticker.balance_sheet` / `ticker.cashflow` 全部抛 `NameError: name 'ticker' is not defined`，被各自 except 捕获后置 None。最终输出"## 关键财务指标\n\n> 无数据\n"，掩盖真实失败原因（公司概况网络瞬断被误表现为"报表无数据"）。
- **Verification**: 本地 mock `yf.Ticker` 抛异常，确认三个 try 块均抛 NameError。
- **Fix**: 函数顶部预声明 `ticker = None`，三大报表 try 块前加 `if ticker is None: financials = balance = cashflow = None` 短路；或将 ticker 创建移出 info try 块单独 try。

#### BUG R6-2 [HIGH] — `fetch_yfinance_news` 的 `days` 参数完全未生效 [PY]

- **Files**: `tradingagents-analysis/scripts/fetch_news.py:66-108` (+ skills copy)
- **Problem**: 函数签名 `days: int = 7`，docstring 说"获取最近多少天的新闻"，header 显示"最近 {days} 天"，但函数体内 `days` 仅出现在签名/docstring/header 字符串，**没有任何过滤逻辑**。`yf.Ticker(symbol).news` 返回 yfinance 默认新闻流，`for item in news_list[:limit]` 只按条数截断。用户传 `--days 1` 或 `--days 30` 得到同一批新闻，header 欺骗 LLM。
- **Verification**: `rg "days" fetch_news.py` 确认 days 仅在字符串中出现。
- **Fix**: 循环内对每条新闻的 `publishTime`/`providerPublishTime` 字段做 `>= now - timedelta(days=days)` 过滤；若 yfinance 返回结构不含时间字段，则在 docstring 与 header 中明确"days 仅用于显示"。

#### BUG R6-3 [HIGH] — `{target_label}` / `{asset_label}` / `{fundamentals_label}` 替换规则与源仓库不符 [PR]

- **Files**: `tradingagents-analysis/references/prompts/README.md:69-72`, `tradingagents-analysis/SKILL.md:163` (+ skills copy)
- **Problem**: 源仓库 `TradingAgents/tradingagents/agents/researchers/bull_researcher.py:20-25` 和 `news_analyst.py:17`：
  - `{target_label}` → `"stock"` 或 `"asset"`（按 asset_type）
  - `{asset_label}` → `"company"` 或 `"asset"`
  - `{fundamentals_label}` → `"Company fundamentals report"` 或 `"Asset fundamentals report (may be unavailable for crypto)"`

  trad-skill README 和 SKILL.md 却声称前两者替换为 ticker symbol、第三个替换为字面 `Fundamentals`。主 agent 按此替换后，bull_researcher.md 的 "advocating for investing in the {target_label}" 变成 "advocating for investing in AAPL"（语法生硬且丢失 stock/asset 语义）。
- **Fix**: README L69-72 和 SKILL.md L163 Quick reference 把这三个变量的替换规则改为与源仓库一致。

#### BUG R6-4 [HIGH] — 6 个 prompt 的 front-matter "Template variables" 列了 body 中并不存在的变量 [PR]

- **Files**: `tradingagents-analysis/references/prompts/`:
  - `china_market_analyst.md:7` — 声称 `{tool_names}, {current_date}, {ticker}, {system_message}`，body 无任何变量
  - `cn_news_analyst.md:7` — 声称 5 个变量，body 无
  - `market_analyst.md:7` — 声称 4 个变量，body 只有 `{get_language_instruction()}`
  - `fundamentals_analyst.md:7` — 同上
  - `news_analyst.md:7` — 同上（body 还有 `{asset_label}`）
  - `sentiment_analyst.md:7` — 声称 `{current_date}, {instrument_context}`，body 两者皆缺
- **Problem**: 源仓库这些 prompt 走 ChatPromptTemplate 外层模板（含 `{tool_names}` 等），system_message 只是其中一段。trad-skill 抽取 system_message 到 prompt body，但 front-matter 仍把外层模板变量列为 "Template variables"。主 agent 读 front-matter 以为需替换，但 body 里没槽位 — 替换是 no-op。CN 两个 prompt 的 body 连一个变量都没有，front-matter 完全是误导。
- **Fix**: 把这 6 个文件的 front-matter "Template variables" 改为只列出 body 中实际出现的变量。CN 两个 prompt 写 "Template variables: (none — body is static text)"。同时在 README 加 Note 说明 `{tool_names}/{current_date}/{instrument_context}/{system_message}` 由 SKILL.md §4 spawn 模板在外层处理。

### MEDIUM (8)

#### BUG R6-5 [MEDIUM] — `compute_indicators` 的 RSI 用 ewm 简化实现，与标准 Wilder RSI 数值偏差约 1pp [PY]

- **Files**: `tradingagents-analysis/scripts/fetch_stock_data.py:100-101` (+ skills copy)
- **Problem**: pandas `ewm(adjust=False)` 递推种子是 `gain[0]`，标准 Wilder RSI 种子是 `mean(gain[1:15])`（前 14 期 SMA）。偏差约 1pp，接近 30/70 阈值时可能跨越导致超买/超卖判定翻转。注释写"RSI(14)：Wilder 平滑法"但实现是简化版。
- **Decision**: **文档化为已知限制**，不改代码（改 Wilder 标准实现需引入 O(n) 循环或复杂向量化，风险与收益不匹配；ewm 简化版是 pandas 社区常见近似）。在 `indicators.md` 和代码注释中标注偏差。
- **Fix**: 注释改为"RSI(14)：Wilder 平滑法的 pandas ewm 简化实现，与标准 Wilder（SMA 种子）偏差约 1pp，接近阈值时需结合其他指标交叉验证"。

#### BUG R6-6 [MEDIUM] — `fetch_cn_stock_data` / `fetch_hk_stock_data` 中 `start_date.replace` 在 try 块外，违反 never raises 契约 [PY]

- **Files**: `tradingagents-analysis/scripts/fetch_stock_data.py:278-280`（CN）, `331-333`（HK） (+ skills copy)
- **Problem**: `ak_start = start_date.replace("-", "")` 在 try 块外。若 `start_date` 为 None（sub-agent 误传、日期计算失败），抛 `AttributeError: 'NoneType' object has no attribute 'replace'`，未被捕获。fetch_stock_data 入口仅守卫 symbol，未守卫 start_date/end_date。
- **Verification**: `fetch_cn_stock_data('600519', None, '2024-01-01')` 确认抛 AttributeError。
- **Fix**: 在 fetch_cn_stock_data/fetch_hk_stock_data 顶部加 `if not isinstance(start_date, str) or not isinstance(end_date, str): return "错误: 日期参数无效"` 守卫。

#### BUG R6-7 [MEDIUM] — `compute_indicators` 的 RSI 在持续上涨时返回 NA 而非 100 [PY]

- **Files**: `tradingagents-analysis/scripts/fetch_stock_data.py:100-103` (+ skills copy)
- **Problem**: `avg_loss.replace(0, pd.NA)` 把 -0.0 也替换为 NA（因为 `-0.0 == 0` 为 True），rs=NA，rsi=NA。标准定义下持续上涨应 RSI=100（超买）。Market Analyst 拿到"RSI=N/A, 中性"会误判强势标的为无信号。
- **Fix**: 用 `rs = avg_gain / avg_loss`（不 replace 0），然后 `rsi = avg_gain.copy()`，`rsi[avg_loss == 0] = 100`，`rsi[avg_loss != 0] = 100 - 100 / (1 + rs[avg_loss != 0])`。

#### BUG R6-8 [MEDIUM] — `compute_indicators` 的 Bollinger Bands 用默认 ddof=1，与传统 ddof=0 不一致 [PY]

- **Files**: `tradingagents-analysis/scripts/fetch_stock_data.py:107` (+ skills copy)
- **Problem**: pandas `rolling().std()` 默认 `ddof=1`（样本标准差），传统布林带（StockCharts、TradingView）使用 `ddof=0`（总体标准差）。上下轨偏宽约 2.6%，可能导致 `px >= boll_ub` 在边界情况误判为"突破上轨"。
- **Fix**: 改为 `close.rolling(20).std(ddof=0)`。

#### BUG R6-9 [MEDIUM] — SKILL.md §4 "Re-injection discipline" 指示 Stage 6 绑定 4 份 full report，但 portfolio_manager.md body 没有对应变量 [PR]

- **Files**: `tradingagents-analysis/SKILL.md:183`, `tradingagents-analysis/references/prompts/portfolio_manager.md:7,12-36` (+ skills copy)
- **Problem**: SKILL.md §4 指示主 agent 在 Stage 6 把 4 份 analyst 全文绑定到模板变量，但 portfolio_manager.md（与源仓库 verbatim 一致）根本没有 `{market_research_report}` / `{sentiment_report}` / `{news_report}` / `{fundamentals_report}` 这 4 个变量槽位。"bind the full reports" 这条指令在当前 prompt 结构下无从执行。
- **Fix**: 把 SKILL.md §4 Stage 6 的措辞改为 "append the full reports as out-of-template context (e.g., prepend them to the prompt as a '## Analyst Reports' section)"。

#### BUG R6-10 [MEDIUM] — SKILL.md §4 CN 市场替换描述把"2 个"写成"3 个"，且把 Bull/Bear Researcher 错列为分析师 [PR]

- **Files**: `tradingagents-analysis/SKILL.md:130` (+ skills copy)
- **Problem**: "其余 3 个分析师（Sentiment / Fundamentals / Bull-Bear-Researcher 等）保持不变" — Stage 1 总共只有 4 个分析师，替换 2 个（Market、News）后剩 2 个（Sentiment、Fundamentals），不是 3 个。且 Bull-Bear-Researcher 是 Stage 2 角色，不属于 Stage 1 分析师。
- **Fix**: 改为"其余 2 个分析师（Sentiment / Fundamentals）保持不变；Stage 2 及之后的 researcher/manager/risk debator 等角色不受 market 影响"。

#### BUG R6-11 [MEDIUM] — `{instrument_context}` 替换文本与源仓库语义差异显著，未在 README 标注为偏差 [PR]

- **Files**: `tradingagents-analysis/references/prompts/README.md:86` (+ skills copy)
- **Problem**: 源仓库 `build_instrument_context` 是一段强调"ticker 保全"和"身份锚定"的段落（防 hallucination），trad-skill 改成紧凑单行 `Market: ...; Ticker: ...; Trade date: ...`，丢了反幻觉措辞。
- **Decision**: **文档化为已知限制**（采用紧凑单行是为 token 效率，刻意取舍）。
- **Fix**: README L86 加 Note："trad-skill uses a compact one-liner instead of the source's full paragraph for token efficiency; the source's anti-hallucination wording is dropped — agents should rely on SKILL.md §2 ticker confirmation as the anti-hallucination gate."

#### BUG R6-12 [MEDIUM] — README 声称 "30 unique variables"，但其中 3 个在所有 prompt body 中都不存在 [PR]

- **Files**: `tradingagents-analysis/references/prompts/README.md:60,62,78,88-90` (+ skills copy)
- **Problem**: `{current_date}`, `{tool_names}`, `{system_message}` 只存在于源仓库外层 ChatPromptTemplate，trad-skill 抽取的 system_message body 里没有。但 front-matter 和 README 都把它们列为 prompt 内变量。结合 R6-4，主 agent 可能困惑"为什么替换了但 prompt 没变化"。
- **Fix**: README L62 加注："Of these 30, 3 (`{current_date}`, `{tool_names}`, `{system_message}`) appear only in the source's outer ChatPromptTemplate, not in the extracted system_message bodies of trad-skill's prompt files; their substitution is a no-op but documented for completeness."

### LOW (18)

#### BUG R6-13 [LOW] — `fetch_news.py` 顶部 `from datetime import datetime, timedelta` 是死代码 [PY]

- **Files**: `tradingagents-analysis/scripts/fetch_news.py:23` (+ skills copy)
- **Problem**: `rg "datetime|timedelta"` 仅匹配 import 行，函数体内无引用。但 R6-2 修复 days 过滤后会用到 `timedelta`，届时该 import 不再是死代码。
- **Fix**: R6-2 修复时若引入 `timedelta` 过滤则保留 import；否则删除。

#### BUG R6-14 [LOW] — `compute_indicators` 的 `_val` 对 inf 返回字符串 "inf" 而非 "N/A" [PY]

- **Files**: `tradingagents-analysis/scripts/fetch_stock_data.py:140-142` (+ skills copy)
- **Problem**: `pd.notna(float('inf'))` 返回 True，`round(float('inf'), 4)` 返回 `inf`。VWMA 和 MFI 在分母为 0 时可能产生 inf。
- **Fix**: 加 `if not pd.notna(v) or not np.isfinite(v): return "N/A"`。需 `import numpy as np` 或用 `math.isfinite`。

#### BUG R6-15 [LOW] — `fetch_stocktwits` 的 docstring 说 limit 默认 30，实际默认 15 [PY]

- **Files**: `tradingagents-analysis/scripts/fetch_sentiment.py:38-39` (+ skills copy)
- **Problem**: `DEFAULT_SENTIMENT_LIMIT=15`，但 docstring 写"默认 30"。
- **Fix**: docstring 改为"默认 15"。

#### BUG R6-16 [LOW] — `fetch_news` / `fetch_sentiment` 的 `limit` / `days` 参数未钳制负数 [PY]

- **Files**: `fetch_news.py:91,148,196,239`, `fetch_sentiment.py:69,142,178` (+ skills copy)
- **Problem**: `--limit -1` 时 `news_list[:-1]` 返回除最后一条外所有条目，违背降本设计。R5 已修复 fetch_stock_data 的 `--tail` 负数钳制，但同模式未同步到 news/sentiment。
- **Fix**: 各入口函数顶部加 `limit = max(0, int(limit))` 和 `days = max(1, int(days))`。

#### BUG R6-17 [LOW] — `compute_stats` 注释说"日对数收益"，实际用 `pct_change`（百分比收益）[PY]

- **Files**: `tradingagents-analysis/scripts/fetch_stock_data.py:208-210` (+ skills copy)
- **Problem**: 注释 `# 日对数收益 → 年化波动率（252 交易日）` 与代码 `close.pct_change()` 不一致。
- **Fix**: 改注释为"日百分比收益 → 年化波动率"。

#### BUG R6-18 [LOW] — `compute_indicators` 的 MFI 在所有 tp 持平时返回 NA 而非 50 [PY]

- **Files**: `tradingagents-analysis/scripts/fetch_stock_data.py:127-134` (+ skills copy)
- **Problem**: `pos_sum=0, neg_sum=0` 时 `mfr=NA, mfi=NA`。标准定义 MFI=50（中性）。
- **Fix**: 同 R6-7 RSI 的 0/0 处理，对 `pos_sum==0 & neg_sum==0` 分支返回 50。

#### BUG R6-19 [LOW] — `fetch_reddit_sentiment` / `fetch_stocktwits` 中 symbol 未 URL 编码 [PY]

- **Files**: `tradingagents-analysis/scripts/fetch_sentiment.py:45,140-143` (+ skills copy)
- **Problem**: symbol 含 `&`、`=`、`/`、`?` 等特殊字符时 URL 被破坏。常见股票代码不含这些，低概率。
- **Decision**: **文档化为已知限制**（常见股票代码不含特殊字符，Reddit/StockTwits API 对 `.`/`-` 容忍）。
- **Fix**: 代码注释加说明"symbol 未 URL 编码；常见股票代码（含 `.`/`-`）经实测可被 Reddit/StockTwits API 接受"。

#### BUG R6-20 [LOW] — `fetch_yfinance_news` 中 `content.get` 在 content 非 dict 时抛 AttributeError 中止整个循环 [PY]

- **Files**: `tradingagents-analysis/scripts/fetch_news.py:93-99` (+ skills copy)
- **Problem**: yfinance API 变更返回结构时，一条坏数据会中止整个循环，外层 try/except 捕获后返回错误，丢弃其余有效新闻。
- **Fix**: 循环内用 `try/except` 单独包裹每条 item 解析，`continue` 跳过坏数据。

#### BUG R6-21 [LOW] — `build_compact_report` 中 `tail=None` 抛 TypeError 不被捕获 [PY]

- **Files**: `tradingagents-analysis/scripts/fetch_stock_data.py:441-442` (+ skills copy)
- **Problem**: `tail = max(0, int(tail))` 在 tail=None 时 `int(None)` 抛 TypeError。CLI 路径 argparse type=int 不会产生 None，仅直接调用时触发。
- **Fix**: `tail = max(0, int(tail)) if tail is not None else 0`。

#### BUG R6-22 [LOW] — `fetch_cn_fundamentals` / `fetch_cn_stock_data` 的 yfinance 降级用 `symbol.startswith("6")` 判断交易所，未覆盖北交所（8 开头）和 B 股（9 开头）[PY]

- **Files**: `fetch_fundamentals.py:268-271`, `fetch_stock_data.py:301-304` (+ skills copy)
- **Problem**: 8 开头北交所代码（如 830799）或 9 开头 B 股代码（如 900901）被错误加 .SZ 后缀，yfinance 找不到数据。北交所流动性低、概率小。
- **Decision**: **文档化为已知限制**（北交所/B 股流动性低，yfinance 本身覆盖有限；AKShare 优先路径已覆盖）。
- **Fix**: 代码注释加说明"yfinance 降级仅支持 6/0/3 开头的沪深 A 股；北交所（8 开头）和 B 股（9 开头）请依赖 AKShare 优先路径"。

#### BUG R6-23 [LOW] — `market_analyst.md` 的指标列表不含 MFI，但 indicators.md / SKILL.md §3 / fetch_stock_data.py 都把 MFI 列为预计算指标 [PR]

- **Files**: `tradingagents-analysis/references/prompts/market_analyst.md:33-34`, `indicators.md:84-87`, `SKILL.md:83`, `fetch_stock_data.py:126-134,184` (+ skills copy)
- **Problem**: 源仓库 `market_analyst.py:46-47` 的原貌就不含 MFI — trad-skill verbatim 继承了这一不一致。脚本输出会塞给 analyst MFI 值，但 prompt 没告诉它这是什么。
- **Decision**: **文档化**（不改 verbatim prompt）。
- **Fix**: 在 `indicators.md` 开头或 SKILL.md §6 加注："Note: 源仓库 market_analyst.py 的指标列表未列 MFI，但 fetch_stock_data.py 会预计算 MFI；Market Analyst 应把脚本输出的 MFI 行作为补充指标解读。"

#### BUG R6-24 [LOW] — 多数非 CN prompt 在 `{get_language_instruction()}` 前多了一个空行 [PR]

- **Files**: `market_analyst.md:41-42`, `news_analyst.md:13-14`, 等 9 个文件 (+ skills copy)
- **Problem**: 源仓库 `"""...easy to read."""` 后直接 `+ get_language_instruction()` 无换行，trad-skill 加了空行。属轻微 whitespace 偏差。
- **Decision**: **文档化**（whitespace 美化，对 LLM 几乎无影响；删除空行风险大于收益）。
- **Fix**: README 加注 "whitespace before `{get_language_instruction()}` normalized to one blank line for readability"。

#### BUG R6-25 [LOW] — `trader.md` 的 2-block 结构（System Message + User Message）未在 SKILL.md §5 Stage 4 说明 [PR]

- **Files**: `tradingagents-analysis/references/prompts/trader.md:9-23`, `SKILL.md:148-159,211-216` (+ skills copy)
- **Problem**: trader.md 有 `## System Message` 和 `## User Message` 两个独立代码块。SKILL.md §5 Stage 4 只说 "Prompt: references/prompts/trader.md"，没告诉主 agent 这是 2-block 结构。
- **Fix**: SKILL.md §5 Stage 4 加一行 Note："trader.md has separate System Message and User Message blocks; construct the LLM call with both roles."

#### BUG R6-26 [LOW] — `{get_language_instruction()}` 在 English 场景下的替换值与源仓库行为不一致 [PR]

- **Files**: `tradingagents-analysis/references/prompts/README.md:87` (+ skills copy)
- **Problem**: 源仓库 English 时返回空字符串（不注入任何指令），非 English 时返回 ` Write your entire response in {lang}.`。trad-skill README 声称总是替换为 `Respond in <language> per output_language.`。
- **Fix**: README L87 改为：English 时 → 空字符串；非 English 时 → `Write your entire response in {lang}.`（贴近源行为）。

#### BUG R6-27 [LOW] — SKILL.md §6 表格未列出 `--indicators` / `--no-indicators` / `--no-stats` 三个 argparse flag [PR]

- **Files**: `tradingagents-analysis/SKILL.md:237-242` (+ skills copy)
- **Problem**: §6 表格 invocation 示例只展示 `--stats`，描述提到 `--raw`，但 `--indicators` / `--no-indicators` / `--no-stats` 完全未提及。
- **Fix**: §6 表格 fetch_stock_data.py 行的描述补一句："Flags: `--indicators`/`--no-indicators` (default on), `--stats`/`--no-stats` (default off), `--raw` (legacy full CSV), `--tail N` (default 30)."

#### BUG R6-28 [LOW] — `install.mjs` L107/L126 `destDir` 仍用 `path.join`，`--dir` 相对路径时显示相对路径 [DI]

- **Files**: `install.mjs:107, 126`
- **Problem**: round-5 修了 L139 的 `scriptsDir` 用 `path.resolve`，但漏了 L107/L126 的 `destDir` 仍用 `path.join`。`--dir ./foo` 时 L126 显示 `foo\tradingagents-analysis`（相对路径），与 L139 的绝对路径输出不一致。
- **Fix**: L107 把 `path.join(parentDir, SKILL_NAME)` 改为 `path.resolve(parentDir, SKILL_NAME)`。

#### BUG R6-29 [LOW] — `README.md` L144 配置表 `market` 默认值 `...` 应为 `—` [DI]

- **Files**: `README.md:144`
- **Problem**: README.md L144 用 `...`（看似占位符未填），SKILL.md L285 和 README_CN.md L187 都用 `—`（em dash 表示"无默认值，自动检测"）。
- **Fix**: README.md L144 把 `...` 改为 `—`。

#### BUG R6-30 [LOW] — `README_CN.md` L136 港股"5位数字"与同文件 L29 4位数字示例矛盾 [DI]

- **Files**: `README_CN.md:29, 136`
- **Problem**: L29 用 4 位数字示例 `0700.HK, 9988.HK`，L136 说"5位数字"用 5 位示例 `00700.HK`。同文件内部矛盾。脚本 `zfill(5)` 两种都接受。
- **Fix**: L136 改为"使用 4-5 位数字 + `.HK` 后缀（如 0700.HK 或 00700.HK）"。

---

## Out of scope / Documented as limitation

以下 6 项因 verbatim 约束、低风险边缘情况、或代码改动风险大于收益，**不修改代码**，仅加注释或文档说明：

- **R6-5** (RSI Wilder 偏差 ~1pp) — 代码注释标注简化实现
- **R6-11** ({instrument_context} 语义偏差) — README 加 Note（token 效率取舍）
- **R6-19** (symbol URL 编码) — 代码注释标注（常见代码不含特殊字符）
- **R6-22** (北交所/B 股前缀) — 代码注释标注（AKShare 优先路径覆盖）
- **R6-23** (market_analyst.md 不列 MFI) — indicators.md 加 Note（verbatim 不改 prompt）
- **R6-24** (whitespace 偏差) — README 加 Note（美化，对 LLM 无影响）

## Verification plan

- `verify_round6.py` — 逐 bug 验证脚本，目标 ≥30 项检查全过
- `uv run python -c "import ast; ast.parse(...)"` — 4 个脚本语法
- `node --check install.mjs` — 安装器语法
- `git diff --no-index --stat tradingagents-analysis skills/tradingagents-analysis` — 副本一致性（仅 __pycache__ 差异）
- 源仓库 grep 抽检 verbatim 合规

## Branch / PR

- Branch: `fix/round6-bugs` (base: `fix/round5-bugs`)
- PR: #7 (round6 → round5 base)
- Commits grouped: HIGH fixes → MEDIUM fixes → LOW fixes → spec/verify/changelog/version
