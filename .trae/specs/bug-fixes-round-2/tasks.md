# 第二轮 Bug 修复 - 任务清单

## 实现策略
- 新分支 `fix/round2-bugs`，基于 `main`
- 按任务组提交，每组 1 commit，提交后立即 `git push`
- 所有 Python/技能目录修改同时改 `tradingagents-analysis/` 和 `skills/tradingagents-analysis/` 两份
- 提交前验证：`git diff --no-index tradingagents-analysis skills/tradingagents-analysis`（应为空）+ `uv run python -c "import ast; ast.parse(...)"`

---

## Task 1: 修复 `fetch_fundamentals.py` 的 YoY 方向和 to_markdown 依赖
- **Priority**: high
- **Depends On**: None
- **Description**:
  - 修改 `_yoy()` 函数：把 `vals.iloc[-2]` / `vals.iloc[-1]` 改为 `vals.iloc[0]` / `vals.iloc[1]`（yfinance financials 列降序，最近年在前）
  - 同步修改 `_yoy()` docstring：从「计算序列最早可得的前后两年同比变化」改为「计算最近一年的同比变化（百分比）」
  - 把 A 股表格输出中的 `df.to_markdown(index=False)` 替换为 `df.to_string(index=False)`（约 line 226、245）
  - 同时修改 `tradingagents-analysis/scripts/fetch_fundamentals.py` 和 `skills/tradingagents-analysis/scripts/fetch_fundamentals.py`
- **Acceptance Criteria**: spec 中 MODIFIED Requirement「Python 脚本契约一致性」的两个 Scenario
- **Test Requirements**:
  - `programmatic` TR-1.1: `_yoy(pd.Series([100, 90, 80, 70]))` 返回 `+11.11%`（或 `"11.11"`），不是 `-12.5%`
  - `programmatic` TR-1.2: 在未安装 `tabulate` 的环境下调用 `fetch_cn_fundamentals("600519")`（akshare 可用时）返回的字符串不含 `Import tabulate failed`
  - `programmatic` TR-1.3: `uv run python -c "import ast; ast.parse(open('tradingagents-analysis/scripts/fetch_fundamentals.py', encoding='utf-8').read())"` 通过
  - `programmatic` TR-1.4: `git diff --no-index tradingagents-analysis/scripts/fetch_fundamentals.py skills/tradingagents-analysis/scripts/fetch_fundamentals.py` 输出为空

## Task 2: 修复 `fetch_sentiment.py` 的 to_markdown 依赖
- **Priority**: high
- **Depends On**: None
- **Description**:
  - 把 A 股表格输出中的 `df.to_markdown(index=False)` 替换为 `df.to_string(index=False)`（约 line 218、237）
  - 同时修改两份拷贝
- **Acceptance Criteria**: spec MODIFIED Requirement「Python 脚本契约一致性」Scenario「A 股基本面表格在未装 tabulate 时正常输出」
- **Test Requirements**:
  - `programmatic` TR-2.1: 在未安装 `tabulate` 的环境下，`fetch_cn_sentiment("600519")` 返回的字符串不含 `Import tabulate failed`
  - `programmatic` TR-2.2: ast.parse 通过
  - `programmatic` TR-2.3: 两份拷贝 diff 为空

## Task 3: 修复 `fetch_stock_data.py` 的重复网络请求 + 删除 4 脚本的 `import sys`
- **Priority**: medium
- **Depends On**: None
- **Description**:
  - 修改 `build_compact_report()`：让失败路径不再重复调用 `fetch_stock_data`（改为 `fetch_stock_df` 复用一次结果，或在 `build_compact_report` 内只调一次 `fetch_stock_data` 并自行解析 CSV）
  - 删除 4 个脚本中的 `import sys`（`fetch_stock_data.py:23`、`fetch_fundamentals.py:19`、`fetch_news.py:22`、`fetch_sentiment.py:13`）
  - 同时修改两份拷贝
- **Acceptance Criteria**: 代码清理 + 性能修复
- **Test Requirements**:
  - `programmatic` TR-3.1: `grep -n "import sys" tradingagents-analysis/scripts/*.py` 无输出
  - `programmatic` TR-3.2: ast.parse 通过 4 个脚本
  - `programmatic` TR-3.3: 两份拷贝 diff 为空
  - `programmatic` TR-3.4: 阅读 `build_compact_report` 确认 `fetch_stock_data` 在失败路径只被调用一次

