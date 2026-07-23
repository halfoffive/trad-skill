# TradingAgents 多智能体分析技能

[English](README.md) | **中文**

[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](LICENSE)
[![Python](https://img.shields.io/badge/Python-3.10%2B-blue.svg)](https://www.python.org/)

基于 [TradingAgents](https://github.com/TauricResearch/TradingAgents) 和 [TradingAgents-CN](https://github.com/hsliuping/TradingAgents-CN) 的多智能体股票/加密货币分析 AI 技能。

> **仅供研究与学习使用** — 本技能不构成任何投资建议。

---

## 概述

本技能教会 AI 代理复现 TradingAgents 多智能体分析流水线：

1. **分析师团队**（并行子代理）：市场分析师、情绪分析师、新闻分析师、基本面分析师
2. **研究辩论**：看多研究员 vs 看空研究员（结构化辩论）
3. **研究主管**：评判辩论 → 生成带评级的投资计划
4. **交易员**：将计划转化为具体交易提案
5. **风控辩论**：激进派 vs 保守派 vs 中性派
6. **投资组合经理**：最终决策（买入/增持/持有/减持/卖出）

**支持市场**：

- 美股（AAPL, MSFT, TSLA…）
- A股 / 中国A股（600519, 000858…）
- 港股（0700.HK, 9988.HK…）
- 加密货币（BTC-USD, ETH-USD…）

---

## 安装

### 通过 npx 安装（推荐）

```bash
npx halfoffive/trad-skill
```

这会下载并运行一个零依赖的微型安装器，把 `tradingagents-analysis/` 技能复制到 `~/.claude/skills/tradingagents-analysis`（Claude Code）。安装器会打印后续步骤。可选参数：

```bash
npx halfoffive/trad-skill --agent agents      # 安装到 ~/.agents/skills
npx halfoffive/trad-skill --agent opencode   # 安装到 ~/.opencode/skills
npx halfoffive/trad-skill --dir <path>       # 安装到自定义技能目录
```

> 若 `npx halfoffive/trad-skill` 在你的 npx 版本上没有自动运行安装器，请使用 `npx -p halfoffive/trad-skill trad-skill`。

### 让 AI 代理帮你安装

> 安装 halfoffive/trad-skill 中的 tradingagents-analysis 技能

### 手动安装

将技能子目录复制到 AI 代理的技能文件夹：

```bash
# Claude Code（用户级）
cp -r tradingagents-analysis ~/.claude/skills/tradingagents-analysis

# OpenCode / 通用（用户级）
cp -r tradingagents-analysis ~/.agents/skills/tradingagents-analysis

# OpenCode（项目级）
cp -r tradingagents-analysis .opencode/skills/tradingagents-analysis
```

---

## 使用

向你的 AI 代理请求分析某个标的即可触发：

- "分析一下苹果公司 AAPL"
- "帮我看看英伟达 NVDA 怎么样"
- "分析贵州茅台 600519"
- "用多智能体分析一下比特币 BTC-USD"
- "帮我分析港股腾讯 0700.HK"
- "交易分析：特斯拉 TSLA"
- "投资研究：比亚迪 002594"

代理将编排完整的分析流水线，生成结构化的投资研究报告。

### A股特别说明

- 股票代码使用6位纯数字格式（如 600519、000858）
- 自动识别上海/深圳市场（`.SS` 后缀为上海，`.SZ` 后缀为深圳）
- 数据源优先级：Tushare → AKShare → Baostock
- 支持中文新闻和情绪分析
- 使用中国市场专用分析师提示词（`china_market_analyst.md`、`cn_news_analyst.md`）

### 港股特别说明

- 使用5位数字 + `.HK` 后缀（如 00700.HK、09988.HK）
- 数据源：AKShare → yfinance
- 支持港股通标的和港股主板股票

---

## 架构

```
┌─────────────────────────────────────────────────────┐
│                   分析师团队（并行）                    │
│  ┌──────────┐ ┌──────────┐ ┌────────┐ ┌──────────┐ │
│  │ 市场分析师│ │情绪分析师│ │新闻分析师│ │基本面分析师│ │
│  └──────────┘ └──────────┘ └────────┘ └──────────┘ │
└─────────────────────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────┐
│            研究辩论（1-3 轮）                          │
│         看多研究员  ↔  看空研究员                      │
└─────────────────────────────────────────────────────┘
                         │
                         ▼
              ┌─────────────────────┐
              │     研究主管         │ → 投资计划
              └─────────────────────┘
                         │
                         ▼
              ┌─────────────────────┐
              │      交易员          │ → 交易提案
              └─────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────┐
│            风控辩论（1-3 轮）                          │
│     激进派  ↔  保守派  ↔  中性派                      │
└─────────────────────────────────────────────────────┘
                         │
                         ▼
              ┌─────────────────────┐
              │   投资组合经理       │ → 最终决策
              └─────────────────────┘
```

### 配置参数

| 参数 | 范围 | 默认值 | 说明 |
|------|------|--------|------|
| `max_debate_rounds` | 1–3 | 1 | 研究辩论中看多/看空交锋轮数 |
| `max_risk_discuss_rounds` | 1–3 | 1 | 风控辩论中三方交锋轮数 |
| `output_language` | English / 中文 | 跟随用户语言 | 所有报告和最终决策的输出语言 |
| `market` | 自动检测 | — | 根据代码后缀自动识别市场 |

---

## 项目结构

```
trad-skill/                        # 仓库根目录（元文件 + 安装器）
├── package.json                  # npx 入口（name: trad-skill）
├── install.mjs                   # 零依赖安装器（把技能复制到 agent 的 skills 目录）
├── README.md                      # 英文文档
├── README_CN.md                   # 本文件（中文文档）
├── CHANGELOG.md                   # 版本历史
├── AGENTS.md                      # AI 代理接入文档
└── LICENSE                        # Apache 2.0 许可证
└── tradingagents-analysis/        # 可安装的技能
    ├── SKILL.md                   # 核心技能指令文件
    ├── references/
    │   ├── prompts/               # 14个智能体角色提示词
    │   │   ├── market_analyst.md       # 市场分析师
    │   │   ├── sentiment_analyst.md    # 情绪分析师
    │   │   ├── news_analyst.md         # 新闻分析师
    │   │   ├── fundamentals_analyst.md # 基本面分析师
    │   │   ├── bull_researcher.md      # 看多研究员
    │   │   ├── bear_researcher.md      # 看空研究员
    │   │   ├── research_manager.md     # 研究主管
    │   │   ├── trader.md               # 交易员
    │   │   ├── aggressive_risk.md      # 激进风控
    │   │   ├── conservative_risk.md    # 保守风控
    │   │   ├── neutral_risk.md         # 中性风控
    │   │   ├── portfolio_manager.md    # 投资组合经理
    │   │   ├── china_market_analyst.md # 中国市场分析师
    │   │   ├── cn_news_analyst.md      # 中文新闻分析师
    │   │   └── README.md               # 提示词索引
    │   ├── data-sources.md         # 数据源目录（美股+A股+港股）
    │   └── indicators.md           # 技术指标参考
    └── scripts/
        ├── fetch_stock_data.py     # 行情数据获取
        ├── fetch_news.py           # 新闻数据获取
        ├── fetch_fundamentals.py   # 基本面数据获取
        └── fetch_sentiment.py      # 情绪数据获取
```

---

## 数据源

### 美股数据源

| 数据源 | 提供内容 | API Key |
|--------|----------|---------|
| Yahoo Finance | 行情、基本面、新闻 | 免费 |
| Alpha Vantage | 行情、指标、基本面 | 免费额度 |
| FRED | 宏观经济指标 | 免费 |
| Polymarket | 预测市场概率 | 免费 |
| StockTwits | 散户情绪 | 免费 |
| Reddit | 社区讨论 | 免费 |

### A股/港股数据源

| 数据源 | 提供内容 | API Key |
|--------|----------|---------|
| Tushare | A股行情、基本面 | 需要 Token |
| AKShare | A股/港股行情、新闻、情绪 | 免费开源 |
| Baostock | A股历史数据 | 免费 |
| 通达信 TDX | 技术指标 | 免费 |

详见 [references/data-sources.md](references/data-sources.md)。

---

## 脚本

Python 辅助脚本（函数式编程，中文注释），用于为分析师子代理获取和格式化数据，永不抛异常——失败时打印错误信息，代理可据此回退。

> 代理必须用技能安装目录内的**绝对路径**来运行这些脚本（如 `~/.claude/skills/tradingagents-analysis/scripts/...`），因为子代理的工作目录是用户项目，而非技能文件夹。`SKILL.md` 已指示主代理在派生子代理前先解析该路径。

```bash
# 在技能的 scripts/ 目录内手动测试：
python scripts/fetch_stock_data.py --symbol AAPL --start 2024-01-01 --end 2024-01-31

# 或用绝对路径调用（代理实际调用方式）：
python ~/.claude/skills/tradingagents-analysis/scripts/fetch_stock_data.py --symbol AAPL --start 2024-01-01 --end 2024-01-31

# 获取行情数据（A股）
python scripts/fetch_stock_data.py --symbol 600519 --start 2024-01-01 --end 2024-01-31

# 获取新闻（美股）
python scripts/fetch_news.py --symbol AAPL --days 7

# 获取新闻（A股）
python scripts/fetch_news.py --symbol 600519 --days 7

# 获取基本面（美股）
python scripts/fetch_fundamentals.py --symbol AAPL

# 获取基本面（A股）
python scripts/fetch_fundamentals.py --symbol 600519

# 获取情绪数据（美股）
python scripts/fetch_sentiment.py --symbol AAPL --limit 30

# 获取情绪数据（A股）
python scripts/fetch_sentiment.py --symbol 600519
```

> 脚本是**主要**数据源，必须优先尝试。它们不是硬性依赖是指：当某个脚本执行失败或数据源不可用时，代理**仅对脚本未能提供的部分**回退到网络搜索/浏览器工具——绝不跳过脚本直接用网页搜索。

### 依赖安装

```bash
pip install yfinance akshare requests pandas
```

---

## 致谢

- **[TauricResearch/TradingAgents](https://github.com/TauricResearch/TradingAgents)** — 原创多智能体交易框架（Apache 2.0）
- **[hsliuping/TradingAgents-CN](https://github.com/hsliuping/TradingAgents-CN)** — 中国市场增强版
- **论文**: Xiao et al., "TradingAgents: Multi-Agents LLM Financial Trading Framework", [arXiv:2412.20138](https://arxiv.org/abs/2412.20138)

感谢上述项目的开源贡献，本技能的提示词和分析方法论均源自这些优秀项目。

---

## 免责声明

> ⚠️ **重要提示**：本技能**仅供研究与学习使用**。
>
> - 不构成任何投资建议、财务建议或交易建议
> - 过往表现不代表未来收益
> - 大语言模型分析具有非确定性，可能包含事实错误、虚构数据或有缺陷的推理
> - 投资决策前请务必咨询持牌金融顾问
> - 作者和贡献者不对基于本工具输出做出的任何交易决策承担责任
> - 投资有风险，入市需谨慎

---

## 许可证

[Apache License 2.0](LICENSE)
