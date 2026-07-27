# Round 5 — Checklist

每项必须验证通过才能标记完成。

## R5-1 HIGH — 9 prompt 补 {get_language_instruction()}
- [ ] `market_analyst.md` prompt body 末尾含 `{get_language_instruction()}`
- [ ] `news_analyst.md` prompt body 末尾含 `{get_language_instruction()}`
- [ ] `fundamentals_analyst.md` prompt body 末尾含 `{get_language_instruction()}`
- [ ] `bull_researcher.md` prompt body 末尾含 `{get_language_instruction()}`
- [ ] `bear_researcher.md` prompt body 末尾含 `{get_language_instruction()}`
- [ ] `research_manager.md` prompt body 末尾含 `{get_language_instruction()}`
- [ ] `aggressive_risk.md` prompt body 末尾含 `{get_language_instruction()}`
- [ ] `conservative_risk.md` prompt body 末尾含 `{get_language_instruction()}`
- [ ] `neutral_risk.md` prompt body 末尾含 `{get_language_instruction()}`
- [ ] 9 个文件 front-matter "Template variables" 行含 `{get_language_instruction()}`
- [ ] 12 个非 CN prompt 文件全部含 `{get_language_instruction()}`（grep 验证）
- [ ] 2 个 CN prompt 文件不含 `{get_language_instruction()}`（保持现状）

## R5-2 HIGH — 3 脚本 None 守卫
- [ ] `fetch_news(None)` 返回错误串不抛异常
- [ ] `fetch_stock_data(None, '2024-01-01', '2024-01-02')` 返回错误串不抛异常
- [ ] `fetch_sentiment(None)` 返回错误串不抛异常
- [ ] `fetch_news("")` 返回错误串不抛异常
- [ ] `fetch_news("   ")` 返回错误串不抛异常
- [ ] `fetch_news("AAPL")` 正常工作（不回归）

## R5-3 MEDIUM — npm pack 无 .pyc
- [ ] `package.json` 含 `prepublishOnly` script
- [ ] `npm pack --dry-run` 输出不含 `.pyc`
- [ ] `npm pack --dry-run` 输出不含 `__pycache__`

## R5-4 MEDIUM — install.mjs 绝对路径
- [ ] `install.mjs` 用 `path.resolve`（非 `path.join`）处理 scriptsDir
- [ ] `node install.mjs --dir ./relpath`（在 %TEMP% 下）打印绝对路径

## R5-5 MEDIUM — README_CN .SS/.SZ
- [ ] README_CN L129 不含 `.SS 后缀` 字样
- [ ] README_CN L129 含 "6 位代码前缀" 或等价说明

## R5-6 MEDIUM — install.mjs mkdirSync
- [ ] `install.mjs` L104 `mkdirSync` 在 try/catch 内
- [ ] `node install.mjs --dir 'C:\bad:path'` 打印友好错误非裸堆栈

## R5-7 LOW — CHANGELOG 9 → 11
- [ ] CHANGELOG L15 含 "11 个 ghost tools"
- [ ] CHANGELOG L15 不含 "9 个 ghost tools"

## R5-8 LOW — README 相对链接
- [ ] `grep -r '](references/data-sources.md)' README.md README_CN.md` 无结果
- [ ] README.md / README_CN.md 含 `tradingagents-analysis/references/data-sources.md` 链接

## R5-9 LOW — {investment_plan} 流向
- [ ] `prompts/README.md` L104 不含 "Risk Debate input"
- [ ] `prompts/README.md` L104 含 "Trader input"

## R5-10 LOW — CN prompt 归因
- [ ] `prompts/README.md` Market Analyst tools 章节含 CN 说明
- [ ] `prompts/README.md` News Analyst tools 章节含 CN 说明

## R5-11 LOW — _fmt_num pd.NA
- [ ] `_fmt_num(pd.NA)` 返回 'N/A'
- [ ] `_fmt_num(None)` 返回 'N/A'
- [ ] `_fmt_num(np.nan)` 返回 'N/A'
- [ ] `_fmt_num(1.5)` 返回 '1.5'（不回归）

## R5-12 LOW — fetch_stock_df None
- [ ] `fetch_stock_df(None, '2024-01-01', '2024-01-02')` 不抛异常

## R5-13 LOW — .npmignore
- [ ] `.npmignore` 含 `node_modules/`
- [ ] `.npmignore` 含 `CLAUDE.md`

## R5-14 LOW — --dir + --agent 互斥
- [ ] `install.mjs` 含 `args.dir && args.agent` 检查
- [ ] `node install.mjs --dir foo --agent opencode` 退出码 1 + 友好错误

## 回归（round 1-4 检查点）
- [ ] `fetch_fundamentals("")` 返回错误串不抛异常
- [ ] `fetch_fundamentals(None)` 返回错误串不抛异常
- [ ] `compute_stats` 单行 DataFrame 输出 "N/A" 不输出 "nan%"
- [ ] `--no-stats` 参数存在且工作
- [ ] `--indicators` / `--no-indicators` 参数对称
- [ ] SKILL.md §6 示例日期为 2023-07-01 至 2024-06-30
- [ ] README.md / README_CN.md 示例日期与 SKILL.md 一致
- [ ] install.mjs `~` 展开工作
- [ ] install.mjs `--dir=PATH` 等号语法工作
- [ ] install.mjs 未知参数 fail 退出码 1
- [ ] install.mjs idempotent
- [ ] install.mjs `__pycache__` filter 工作
- [ ] 4 个脚本无 `class` 关键字
- [ ] 4 个脚本无 tabulate 残留
- [ ] akshare 软依赖 try/except 守卫
- [ ] A股函数 `ak is not None` 检查
- [ ] prompts/README "Template Variable Substitution" 章节含 30 个变量
- [ ] prompts/README "Tool-Name Override" 章节含 11 个 ghost tools

## 双拷贝同步
- [ ] `git diff --no-index --quiet tradingagents-analysis skills/tradingagents-analysis` 退出码 0（清理 __pycache__ 后）

## 版本
- [ ] `package.json` version == 1.3.5
- [ ] CHANGELOG 含 `## [1.3.5]`
