# TradingAgents Skill 第二轮 Bug 修复 Spec

## Why

第一轮 bug 修复（commit `7c958ec`，spec `.trae/specs/bug-fixes/`）已合并，但通过 5 个并行子代理多角度复审后发现：仍存在 **12 个确认 BUG**（直接影响功能正确性）和 **11 个高影响 ISSUE**（文档/契约不一致会误导用户和代理）。最严重的包括：

- A 股整套基本面/情绪章节静默失败（`to_markdown` 依赖未声明的 `tabulate`）
- 基本面 YoY 计算方向反了（取最旧两年而非最近一年）
- `install.mjs` 把 `__pycache__/*.pyc` 复制到用户目录且 npm 包也打包它
- `install.mjs` 的 `--dir`/`--agent` 缺值时静默走默认路径，`~` 不展开
- README/SKILL.md/data-sources.md 多处描述与脚本实际行为不符（A股检测规则、降级链、宏观新闻能力、OpenCode 安装路径等）
- SKILL.md 的 spawn 模板硬编码脚本名和 `--start/--end`，导致 3/4 分析师脚本会报 `unrecognized arguments`
- CN 专用 prompt 文件存在但 SKILL.md 完全没引用，A股/港股分析丢失 T+1/涨跌停等关键维度
- 第一轮的 5 项修复未入 CHANGELOG，version 未 bump

本轮目标：修掉上述真实缺陷，让脚本/安装器/文档三者自洽，让代理按 SKILL.md 执行不再踩坑。

## What Changes

### Python 脚本（`tradingagents-analysis/scripts/*.py` + `skills/.../scripts/*.py` 双份同步）
- **MODIFIED** `fetch_fundamentals.py` `_yoy()`：取最近一年同比（`iloc[0]` vs `iloc[1]`），同步修 docstring
- **MODIFIED** `fetch_fundamentals.py` A 股表格输出：用 `to_string(index=False)` 或纯 pandas 手写表格替换 `df.to_markdown()`，移除对 `tabulate` 的惰性依赖
- **MODIFIED** `fetch_sentiment.py` A 股表格输出：同上替换 `to_markdown()`
- **MODIFIED** `fetch_stock_data.py` `build_compact_report()`：消除失败时的重复网络请求
- **REMOVED** 4 个脚本中未使用的 `import sys`

### 安装器与发包（`install.mjs`、`package.json`、新增 `.npmignore`）
- **MODIFIED** `install.mjs` `cpSync` 加 `filter` 排除 `__pycache__`
- **MODIFIED** `install.mjs` `parseArgs`：`--dir`/`--agent` 缺值或值以 `--` 开头时 `fail()`，支持 `--dir=PATH`/`--agent=NAME` 等号语法，未知参数 `fail()`
- **MODIFIED** `install.mjs` `--dir` 路径展开 `~` 和 `~/`
- **MODIFIED** `install.mjs` `cpSync`/`rmSync` 包 `try/catch` 走 `fail()`
- **ADDED** `.npmignore`：排除 `**/__pycache__/`、`*.pyc`、`*.pyo`、`.omo/`、`.codegraph/`、`.trae/`、`*.log`

### SKILL.md（两份同步）
- **MODIFIED** §2/§3 A 股自动检测规则：把 `.SS`/`.SZ` 后缀 → A 股 改为「6 位纯数字（如 600519、000858）→ A 股」
- **MODIFIED** §4 spawn 模板：把硬编码的 `fetch_stock_data.py --symbol {ticker} --start {start} --end {end}` 改为 `{SCRIPTS_DIR}/{script_name} {script_args}`，注释明确「按 §6 替换脚本名 **和** 参数」
- **ADDED** §2/§6 对 `--start`/`--end` 默认窗口的指引（默认取 trade date 前 1 年到当天，至少 200 个交易日）
- **ADDED** §3/§4 A 股/港股时的 CN prompt 切换说明（用 `china_market_analyst.md` 替换 `market_analyst.md`，`cn_news_analyst.md` 替换 `news_analyst.md`）
- **MODIFIED** §3 语法：`Stages 1` → `Stage 1`
- **MODIFIED** §6 表格：`fetch_news.py` 描述去掉 "macro"，`fetch_sentiment.py` 描述去掉 "headline analysis"

