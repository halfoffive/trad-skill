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

### 通过 npx skill 安装（推荐）

```bash
npx skill add halfoffive/trad-skill/tradingagents-analysis
```

### 让 AI 代理帮你安装

> 安装 halfoffive/trad-skill 中的 tradingagents-analysis 技能

### 手动安装

将技能子目录复制到 AI 代理的技能文件夹：

```bash
# Claude Code / OpenCode 用户级
cp -r tradingagents-analysis ~/.agents/skills/tradingagents-analysis

# OpenCode 项目级
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
trad-skill/
├── SKILL.md                    # 核心技能指令文件
├── references/
│   ├── prompts/                # 14个智能体角色提示词
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
├── scripts/
│   ├── fetch_stock_data.py     # 行情数据获取
│   ├── fetch_news.py           # 新闻数据获取
│   ├── fetch_fundamentals.py   # 基本面数据获取
│   └── fetch_sentiment.py      # 情绪数据获取
├── README.md                   # 英文文档
├── README_CN.md                # 本文件（中文文档）
└── LICENSE                     # Apache 2.0 许可证
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

Python 辅助脚本（函数式编程，中文注释），用于为分析师子代理获取和格式化数据：

```bash
# 获取行情数据（美股）
python scripts/fetch_stock_data.py --symbol AAPL --start 2024-01-01 --end 2024-01-31

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

> 脚本是辅助工具，不是硬性依赖。如果某个脚本执行失败或数据源不可用，代理可以回退到网络搜索、浏览器工具或其他可用方法来获取所需数据。

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
