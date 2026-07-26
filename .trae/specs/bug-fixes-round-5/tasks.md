# Round 5 — Tasks

**Spec**: `spec.md`
**Branch**: `fix/round5-bugs` (stacked on `fix/round4-bugs`)

## Tasks

### Task 1: 9 个非 CN prompt 补 `{get_language_instruction()}` [BUG R5-1, HIGH]
- **Priority**: high
- **Files**: 9 个 prompt 文件（根 + skills 两份）:
  `market_analyst.md`, `news_analyst.md`, `fundamentals_analyst.md`, `bull_researcher.md`, `bear_researcher.md`, `research_manager.md`, `aggressive_risk.md`, `conservative_risk.md`, `neutral_risk.md`
- **Description**:
  1. 每个文件 prompt body 末尾（closing ``` 前）追加 `{get_language_instruction()}`，参照 `sentiment_analyst.md:64` / `portfolio_manager.md:35` 的放置。
  2. 每个文件 front-matter "Template variables" 行加入 `{get_language_instruction()}`。
- **Test**: 12 个非 CN prompt 文件全部含 `{get_language_instruction()}`；2 个 CN 文件不含。

### Task 2: 3 个脚本入口补 None 守卫 [BUG R5-2, HIGH]
- **Priority**: high
- **Files**: `tradingagents-analysis/scripts/fetch_news.py`, `fetch_stock_data.py`, `fetch_sentiment.py` (+ skills copy)
- **Description**: 在 `fetch_news` / `fetch_stock_data` / `fetch_sentiment` 入口加 `isinstance(symbol, str)` + `strip()` + 空串守卫，返回错误字符串。镜像 round-4 `fetch_fundamentals` 修复模式。
- **Test**: `fetch_news(None)` / `fetch_stock_data(None,...)` / `fetch_sentiment(None)` 返回错误串不抛异常。

### Task 3: fetch_stock_df 补 None 守卫 [BUG R5-12, LOW]
- **Priority**: low
- **Depends on**: Task 2（同文件）
- **Files**: `tradingagents-analysis/scripts/fetch_stock_data.py` (+ skills copy)
- **Description**: `fetch_stock_df` 入口加同 Task 2 的 None 守卫。
- **Test**: `fetch_stock_df(None, '2024-01-01', '2024-01-02')` 不抛异常。

### Task 4: `_fmt_num` 统一 NA 处理 [BUG R5-11, LOW]
- **Priority**: low
- **Files**: `tradingagents-analysis/scripts/fetch_fundamentals.py` (+ skills copy)
- **Description**: `_fmt_num` 守卫从 `isinstance(v, float) and pd.isna(v)` 改为 `v is None or pd.isna(v)`，覆盖 None/np.nan/pd.NA，统一返回 "N/A"。
- **Test**: `_fmt_num(pd.NA)` / `_fmt_num(None)` / `_fmt_num(np.nan)` 全返回 'N/A'。

### Task 5: npm `prepublishOnly` 清理 __pycache__ [BUG R5-3, MEDIUM]
- **Priority**: medium
- **Files**: `package.json`
- **Description**: 加 `prepublishOnly` script，发布前清理两份 `scripts/__pycache__`。
- **Test**: `npm pack --dry-run` 输出不含 `.pyc` / `__pycache__`。

### Task 6: install.mjs `path.resolve` 修绝对路径 [BUG R5-4, MEDIUM]
- **Priority**: medium
- **Files**: `install.mjs`
- **Description**: L130 `path.join(scriptsDir, s)` → `path.resolve(scriptsDir, s)`。
- **Test**: `node install.mjs --dir ./relpath` 打印绝对路径。

### Task 7: install.mjs `mkdirSync` 移入 try/catch [BUG R5-6, MEDIUM]
- **Priority**: medium
- **Depends on**: Task 6（同文件）
- **Files**: `install.mjs`
- **Description**: L104 `fs.mkdirSync(parentDir, { recursive: true })` 移入 L107 既有 try/catch 块。
- **Test**: `node install.mjs --dir 'C:\bad:path'` 打印友好错误非裸堆栈。

### Task 8: install.mjs `--dir` + `--agent` 互斥 [BUG R5-14, LOW]
- **Priority**: low
- **Depends on**: Task 6、7（同文件）
- **Files**: `install.mjs`
- **Description**: L87 `if (args.dir)` 前加 `if (args.dir && args.agent) fail('不能同时指定 --dir 和 --agent')`。
- **Test**: `node install.mjs --dir foo --agent opencode` 退出码 1 + 友好错误。

### Task 9: README_CN `.SS`/`.SZ` 残留 [BUG R5-5, MEDIUM]
- **Priority**: medium
- **Files**: `README_CN.md`
- **Description**: L129 改为 "脚本内部根据 6 位代码前缀自动判断交易所（6 开头 → 上海 .SS；0/3 开头 → 深圳 .SZ），用户只需提供 6 位纯数字"。
- **Test**: README_CN L129 不含 `.SS 后缀` 字样。

### Task 10: README 相对链接 404 [BUG R5-8, LOW]
- **Priority**: low
- **Files**: `README.md`, `README_CN.md`
- **Description**: `references/data-sources.md` → `tradingagents-analysis/references/data-sources.md`。
- **Test**: grep `](references/data-sources.md)` 无结果。

### Task 11: prompts/README `{investment_plan}` 流向 [BUG R5-9, LOW]
- **Priority**: low
- **Files**: `tradingagents-analysis/references/prompts/README.md` (+ skills copy)
- **Description**: L104 删除 "+ Risk Debate input"。
- **Test**: L104 不含 "Risk Debate input"。

### Task 12: prompts/README CN prompt 归因 [BUG R5-10, LOW]
- **Priority**: low
- **Files**: `tradingagents-analysis/references/prompts/README.md` (+ skills copy)
- **Description**: L125 / L131 标题下各加一行说明 CN 对应文件不引用这些 ghost tools。
- **Test**: README 含 CN 说明。

### Task 13: CHANGELOG "9 ghost tools" → 11 [BUG R5-7, LOW]
- **Priority**: low
- **Files**: `CHANGELOG.md`
- **Description**: L15 "9 个 ghost tools" → "11 个 ghost tools"。
- **Test**: CHANGELOG L15 含 "11 个 ghost tools"。

### Task 14: .npmignore 补 node_modules + CLAUDE.md [BUG R5-13, LOW]
- **Priority**: low
- **Files**: `.npmignore`
- **Description**: 加 `node_modules/` 和 `CLAUDE.md` 两行。
- **Test**: .npmignore 含两行。

### Task 15: 版本号 + CHANGELOG
- **Priority**: medium
- **Depends on**: Task 1-14
- **Files**: `package.json`, `CHANGELOG.md`
- **Description**: 版本 1.3.4 → 1.3.5；CHANGELOG 新增 `## [1.3.5]` 章节列 14 个修复。
- **Test**: package.json version == 1.3.5；CHANGELOG 含 `## [1.3.5]`。

### Task 16: 双拷贝同步最终验证
- **Priority**: high
- **Depends on**: Task 1-14
- **Description**: 清理 `__pycache__` 后 `git diff --no-index --quiet tradingagents-analysis skills/tradingagents-analysis` 退出码 0。
- **Test**: diff 退出码 0。

### Task 17: 验证套件 + PR
- **Priority**: high
- **Depends on**: Task 1-16
- **Description**: 扩展 round-4 验证脚本为 round-5（新增 14 bug 检查 + 回归），运行通过后推送并创建 PR #6（base: fix/round4-bugs）。
- **Test**: round-4 回归 + round-5 新增检查全部通过。

## Status

待实施。