### 文档（`README.md`、`README_CN.md`、`AGENTS.md`、`CHANGELOG.md`、`references/*.md`）
- **MODIFIED** `README.md` / `README_CN.md`：`--agent opencode` 注释从 `~/.opencode/skills` 改为 `~/.config/opencode/skills`
- **MODIFIED** `README.md` / `README_CN.md`：可用安装位置列表补上 `~/.config/opencode/skills/tradingagents-analysis (OpenCode global)`
- **MODIFIED** `README_CN.md`：A股数据源优先级从 `Tushare → AKShare → Baostock` 改为 `AKShare → yfinance`
- **MODIFIED** `README.md` / `README_CN.md` Project Structure tree：补上 `skills/` 目录，修掉重复的 `└──`
- **MODIFIED** `README.md` / `README_CN.md` A 股检测规则描述与 SKILL.md 对齐
- **MODIFIED** `references/data-sources.md` 降级链：A股改 `AKShare → yfinance`；美股改 `yfinance`；美股新闻改 `yfinance + Google News`；删除未实现的 MongoDB/Tushare/Baostock/TDX/Alpha Vantage/FRED/Polymarket 兜底
- **MODIFIED** `references/data-sources.md` Configuration 章节：注明「此配置体系属于原始 TradingAgents 框架，本 skill 不读取」
- **MODIFIED** `references/prompts/README.md`：删除开发机绝对路径（改为 GitHub 仓库链接），补 MFI 指标，"Tushare data" 改 "akshare data"，5 阶段图注说明 `Decision = Research Manager + Trader`
- **MODIFIED** `AGENTS.md` Gotchas：补一行说明 `.trae/specs/` 是 spec 工作流状态（已跟踪，与 `.omo/` 不同）
- **MODIFIED** `CHANGELOG.md`：新增 `[1.3.1] - 2026-07-25` 条目，列出第一轮 + 第二轮所有 Fixed
- **MODIFIED** `package.json` `version`: `1.3.0` → `1.3.1`

### 行尾符与 gitignore
- **MODIFIED** `fetch_news.py` 两份拷贝行尾符统一（root LF → CRLF 与 skills/ 对齐，或反之；通过 `.gitattributes` 统一）
- **ADDED** `.gitattributes`：`* text=auto eol=lf`（统一行尾，防止后续漂移）
- **MODIFIED** `.gitignore`：补 `.codegraph/`（防御性，不依赖嵌套 .gitignore）

### Non-Goals (Out of Scope，留给后续轮次)
- 不改 prompts/ 下的 verbatim prompt 文件（`market_analyst.md` 等 14 个），只改 `prompts/README.md` 元文档
- 不重构 `days` 参数过滤逻辑（`fetch_yfinance_news` / `fetch_cn_news` / `fetch_reddit_sentiment`）——属行为变更，需独立讨论
- 不实现 Alpha Vantage/FRED/Tushare 等数据源接入
- 不重写 SKILL.md 的整体编排流程
- 不补双语漂移（TDX、A股特别说明等）——下轮处理
- 不动 `CLAUDE.md`（workspace rule）

## Impact

- **Affected specs**: 第一轮 `.trae/specs/bug-fixes/`（已 completed，不动）
- **Affected code**:
  - `tradingagents-analysis/scripts/*.py`（4 个）+ `skills/tradingagents-analysis/scripts/*.py`（4 个，需保持字节同步）
  - `tradingagents-analysis/SKILL.md` + `skills/tradingagents-analysis/SKILL.md`
  - `tradingagents-analysis/references/data-sources.md` + `skills/.../references/data-sources.md`
  - `tradingagents-analysis/references/prompts/README.md` + `skills/.../references/prompts/README.md`
  - `install.mjs`、`package.json`、`.gitignore`、`.gitattributes`（新增）、`.npmignore`（新增）
  - `README.md`、`README_CN.md`、`AGENTS.md`、`CHANGELOG.md`
