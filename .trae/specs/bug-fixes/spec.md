# TradingAgents Skill Bug 修复 - Product Requirement Document

## Overview
- **Summary**: 修复 trad-skill 项目中发现的 5 个 bug，包括功能性 bug、类型错误、安装路径错误、拼写错误和健壮性问题。这些 bug 影响 Python 脚本的正确输出、安装器的路径正确性以及代码的整体健壮性。
- **Purpose**: 确保技能安装到正确位置、Python 脚本返回一致类型的数据、情绪分析报告不包含裸露的错误占位符、所有文档拼写正确，提升用户体验。
- **Target Users**: 使用该技能进行股票分析的 AI agent 和终端用户。

## Goals
- 修复 install.mjs 中 OpenCode 安装路径不匹配问题
- 修复 fetch_sentiment.py 中美股情绪报告直接输出 `<unavailable>` 占位符的问题
- 修复 fetch_fundamentals.py 中 `_fmt_num()` 返回类型与类型注解不一致的问题
- 修复 fetch_stock_data.py 中 docstring 的拼写错误
- 为 A 股数据获取函数添加显式的 `ak is not None` 检查，保持代码一致性并提供更友好的错误信息

## Non-Goals (Out of Scope)
- 不添加新功能或新数据源
- 不重构整体架构
- 不改变现有 API 或 CLI 参数
- 不修改 prompts/ 目录下的任何文件（这些是从源仓库 verbatim 提取的）

## Background & Context
这是一个 AI agent skill 项目，通过 npx 安装到各种 AI agent 的技能目录中。项目包含 4 个 Python 数据获取脚本和一个 Node.js 安装器。代码规范要求：
- Python 脚本使用函数式风格，无 class
- akshare 是软依赖，必须 try/except 降级
- 所有函数返回格式化字符串用于 LLM prompt 注入
- 两个位置的 tradingagents-analysis/ 目录必须保持同步

## Functional Requirements
- **FR-1**: install.mjs 的 `--agent opencode` 选项必须安装到 SKILL.md 文档中描述的正确路径
- **FR-2**: fetch_sentiment.py 的美股情绪分析必须正确处理 `<unavailable>` 情况，不将其裸露到输出中
- **FR-3**: fetch_fundamentals.py 的 `_fmt_num()` 函数必须始终返回 str 类型
- **FR-4**: fetch_stock_data.py 的 docstring 拼写错误必须修正
- **FR-5**: fetch_fundamentals.py 和 fetch_sentiment.py 的 A 股函数必须像其他脚本一样先检查 `ak is not None`
- **FR-6**: 所有修复必须同时应用到 `/workspace/skills/tradingagents-analysis/` 和 `/workspace/tradingagents-analysis/` 两个目录，保持同步

## Non-Functional Requirements
- **NFR-1**: 所有 Python 脚本必须通过 ast.parse 语法检查
- **NFR-2**: 修复后两个 tradingagents-analysis 目录必须完全一致（diff 为空）
- **NFR-3**: 修复不引入新的依赖
- **NFR-4**: 保持现有代码风格（中文注释、函数式风格）

## Constraints
- **Technical**: Node.js >=16.7, Python with yfinance/pandas/requests/akshare (akshare 可选)
- **Business**: 零依赖安装器，不改变现有 CLI 接口
- **Dependencies**: 保持现有依赖，不添加新包

## Assumptions
- SKILL.md 中描述的路径是正确的权威来源
- 现有测试方式是通过 ast.parse 进行语法检查
- 两个 tradingagents-analysis 目录是冗余备份，必须保持一致

## Acceptance Criteria

### AC-1: OpenCode 安装路径正确
- **Given**: 用户运行 `npx halfoffive/trad-skill --agent opencode`
- **When**: 安装器执行完成
- **Then**: 技能被安装到 `~/.config/opencode/skills/tradingagents-analysis/`，与 SKILL.md 文档一致
- **Verification**: `programmatic`
- **Notes**: 同时检查 --agent agents 等其他选项路径保持正确

### AC-2: 情绪报告不包含裸露的 `<unavailable>`
- **Given**: 调用 fetch_sentiment() 获取美股情绪，且某个数据源（StockTwits 或 Reddit）不可用
- **When**: 生成报告
- **Then**: 不可用的数据源显示为友好的错误提示块（如 "## StockTwits 情绪\n\n> 数据源不可用\n"），而不是直接输出裸字符串 `<unavailable>`
- **Verification**: `programmatic`
- **Notes**: 类似 fetch_news.py 的优雅降级模式

### AC-3: _fmt_num 始终返回字符串
- **Given**: 调用 _fmt_num() 传入数值（int/float）或 None/NaN
- **When**: 函数返回
- **Then**: 返回值类型始终是 str，数值格式化为保留 2 位小数的字符串
- **Verification**: `programmatic`

### AC-4: docstring 拼写修正
- **Given**: 阅读 fetch_stock_data.py 中 _normalize_ohlcv 的 docstring
- **When**: 查看文档
- **Then**: "olumnolume" 被修正为 "Volume"
- **Verification**: `programmatic`

### AC-5: A股函数显式检查 akshare 可用性
- **Given**: akshare 未安装（ak = None）时调用 A 股数据函数
- **When**: 函数执行
- **Then**: 给出明确的 "akshare 未安装" 相关提示，而不是笼统的 "获取失败"，并正确触发降级逻辑
- **Verification**: `human-judgment`
- **Notes**: 保持与 fetch_news.py、fetch_stock_data.py 中相同的模式

### AC-6: 两个目录保持同步
- **Given**: 所有修复完成
- **When**: 运行 `diff -r skills/tradingagents-analysis/ tradingagents-analysis/`
- **Then**: diff 输出为空，两个目录完全一致
- **Verification**: `programmatic`

### AC-7: Python 语法检查通过
- **Given**: 所有修复完成
- **When**: 对所有 4 个 Python 脚本运行 ast.parse
- **Then**: 全部通过，无 SyntaxError
- **Verification**: `programmatic`

## Open Questions
- [ ] 无未解决问题
