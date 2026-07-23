# TradingAgents Analysis Skill

**English** | [中文](README_CN.md)

[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](LICENSE)
[![Python](https://img.shields.io/badge/Python-3.10%2B-blue.svg)](https://www.python.org/)

A multi-agent stock and crypto trading analysis skill for AI agents, inspired by [TradingAgents](https://github.com/TauricResearch/TradingAgents) and [TradingAgents-CN](https://github.com/hsliuping/TradingAgents-CN).

> **RESEARCH AND EDUCATION ONLY.** This skill does not constitute financial advice.

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

### Via npx (recommended)

```bash
npx halfoffive/trad-skill
```

This downloads and runs a tiny zero-dependency installer that copies the `tradingagents-analysis/` skill into `~/.claude/skills/tradingagents-analysis` (Claude Code). It prints the next steps when done. Options:

```bash
npx halfoffive/trad-skill --agent agents      # install to ~/.agents/skills
npx halfoffive/trad-skill --agent opencode   # install to ~/.opencode/skills
npx halfoffive/trad-skill --dir <path>       # install into a custom skills dir
```

> If `npx halfoffive/trad-skill` does not auto-run the installer on your npx version, use `npx -p halfoffive/trad-skill trad-skill`.

### Tell your AI agent to install it

> Install the tradingagents-analysis skill from halfoffive/trad-skill

### Manual installation

Copy the skill subfolder to your AI agent's skills directory:

```bash
# For Claude Code (user-level)
cp -r tradingagents-analysis ~/.claude/skills/tradingagents-analysis

# For OpenCode / generic (user-level)
cp -r tradingagents-analysis ~/.agents/skills/tradingagents-analysis

# For OpenCode (project-level)
cp -r tradingagents-analysis .opencode/skills/tradingagents-analysis
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
trad-skill/                        # repo root (meta files + installer)
├── package.json                  # npx entry point (name: trad-skill)
├── install.mjs                   # zero-dependency installer (copies skill into the agent's skills dir)
├── README.md                      # This file (English)
├── README_CN.md                   # Chinese documentation
├── CHANGELOG.md                   # Version history
├── AGENTS.md                      # AI-agent onboarding doc
└── LICENSE                        # Apache 2.0
└── tradingagents-analysis/        # the installable skill
    ├── SKILL.md                   # Core skill instructions (pipeline, orchestration, output format)
    ├── references/
    │   ├── prompts/               # 14 verbatim agent role prompts
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
    │   ├── data-sources.md        # Data source catalog (US + CN markets)
    │   └── indicators.md          # Technical indicator reference
    └── scripts/
        ├── fetch_stock_data.py    # Stock OHLCV data fetcher (US/A-shares/HK/Crypto)
        ├── fetch_news.py          # News data fetcher (company + macro)
        ├── fetch_fundamentals.py  # Fundamentals data fetcher (financial statements)
        └── fetch_sentiment.py     # Sentiment data fetcher (StockTwits, Reddit)
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

Python helper scripts for data fetching, **compaction, and pre-computation**. Functional style, no class definitions. Each script outputs compact formatted text suitable for LLM prompt injection and never raises — on failure it prints an error message the agent can fall back from. By default the outputs are already trimmed so the analyst reasons over a small payload instead of raw data.

> The agent must run these scripts by their **absolute path** inside the installed skill directory (e.g. `~/.claude/skills/tradingagents-analysis/scripts/...`), because a sub-agent's working directory is the user's project, not the skill folder. The skill's `SKILL.md` instructs the main agent to resolve that path before spawning analysts.

```bash
# Stock data: OHLCV tail + pre-computed indicators + optional stats (default; compact)
python scripts/fetch_stock_data.py --symbol AAPL --start 2024-01-01 --end 2024-06-30 --tail 30 --stats

# Or by absolute path (how the agent invokes them):
python ~/.claude/skills/tradingagents-analysis/scripts/fetch_stock_data.py --symbol AAPL --start 2024-01-01 --end 2024-06-30 --tail 30 --stats

# Legacy full-range raw CSV (token-heavy, avoid)
python scripts/fetch_stock_data.py --symbol AAPL --start 2024-01-01 --end 2024-01-31 --raw

# Fetch news (default --limit 8 per source, summaries truncated)
python scripts/fetch_news.py --symbol AAPL --days 7 --limit 8

# Fetch fundamentals (compact key-metrics table + company profile)
python scripts/fetch_fundamentals.py --symbol AAPL

# Fetch sentiment (default --limit 15)
python scripts/fetch_sentiment.py --symbol AAPL --limit 15
```

| Script | Defaults (compact) | Expand flags |
|---|---|---|
| `fetch_stock_data.py` | `--tail 30` + `--indicators` on | `--stats`, `--raw` |
| `fetch_news.py` | `--limit 8`, 200-char summaries | `--limit N`, `--days N` |
| `fetch_fundamentals.py` | compact key-metrics table | — |
| `fetch_sentiment.py` | `--limit 15`, 8 messages/posts shown | `--limit N` |

### Dependencies

```bash
pip install yfinance akshare requests pandas
```

Scripts are the **primary** data source and are tried first. They are not hard dependencies in the sense that, if a script errors or a source is unavailable, the agent falls back to web search / browser tools **only for the parts the script could not provide** — it never skips the scripts outright.

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