- **Affected users**: 所有 npx 安装本 skill 的用户（受益于 `__pycache__` 清理、`~` 展开、参数校验）；A股/港股用户（受益于 `to_markdown` 修复、CN prompt 接入、文档校准）；维护者（受益于 CHANGELOG/AGENTS.md 完整性）

## ADDED Requirements

### Requirement: 安装器参数校验与路径展开
安装器 SHALL 对 `--dir` / `--agent` 缺值或值以 `--` 开头时立即失败并打印友好错误；SHALL 支持 `--dir=PATH` / `--agent=NAME` 等号语法；SHALL 展开 `~` 和 `~/` 为用户主目录；SHALL 对未知参数报错而非静默走默认路径；SHALL 在 `cpSync` / `rmSync` 失败时走 `fail()` 而非抛原始堆栈。

#### Scenario: --dir 缺值
- **WHEN** 用户运行 `npx halfoffive/trad-skill --dir`
- **THEN** 退出码非 0，打印 `--dir 需要一个路径参数`，不创建任何目录

#### Scenario: --dir= 等号语法
- **WHEN** 用户运行 `npx halfoffive/trad-skill --dir=/tmp/my-skills`
- **THEN** 技能安装到 `/tmp/my-skills/tradingagents-analysis/`

#### Scenario: ~ 展开
- **WHEN** 用户运行 `npx halfoffive/trad-skill --dir ~/my-skills`
- **THEN** 技能安装到 `<home>/my-skills/tradingagents-analysis/`，不在当前目录创建名为 `~` 的目录

### Requirement: 安装产物不含 Python 字节码
安装器和 npm tarball SHALL 排除 `__pycache__/`、`*.pyc`、`*.pyo`。

#### Scenario: cpSync 过滤
- **WHEN** 源目录存在 `scripts/__pycache__/fetch_stock_data.cpython-313.pyc`
- **THEN** 用户目标目录下不出现 `__pycache__/` 子目录

#### Scenario: npm pack 不含 pyc
- **WHEN** 运行 `npm pack --dry-run`
- **THEN** tarball 列表不含任何 `.pyc` / `__pycache__/` 条目

### Requirement: SKILL.md spawn 模板参数化
SKILL.md §4 的 spawn 模板 SHALL 使用 `{script_name}` / `{script_args}` 占位符，SHALL 明确指示按 §6 表格替换脚本名**和**参数；SHALL 在 §2 或 §6 定义 `--start` / `--end` 默认窗口。

#### Scenario: News 分析师不传 --start/--end
- **WHEN** 代理按模板 spawn News 分析师
- **THEN** 实际运行 `python ".../fetch_news.py" --symbol AAPL --limit 8`，不传 `--start`/`--end`，argparse 不报 `unrecognized arguments`

### Requirement: A 股 / 港股 CN prompt 接入
SKILL.md SHALL 在 §3 或 §4 明确说明：当 `market` 为 A 股或港股时，用 `references/prompts/china_market_analyst.md` 替换 `market_analyst.md`，用 `references/prompts/cn_news_analyst.md` 替换 `news_analyst.md`，其余 3 个分析师保持不变。

#### Scenario: 分析 A 股 600519
- **WHEN** 代理分析 `600519`（A股）
- **THEN** Market 分析师读 `china_market_analyst.md`，News 分析师读 `cn_news_analyst.md`，覆盖 T+1、涨跌停、北向资金等维度

