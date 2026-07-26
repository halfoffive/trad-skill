# Round 6 — Checklist

每项修复完成后打勾。验证脚本 `verify_round6.py` 须全过。

## HIGH (4)

- [ ] R6-1: `fetch_fundamentals.py` `fetch_us_fundamentals` 预声明 `ticker = None`，三大报表前 `if ticker is None` 短路
- [ ] R6-1: mock `yf.Ticker` 抛异常时，三大报表 try 块不抛 NameError，返回友好错误
- [ ] R6-2: `fetch_news.py` `fetch_yfinance_news` 循环内对 `publishTime`/`providerPublishTime` 做 `>= now - timedelta(days=days)` 过滤
- [ ] R6-2: header 仍显示"最近 {days} 天"，且与实际过滤一致
- [ ] R6-3: `prompts/README.md` L69-72 `{target_label}`/`{asset_label}`/`{fundamentals_label}` 替换规则改为源仓库语义
- [ ] R6-3: `SKILL.md` L163 Quick reference 同步
- [ ] R6-4: `china_market_analyst.md` front-matter → "(none — body is static text)"
- [ ] R6-4: `cn_news_analyst.md` front-matter → "(none — body is static text)"
- [ ] R6-4: `market_analyst.md` front-matter 只列 `{get_language_instruction()}`
- [ ] R6-4: `fundamentals_analyst.md` front-matter 只列 `{get_language_instruction()}`
- [ ] R6-4: `news_analyst.md` front-matter 列 `{asset_label}`, `{get_language_instruction()}`
- [ ] R6-4: `sentiment_analyst.md` front-matter 只列 body 实际变量
- [ ] R6-4: `prompts/README.md` 加 Note 说明外层模板变量

## MEDIUM (8)

- [ ] R6-5: `compute_indicators` RSI 注释标注"ewm 简化实现，偏差约 1pp"
- [ ] R6-6: `fetch_cn_stock_data` 顶部加日期守卫
- [ ] R6-6: `fetch_hk_stock_data` 顶部加日期守卫
- [ ] R6-6: `fetch_cn_stock_data('600519', None, '2024-01-01')` 返回错误串不抛异常
- [ ] R6-7: `compute_indicators` RSI `avg_loss == 0` 时返回 100
- [ ] R6-7: mock 持续上涨序列确认 RSI=100
- [ ] R6-8: `compute_indicators` Bollinger `.std(ddof=0)`
- [ ] R6-9: `SKILL.md` §4 Stage 6 措辞改为 "append as out-of-template context"
- [ ] R6-10: `SKILL.md` L130 CN swap "3 个" → "2 个"，删除 Bull-Bear-Researcher
- [ ] R6-11: `prompts/README.md` L86 加 instrument_context Note
- [ ] R6-12: `prompts/README.md` L62 加 3 phantom variables 注

## LOW (18)

- [ ] R6-13: `fetch_news.py` import 检查（R6-2 后 `timedelta` 已用，`datetime` 视情况）
- [ ] R6-14: `compute_indicators` `_val` inf → "N/A"
- [ ] R6-15: `fetch_stocktwits` docstring "默认 30" → "默认 15"
- [ ] R6-16: `fetch_news` / `fetch_sentiment` 各入口 limit/days 负数钳制
- [ ] R6-17: `compute_stats` 注释 "日对数收益" → "日百分比收益"
- [ ] R6-18: `compute_indicators` MFI `pos_sum==0 & neg_sum==0` → 50
- [ ] R6-19: `fetch_sentiment.py` symbol URL 编码注释
- [ ] R6-20: `fetch_yfinance_news` 循环内 try/except 包裹每条 item
- [ ] R6-21: `build_compact_report` `tail=None` 守卫
- [ ] R6-22: 北交所/B 股前缀注释
- [ ] R6-23: `indicators.md` 加 MFI Note
- [ ] R6-24: `prompts/README.md` 加 whitespace Note
- [ ] R6-25: `SKILL.md` §5 Stage 4 加 trader.md 2-block Note
- [ ] R6-26: `prompts/README.md` L87 English 时空字符串
- [ ] R6-27: `SKILL.md` §6 表格补 Flags 列表
- [ ] R6-28: `install.mjs` L107 `path.join` → `path.resolve`
- [ ] R6-28: `node install.mjs --dir ./foo` destDir 显示绝对路径
- [ ] R6-29: `README.md` L144 `...` → `—`
- [ ] R6-30: `README_CN.md` L136 "5位数字" → "4-5 位数字"

## 副本一致性

- [ ] `git diff --no-index --stat tradingagents-analysis skills/tradingagents-analysis` 仅 __pycache__ 差异

## 版本与文档

- [ ] `CHANGELOG.md` 加 `## [1.3.6] - 2026-07-26` 章节
- [ ] `package.json` version 1.3.5 → 1.3.6
- [ ] `verify_round6.py` 全部检查通过

## 语法

- [ ] `uv run python -c "import ast; ast.parse(open(f, encoding='utf-8').read())"` 4 个脚本全过
- [ ] `node --check install.mjs` 通过