## Task 4: 修复 `install.mjs` 的 cpSync 过滤、参数校验、~ 展开、try/catch
- **Priority**: high
- **Depends On**: None
- **Description**:
  - `fs.cpSync` 加 `filter: (src) => path.basename(src) !== '__pycache__'`
  - `parseArgs` 重构：支持 `--dir=PATH` / `--agent=NAME` 等号语法；`--dir` / `--agent` 缺值或值以 `--` 开头时 `fail()`；未知参数 `fail()`
  - `--dir` 路径展开 `~` 和 `~/`（用 `os.homedir()`）
  - `cpSync` / `rmSync` 包 `try/catch`，失败走 `fail()`
  - 文件: `install.mjs`
- **Acceptance Criteria**: spec ADDED Requirement「安装器参数校验与路径展开」全部 Scenario + ADDED Requirement「安装产物不含 Python 字节码」Scenario「cpSync 过滤」
- **Test Requirements**:
  - `programmatic` TR-4.1: `node -e "require('fs')"` 模拟 `--dir` 缺值，确认进程退出码非 0 且打印 `--dir 需要一个路径参数`
  - `programmatic` TR-4.2: 模拟 `--dir=/tmp/test-install`，确认装到 `/tmp/test-install/tradingagents-analysis/`，且目标目录不含 `__pycache__`
  - `programmatic` TR-4.3: 模拟 `--dir ~/test-install`（home 展开），确认不在当前目录创建 `~` 目录
  - `programmatic` TR-4.4: 模拟未知参数 `--foo`，确认 `fail()` 触发
  - `programmatic` TR-4.5: `node --check install.mjs` 通过

## Task 5: 新增 `.npmignore` 并校准 `package.json` files 字段
- **Priority**: high
- **Depends On**: None
- **Description**:
  - 新建 `.npmignore`：排除 `**/__pycache__/`、`*.pyc`、`*.pyo`、`.omo/`、`.codegraph/`、`.trae/`、`*.log`、`.vscode/`
  - `package.json` `files` 字段保持现状（已 allowlist），但 `.npmignore` 作为兜底
- **Acceptance Criteria**: spec ADDED Requirement「安装产物不含 Python 字节码」Scenario「npm pack 不含 pyc」
- **Test Requirements**:
  - `programmatic` TR-5.1: `npm pack --dry-run 2>&1` 输出不含 `__pycache__` 或 `.pyc`
  - `programmatic` TR-5.2: `.npmignore` 文件存在且包含上述条目

## Task 6: 修复 `SKILL.md`（spawn 模板、A股检测、CN prompt、{start}/{end}、语法、描述校准）
- **Priority**: high
- **Depends On**: None
- **Description**:
  - §2/§3 A 股检测规则：`.SS`/`.SZ` 后缀 → 改为「6 位纯数字（如 600519、000858）」
  - §4 spawn 模板：`python "{SCRIPTS_DIR}/fetch_stock_data.py" --symbol {ticker} --start {start} --end {end}` → `python "{SCRIPTS_DIR}/{script_name}" {script_args}`，注释明确「按 §6 替换脚本名 **和** 参数」
  - §2 或 §6 加 `--start`/`--end` 默认窗口指引（trade date 前 1 年到当天，至少 200 个交易日）
  - §3 或 §4 加 CN prompt 切换说明（A股/港股时用 `china_market_analyst.md` / `cn_news_analyst.md`）
  - §3 语法：`Stages 1 uses parallel sub-agents` → `Stage 1 uses parallel sub-agents`
  - §6 表格：`fetch_news.py` 描述去掉 "macro"，`fetch_sentiment.py` 描述去掉 "headline analysis"
  - 同时修改两份 SKILL.md
- **Acceptance Criteria**: spec ADDED Requirement「SKILL.md spawn 模板参数化」+ ADDED Requirement「A股/港股 CN prompt 接入」+ MODIFIED Requirement「文档与脚本行为一致」Scenario「A股检测规则」
- **Test Requirements**:
  - `programmatic` TR-6.1: `grep -n "\.SS\|\.SZ" tradingagents-analysis/SKILL.md` 输出不含「后缀 → A 股」语义的行
  - `programmatic` TR-6.2: `grep -n "script_name\|script_args" tradingagents-analysis/SKILL.md` 有命中
  - `programmatic` TR-6.3: `grep -n "china_market_analyst\|cn_news_analyst" tradingagents-analysis/SKILL.md` 有命中
  - `programmatic` TR-6.4: `grep -n "Stages 1" tradingagents-analysis/SKILL.md` 无输出
  - `programmatic` TR-6.5: 两份 SKILL.md diff 为空