### Requirement: CHANGELOG 与版本号同步
package.json `version` SHALL 与 CHANGELOG 最新条目一致；每次合并 bug 修复 SHALL bump patch 版本并新增 CHANGELOG `Fixed` 条目。

#### Scenario: 1.3.1 发布
- **WHEN** 第二轮修复合并
- **THEN** `package.json` version = `1.3.1`，CHANGELOG 新增 `[1.3.1] - 2026-07-25` 条目，列出第一轮和第二轮所有 Fixed 项

## MODIFIED Requirements

### Requirement: Python 脚本契约一致性
所有 4 个 Python 脚本的每个公开函数 SHALL 始终返回 `str`，SHALL 不抛异常，SHALL 不依赖 `yfinance/akshare/requests/pandas` 之外的第三方包。

#### Scenario: A 股基本面表格在未装 tabulate 时正常输出
- **WHEN** 用户未安装 `tabulate`，调用 `fetch_cn_fundamentals("600519")`
- **THEN** 返回的字符串包含「关键财务指标」表格（由 `to_string(index=False)` 或纯 pandas 生成），不是 `> 获取失败: Import tabulate failed.`

#### Scenario: _yoy 返回最近一年同比
- **WHEN** yfinance `financials` 返回 4 年降序列（2023, 2022, 2021, 2020）
- **THEN** `_yoy(series)` 返回 `(2023 vs 2022)` 的百分比，不是 `(2020 vs 2021)`

### Requirement: 文档与脚本行为一致
README.md / README_CN.md / SKILL.md / references/data-sources.md / references/prompts/README.md 中关于脚本能力、数据源、降级链、A股检测规则、安装路径的描述 SHALL 与脚本和 install.mjs 的实际行为一致。

#### Scenario: A 股检测规则
- **WHEN** 用户阅读 README/SKILL.md 的 A 股检测章节
- **THEN** 看到「6 位纯数字（如 600519、000858）→ A 股」，不是「`.SS`/`.SZ` 后缀 → A 股」

#### Scenario: OpenCode 安装路径
- **WHEN** 用户阅读 README 的 `--agent opencode` 行
- **THEN** 注释为 `# install to ~/.config/opencode/skills`，与 install.mjs 实际路径一致

#### Scenario: data-sources.md 降级链
- **WHEN** 用户阅读 data-sources.md 的 A 股降级链
- **THEN** 看到 `AKShare → yfinance`，不看到 MongoDB/Tushare/Baostock/TDX 等未实现的源

### Requirement: 两份技能目录字节同步
`tradingagents-analysis/` 与 `skills/tradingagents-analysis/` 下所有 git 跟踪文件 SHALL 字节一致（行尾符统一）。

#### Scenario: diff 为空
- **WHEN** 运行 `git diff --no-index tradingagents-analysis skills/tradingagents-analysis`
- **THEN** 输出为空（不含 `__pycache__/` 等 gitignored 文件）

## REMOVED Requirements

### Requirement: 4 个脚本中的 `import sys`
**Reason**: 4 个脚本都导入 `sys` 但全程未引用，属冗余代码。
**Migration**: 直接删除 4 处 `import sys`，不影响功能。

### Requirement: `to_markdown()` 调用
**Reason**: 依赖未声明的 `tabulate`，导致 A 股章节静默失败。
**Migration**: 改用 `to_string(index=False)` 或纯 pandas 手写 markdown 表格构建器，无需新依赖。

## Implementation Strategy

- 新分支：`fix/round2-bugs`（基于最新 `main`）
- 按任务组提交（每组 1 个 commit），每组完成后立即 `git push`
- 所有 Python 修复同时改两份拷贝；提交前用 `git diff --no-index` 验证同步
- 所有 Python 脚本通过 `uv run python -c "import ast; ast.parse(open(f, encoding='utf-8').read())"`
- 全部完成后开 PR 到 `main`，PR body 列出本轮所有修复

## Open Questions
- 无未解决问题。所有修复方案已通过审查确认。
