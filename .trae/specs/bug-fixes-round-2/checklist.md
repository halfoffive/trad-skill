# 第二轮 Bug 修复 - 验证 Checklist

## Python 脚本修复

- [ ] Checkpoint 1: `fetch_fundamentals.py` `_yoy()` 取 `iloc[0]` vs `iloc[1]`（最近一年同比），不再取 `iloc[-2]` / `iloc[-1]`
- [ ] Checkpoint 2: `fetch_fundamentals.py` `_yoy()` docstring 改为「计算最近一年的同比变化」
- [ ] Checkpoint 3: `fetch_fundamentals.py` 中 `df.to_markdown(index=False)` 全部替换为 `df.to_string(index=False)` 或纯 pandas 实现
- [ ] Checkpoint 4: `fetch_sentiment.py` 中 `df.to_markdown(index=False)` 全部替换
- [ ] Checkpoint 5: `fetch_stock_data.py` `build_compact_report` 失败路径不再重复调用 `fetch_stock_data`
- [ ] Checkpoint 6: 4 个脚本中无 `import sys`
- [ ] Checkpoint 7: 4 个脚本通过 `uv run python -c "import ast; ast.parse(open(f, encoding='utf-8').read())"`

## install.mjs 修复

- [ ] Checkpoint 8: `fs.cpSync` 加 `filter` 排除 `__pycache__`
- [ ] Checkpoint 9: `--dir` 缺值时 `fail()`，打印 `--dir 需要一个路径参数`
- [ ] Checkpoint 10: `--agent` 缺值时 `fail()`，打印 `--agent 需要一个名称（claude|agents|opencode）`
- [ ] Checkpoint 11: 支持 `--dir=PATH` / `--agent=NAME` 等号语法
- [ ] Checkpoint 12: `--dir ~/foo` 中 `~` 展开为 `os.homedir()`
- [ ] Checkpoint 13: 未知参数（如 `--foo`）触发 `fail()`
- [ ] Checkpoint 14: `cpSync` / `rmSync` 包 `try/catch`，失败走 `fail()`
- [ ] Checkpoint 15: `node --check install.mjs` 通过

## npm 打包

- [ ] Checkpoint 16: `.npmignore` 文件存在，包含 `**/__pycache__/`、`*.pyc`、`*.pyo`、`.omo/`、`.codegraph/`、`.trae/`
- [ ] Checkpoint 17: `npm pack --dry-run` 输出不含 `__pycache__` 或 `.pyc`

## SKILL.md 修复

- [ ] Checkpoint 18: A 股检测规则改为「6 位纯数字（如 600519、000858）」，不再有 `.SS`/`.SZ` 后缀 → A 股的语义
- [ ] Checkpoint 19: spawn 模板用 `{script_name}` / `{script_args}` 占位符
- [ ] Checkpoint 20: 模板注释明确「按 §6 替换脚本名 **和** 参数」
- [ ] Checkpoint 21: §2 或 §6 定义 `--start` / `--end` 默认窗口（trade date 前 1 年到当天）
- [ ] Checkpoint 22: §3 或 §4 加 CN prompt 切换说明（A股/港股时用 `china_market_analyst.md` / `cn_news_analyst.md`）
- [ ] Checkpoint 23: §3 语法 `Stages 1` → `Stage 1`
- [ ] Checkpoint 24: §6 表格 `fetch_news.py` 描述不含 "macro"
- [ ] Checkpoint 25: §6 表格 `fetch_sentiment.py` 描述不含 "headline analysis"

## README 修复

- [ ] Checkpoint 26: `README.md` 和 `README_CN.md` 中 `--agent opencode` 注释为 `~/.config/opencode/skills`
- [ ] Checkpoint 27: `README.md` 和 `README_CN.md` 可用安装位置列表含 `~/.config/opencode/skills/tradingagents-analysis`
- [ ] Checkpoint 28: `README_CN.md` A 股数据源优先级为 `AKShare → yfinance`，不含 `Tushare → AKShare → Baostock`
- [ ] Checkpoint 29: `README.md` 和 `README_CN.md` Project Structure tree 含 `skills/` 目录
- [ ] Checkpoint 30: Project Structure tree 不存在连续两个 `└──`
- [ ] Checkpoint 31: README 中 A 股检测规则描述与 SKILL.md 一致（6 位纯数字）
- [ ] Checkpoint 32: README 中 `fetch_news.py` 描述不含 "macro"

## references 修复

- [ ] Checkpoint 33: `data-sources.md` A 股降级链为 `AKShare → yfinance`，不含 MongoDB/Tushare/Baostock/TDX
- [ ] Checkpoint 34: `data-sources.md` 美股降级链为 `yfinance`，不含 Alpha Vantage
- [ ] Checkpoint 35: `data-sources.md` 美股新闻降级链为 `yfinance + Google News`，不含 Alpha Vantage News
- [ ] Checkpoint 36: `data-sources.md` Configuration 章节注明属原始框架，本 skill 不读取
- [ ] Checkpoint 37: `prompts/README.md` 不含开发机绝对路径 `D:\niaod\...`
- [ ] Checkpoint 38: `prompts/README.md` 指标列表含 MFI
- [ ] Checkpoint 39: `prompts/README.md` China Market Analyst 描述为 `akshare data`，不再说 `Tushare data`
- [ ] Checkpoint 40: `prompts/README.md` 5 阶段图注明 `Decision = Research Manager + Trader`（或改为 6 阶段）

## AGENTS.md / .gitignore / .gitattributes

- [ ] Checkpoint 41: `AGENTS.md` Gotchas 含 `.trae/specs/` 说明
- [ ] Checkpoint 42: `.gitignore` 含 `.codegraph/`
- [ ] Checkpoint 43: `.gitattributes` 文件存在，包含 `* text=auto eol=lf`

## 版本与 CHANGELOG

- [ ] Checkpoint 44: `package.json` `version` 为 `1.3.1`
- [ ] Checkpoint 45: `CHANGELOG.md` 含 `[1.3.1] - 2026-07-25` 条目
- [ ] Checkpoint 46: CHANGELOG 1.3.1 Fixed 列表覆盖第一轮 5 项 + 第二轮所有修复

## 双目录同步

- [ ] Checkpoint 47: `git diff --no-index tradingagents-analysis skills/tradingagents-analysis` 输出为空
- [ ] Checkpoint 48: `fetch_news.py` 两份拷贝行尾符一致

## PR 与流程

- [ ] Checkpoint 49: 新分支 `fix/round2-bugs` 已创建并推送
- [ ] Checkpoint 50: PR 已开到 `main`，PR body 列出本轮所有修复
- [ ] Checkpoint 51: 修复未引入新依赖（仍为 yfinance/akshare/requests/pandas）
- [ ] Checkpoint 52: 代码风格保持一致（中文注释、函数式风格、无 class 定义）
