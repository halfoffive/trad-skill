# TradingAgents Analysis Skill

[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](LICENSE)
[![Python](https://img.shields.io/badge/Python-3.10%2B-blue.svg)](https://www.python.org/)

A multi-agent stock and crypto trading analysis skill for AI agents, inspired by [TradingAgents](https://github.com/TauricResearch/TradingAgents) and [TradingAgents-CN](https://github.com/hsliuping/TradingAgents-CN).

> **RESEARCH AND EDUCATION ONLY.** This skill does not constitute financial advice.

[Chinese Documentation](README_CN.md)

---

## Overview

This skill teaches an AI agent to replicate the TradingAgents multi-agent analysis pipeline. It orchestrates a team of specialized sub-agents to produce a structured, well-reasoned investment decision for any given ticker.

**Output**: one of five ratings (Buy / Overweight / Hold / Underweight / Sell) with detailed reasoning, key risks, and potential catalysts.

**Supported markets**:

- US stocks (AAPL, MSFT, TSLA...)
- China A-shares (600519, 000858...)
- HK stocks (0700.HK, 9988.HK...)
- Crypto (BTC-USD, ETH-USD...)

The pipeline runs in six stages:

1. **Analyst Team** (parallel sub-agents): Market, Sentiment, News, Fundamentals
2. **Research Debate**: Bull vs Bear researchers (structured argumentation, 1-3 rounds)
3. **Research Manager**: Judges the debate, produces an investment plan with a preliminary rating
4. **Trader**: Converts the plan into a concrete transaction proposal
5. **Risk Debate**: Aggressive vs Conservative vs Neutral analysts (1-3 rounds)
6. **Portfolio Manager**: Final decision with confidence level and full reasoning

---

## Installation

### Via npx skill (recommended)

```bash
npx skill add /path/to/trad-skill
```

### Manual installation

Copy this directory to your AI agent's skills folder:

```bash
# For Claude Code / OpenCode (user-level)
cp -r trad-skill ~/.agents/skills/tradingagents-analysis

# For OpenCode (project-level)
cp -r trad-skill .opencode/skills/tradingagents-analysis
```

---

## Usage

Trigger the skill by asking your AI agent to analyze a ticker:

- "Analyze AAPL for me"
- "What do you think about NVDA?"
- "Run a multi-agent analysis on BTC-USD"
- "Give me a trading analysis of 0700.HK"
- "Analyze 600519 with 3 debate rounds"

The agent will orchestrate the full pipeline and produce a structured investment report ending with a summary table:

| Field | Value |
|---|---|
| Ticker | AAPL |
| Date | 2025-01-15 |
| Rating | Overweight |
| Confidence | medium |
| Market | US |

### Configuration

| Parameter | Range | Default | Description |
|---|---|---|---|
| `max_debate_rounds` | 1-3 | 1 | Bull/bear exchanges in the Research Debate |
| `max_risk_discuss_rounds` | 1-3 | 1 | Risk analyst exchanges in the Risk Debate |
| `output_language` | English / Chinese | match user | Language for all reports |
| `market` | auto-detect | ... | Detected from ticker suffix |

Market auto-detection: `.SS` or `.SZ` suffix means A-shares, `.HK` means HK stocks, `-USD` means Crypto, everything else defaults to US stocks.

---

## Architecture

```
+-----------------------------------------------------+
|                   ANALYST TEAM                       |
|  +----------+ +----------+ +--------+ +----------+  |
|  |  Market  | |Sentiment | |  News  | |Fundament.|  |
|  | Analyst  | | Analyst  | |Analyst | | Analyst  |  |
|  +----------+ +----------+ +--------+ +----------+  |
+-----------------------------------------------------+
                         |
                         v
+-----------------------------------------------------+
|              RESEARCH DEBATE (1-3 rounds)            |
|         Bull Researcher  <->  Bear Researcher        |
+-----------------------------------------------------+
                         |
                         v
              +---------------------+
              |  Research Manager   | -> Investment Plan
              +---------------------+
                         |
                         v
              +---------------------+
              |       Trader        | -> Transaction Proposal
              +---------------------+
                         |
                         v
+-----------------------------------------------------+
|              RISK DEBATE (1-3 rounds)                |
|   Aggressive  <->  Conservative  <->  Neutral       |
+-----------------------------------------------------+
                         |
                         v
              +---------------------+
              | Portfolio Manager   | -> FINAL DECISION
              +---------------------+
```

---

## Project Structure

```
trad-skill/
├── SKILL.md                    # Core skill instructions (pipeline, orchestration, output format)
├── references/
│   ├── prompts/                # 14 verbatim agent role prompts
│   │   ├── market_analyst.md
│   │   ├── sentiment_analyst.md
│   │   ├── news_analyst.md
│   │   ├── fundamentals_analyst.md
│   │   ├── bull_researcher.md
│   │   ├── bear_researcher.md
│   │   ├── research_manager.md
│   │   ├── trader.md
│   │   ├── aggressive_risk.md
│   │   ├── conservative_risk.md
│   │   ├── neutral_risk.md
│   │   ├── portfolio_manager.md
│   │   ├── china_market_analyst.md
│   │   ├── cn_news_analyst.md
│   │   └── README.md
│   ├── data-sources.md         # Data source catalog (US + CN markets)
│   └── indicators.md           # Technical indicator reference
├── scripts/
│   ├── fetch_stock_data.py     # Stock OHLCV data fetcher (US/A-shares/HK/Crypto)
│   ├── fetch_news.py           # News data fetcher (company + macro)
│   ├── fetch_fundamentals.py   # Fundamentals data fetcher (financial statements)
│   └── fetch_sentiment.py      # Sentiment data fetcher (StockTwits, Reddit)
├── README.md                   # This file (English)
├── README_CN.md                # Chinese documentation
└── LICENSE                     # Apache 2.0
```

---

## Data Sources

| Source | Market | Provides | API Key |
|--------|--------|----------|---------|
| Yahoo Finance | US/Global | Price, fundamentals, news | Free |
| Alpha Vantage | US | Price, indicators, fundamentals | Free tier |
| FRED | US Macro | CPI, unemployment, rates | Free |
| Polymarket | Global | Prediction probabilities | Free |
| StockTwits | US | Retail sentiment | Free |
| Reddit | US | Community discussion | Free |
| Tushare | A-shares | Price, fundamentals | Token |
| AKShare | A-shares/HK | Price, news, sentiment | Free |
| Baostock | A-shares | Historical data | Free |

See [references/data-sources.md](references/data-sources.md) for the full catalog with fallback strategies.

---

## Scripts

Python helper scripts for data fetching. Functional style, no class definitions. Each script outputs formatted text suitable for LLM prompt injection.

```bash
# Fetch stock OHLCV data (US, A-shares, HK, Crypto)
python scripts/fetch_stock_data.py --symbol AAPL --start 2024-01-01 --end 2024-01-31

# Fetch news (company + macro, default last 7 days)
python scripts/fetch_news.py --symbol AAPL --days 7

# Fetch fundamentals (income statement, balance sheet, cashflow)
python scripts/fetch_fundamentals.py --symbol AAPL

# Fetch sentiment (StockTwits, Reddit, headline analysis)
python scripts/fetch_sentiment.py --symbol AAPL --limit 30
```

### Dependencies

```bash
pip install yfinance akshare requests pandas
```

Scripts are helpers, not hard dependencies. If a script fails or a data source is unavailable, the agent can fall back to web search, browser tools, or any other available method.

---

## Credits

- **[TauricResearch/TradingAgents](https://github.com/TauricResearch/TradingAgents)**: Original multi-agent trading framework (Apache 2.0)
- **[hsliuping/TradingAgents-CN](https://github.com/hsliuping/TradingAgents-CN)**: China market enhancements, A-share and HK stock support
- **Paper**: Xiao et al., "TradingAgents: Multi-Agents LLM Financial Trading Framework", [arXiv:2412.20138](https://arxiv.org/abs/2412.20138)

---

## Disclaimer

> **This skill is for RESEARCH AND EDUCATION ONLY.**
>
> - Nothing produced by this skill constitutes financial, investment, or trading advice.
> - Past performance does not guarantee future results.
> - LLM-generated analysis is non-deterministic and may contain factual errors, hallucinated data, or flawed reasoning.
> - Always consult a qualified, licensed financial advisor before making any investment decision.
> - The authors and contributors accept no liability for any losses incurred from acting on this output.

---

## License

[Apache License 2.0](LICENSE)