## Task 7: 修复 `README.md` / `README_CN.md`（OpenCode 路径、A股优先级、macro news、project tree、检测规则）
- **Priority**: high
- **Depends On**: None
- **Description**:
  - `--agent opencode` 注释：`~/.opencode/skills` → `~/.config/opencode/skills`（两份 README）
  - 可用安装位置列表补 `~/.config/opencode/skills/tradingagents-analysis (OpenCode global)`（两份 README）
  - `README_CN.md` A 股数据源优先级：`Tushare → AKShare → Baostock` → `AKShare → yfinance`
  - `README.md` / `README_CN.md` Project Structure tree：补 `skills/` 目录行；修掉 `└── LICENSE` 后跟 `└── tradingagents-analysis/` 的重复（前者改 `├──`）
  - `README.md` / `README_CN.md` A 股检测规则描述与 SKILL.md 对齐（6 位纯数字）
  - `README.md` 项目结构描述里 `fetch_news.py | News data fetcher (company + macro)` 去掉 `+ macro`
- **Acceptance Criteria**: spec MODIFIED Requirement「文档与脚本行为一致」Scenario「OpenCode 安装路径」+「A股检测规则」
- **Test Requirements**:
  - `programmatic` TR-7.1: `grep -n "~/.opencode/skills" README.md README_CN.md` 无输出
  - `programmatic` TR-7.2: `grep -n "Tushare → AKShare → Baostock\|Tushare.*AKShare.*Baostock" README_CN.md` 无输出
  - `programmatic` TR-7.3: `grep -n "skills/tradingagents-analysis" README.md README_CN.md` 有命中（project tree 含 skills/）
  - `programmatic` TR-7.4: `grep -n "company + macro\|company and macro" README.md README_CN.md SKILL.md` 无输出
  - `human-judgement` TR-7.5: project tree 不存在连续两个 `└──`

## Task 8: 修复 `references/data-sources.md` 降级链与 Configuration 章节
- **Priority**: high
- **Depends On**: None
- **Description**:
  - A 股降级链：`MongoDB cache → Tushare → AKShare → Baostock → TDX` → `AKShare → yfinance`
  - 美股降级链：`yfinance → Alpha Vantage` → `yfinance`
  - 美股新闻降级链：`Yahoo Finance News → Alpha Vantage News → Google News` → `yfinance + Google News`
  - AKShare "Fallback chain" 行：删除或改为 `Uses: stock_zh_a_hist`
  - Configuration 章节：注明「此配置体系属于原始 TradingAgents 框架，本 skill 不读取」
  - 同时修改两份 data-sources.md
- **Acceptance Criteria**: spec MODIFIED Requirement「文档与脚本行为一致」Scenario「data-sources.md 降级链」
- **Test Requirements**:
  - `programmatic` TR-8.1: `grep -n "MongoDB\|Baostock\|TDX" tradingagents-analysis/references/data-sources.md` 无输出（或仅 Configuration 章节注明属原始框架）
  - `programmatic` TR-8.2: `grep -n "Alpha Vantage" tradingagents-analysis/references/data-sources.md` 无输出（或注明未实现）
  - `programmatic` TR-8.3: 两份 data-sources.md diff 为空

## Task 9: 修复 `references/prompts/README.md`（开发机路径、MFI、Tushare、5 阶段图注）
- **Priority**: medium
- **Depends On**: None
- **Description**:
  - 删除开发机绝对路径 `D:\niaod\RustroverProjects\trad\TradingAgents\` 和 `TradingAgents-CN\`，改为 GitHub 链接 `https://github.com/TauricResearch/TradingAgents` 和 `https://github.com/hsliuping/TradingAgents-CN`
  - 指标列表「up to 8 complementary indicators (SMA, EMA, MACD, RSI, Bollinger, ATR, VWMA)」末尾加 `MFI`
  - "China Market Analyst (CN): A-share/HK-specific analysis with Tushare data" → `with akshare data (Tushare referenced in prompt but not wired in scripts)`
  - 5 阶段图：在「Decision」下加注 `Decision = Research Manager + Trader`，或在概览改为 6 阶段
  - 同时修改两份 prompts/README.md
- **Acceptance Criteria**: 文档准确性
- **Test Requirements**:
  - `programmatic` TR-9.1: `grep -n "D:\\\\niaod\|D:/niaod" tradingagents-analysis/references/prompts/README.md` 无输出
  - `programmatic` TR-9.2: `grep -n "MFI" tradingagents-analysis/references/prompts/README.md` 有命中
  - `programmatic` TR-9.3: `grep -n "with Tushare data" tradingagents-analysis/references/prompts/README.md` 无输出（已改为 akshare data）
  - `programmatic` TR-9.4: 两份 prompts/README.md diff 为空

## Task 10: 修复 `AGENTS.md` Gotchas（补 `.trae/specs/` 说明）+ `.gitignore` 补 `.codegraph/`
- **Priority**: medium
- **Depends On**: None
- **Description**:
  - `AGENTS.md` Gotchas 章节加一行：`- .trae/specs/ 是 trae agent 的 spec 工作流状态（spec.md/tasks.md/checklist.md），已跟踪；与 .omo/ 不同，不要 gitignore。`
  - `.gitignore` 加 `.codegraph/`（防御性，不依赖嵌套 .gitignore）
- **Acceptance Criteria**: 文档完整性 + 防御性 gitignore
- **Test Requirements**:
  - `programmatic` TR-10.1: `grep -n ".trae/specs" AGENTS.md` 有命中
  - `programmatic` TR-10.2: `grep -n ".codegraph" .gitignore` 有命中

## Task 11: 新增 `.gitattributes` 统一行尾符 + 同步 `fetch_news.py` 两份拷贝
- **Priority**: medium
- **Depends On**: Task 1, Task 2, Task 3（先完成 Python 修改再统一行尾）
- **Description**:
  - 新建 `.gitattributes`：`* text=auto eol=lf`
  - 把 `skills/tradingagents-analysis/scripts/fetch_news.py` 的 CRLF 转为 LF（与根拷贝对齐）
  - 验证两份拷贝字节一致
- **Acceptance Criteria**: spec MODIFIED Requirement「两份技能目录字节同步」Scenario「diff 为空」
- **Test Requirements**:
  - `programmatic` TR-11.1: `.gitattributes` 存在且包含 `* text=auto eol=lf`
  - `programmatic` TR-11.2: `git diff --no-index tradingagents-analysis skills/tradingagents-analysis` 输出为空

## Task 12: bump version 到 1.3.1 + 更新 CHANGELOG
- **Priority**: high
- **Depends On**: Task 1-11 全部完成
- **Description**:
  - `package.json` `"version": "1.3.0"` → `"1.3.1"`
  - `CHANGELOG.md` 在 `[1.3.0]` 之上加 `[1.3.1] - 2026-07-25` 条目
  - Fixed 列表覆盖：第一轮 5 项（OpenCode 路径、`<unavailable>` 占位符、`_fmt_num` 类型、docstring 拼写、akshare 显式检查）+ 第二轮所有 Fixed（按本 spec 列出）
- **Acceptance Criteria**: spec ADDED Requirement「CHANGELOG 与版本号同步」Scenario「1.3.1 发布」
- **Test Requirements**:
  - `programmatic` TR-12.1: `grep -n "\"version\": \"1.3.1\"" package.json` 有命中
  - `programmatic` TR-12.2: `grep -n "\[1.3.1\] - 2026-07-25" CHANGELOG.md` 有命中
  - `programmatic` TR-12.3: CHANGELOG 1.3.1 条目包含第一轮和第二轮的 Fixed 项

## Task 13: 最终验证
- **Priority**: high
- **Depends On**: Task 1-12 全部完成
- **Description**:
  - 运行完整验证套件
  - 确认所有修复未引入新问题
  - 推送分支，开 PR
- **Acceptance Criteria**: spec 全部 Requirement 的全部 Scenario
- **Test Requirements**:
  - `programmatic` TR-13.1: 4 个 Python 脚本通过 `uv run python -c "import ast; ast.parse(...)"`
  - `programmatic` TR-13.2: `git diff --no-index tradingagents-analysis skills/tradingagents-analysis` 输出为空
  - `programmatic` TR-13.3: `node --check install.mjs` 通过
  - `programmatic` TR-13.4: `npm pack --dry-run` 不含 `__pycache__` / `.pyc`
  - `programmatic` TR-13.5: `grep -rn "import sys" tradingagents-analysis/scripts/` 无输出
  - `programmatic` TR-13.6: `grep -rn "~/.opencode/skills" README.md README_CN.md` 无输出
  - `programmatic` TR-13.7: 模拟 `--dir` 缺值、`--dir=/tmp/x`、`--dir ~/x` 三种场景，行为符合 spec
  - `programmatic` TR-13.8: PR 已开，PR body 列出本轮所有修复

# Task Dependencies
- Task 11 依赖 Task 1, 2, 3（Python 修改完成后才统一行尾）
- Task 12 依赖 Task 1-11（所有修复完成后才 bump version + CHANGELOG）
- Task 13 依赖 Task 1-12（最终验证）
- 其余任务可并行
