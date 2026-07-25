# TradingAgents Prompt Reference

Verbatim prompt extractions from the TradingAgents multi-agent trading framework. These prompts are the core IP of the system — each agent role has a carefully crafted prompt that defines its behavior in the pipeline.

## Pipeline Overview

```
Analysts (parallel) → Research Debate (sequential) → Decision → Risk Debate (sequential) → Final Decision
```

> Note: "Decision" here = Research Manager + Trader; see SKILL.md §3 for the full 6-stage flow.

## Prompt Index

| # | File | Role | Pipeline Stage | Source Repo |
|---|------|------|----------------|-------------|
| 1 | [market_analyst.md](market_analyst.md) | Market Analyst | Analyst | TradingAgents |
| 2 | [sentiment_analyst.md](sentiment_analyst.md) | Sentiment Analyst | Analyst | TradingAgents |
| 3 | [news_analyst.md](news_analyst.md) | News Analyst | Analyst | TradingAgents |
| 4 | [fundamentals_analyst.md](fundamentals_analyst.md) | Fundamentals Analyst | Analyst | TradingAgents |
| 5 | [china_market_analyst.md](china_market_analyst.md) | China Market Analyst | Analyst (CN) | TradingAgents-CN |
| 6 | [cn_news_analyst.md](cn_news_analyst.md) | CN News Analyst | Analyst (CN) | TradingAgents-CN |
| 7 | [bull_researcher.md](bull_researcher.md) | Bull Researcher | Research Debate | TradingAgents |
| 8 | [bear_researcher.md](bear_researcher.md) | Bear Researcher | Research Debate | TradingAgents |
| 9 | [research_manager.md](research_manager.md) | Research Manager | Decision | TradingAgents |
| 10 | [trader.md](trader.md) | Trader | Decision | TradingAgents |
| 11 | [aggressive_risk.md](aggressive_risk.md) | Aggressive Risk Analyst | Risk Debate | TradingAgents |
| 12 | [conservative_risk.md](conservative_risk.md) | Conservative Risk Analyst | Risk Debate | TradingAgents |
| 13 | [neutral_risk.md](neutral_risk.md) | Neutral Risk Analyst | Risk Debate | TradingAgents |
| 14 | [portfolio_manager.md](portfolio_manager.md) | Portfolio Manager | Final Decision | TradingAgents |

## Stage Details

### Analyst Stage (parallel)
- **Market Analyst**: Technical analysis using up to 8 complementary indicators (SMA, EMA, MACD, RSI, Bollinger, ATR, VWMA, MFI)
- **Sentiment Analyst**: Multi-source sentiment (Yahoo Finance news + StockTwits + Reddit) with structured output
- **News Analyst**: Macro/news research using FRED data, prediction markets, and global news
- **Fundamentals Analyst**: Financial statements, balance sheet, cash flow, income statement analysis
- **China Market Analyst** (CN): A-share/HK-specific analysis with akshare data (Tushare referenced in prompt but not wired in scripts), T+1 rules, price limits
- **CN News Analyst** (CN): Chinese financial news analysis with timeliness and impact assessment

### Research Debate (sequential, multi-round)
- **Bull Researcher**: Advocates for investing, counters bear arguments
- **Bear Researcher**: Argues against investing, highlights risks and weaknesses

### Decision
- **Research Manager**: Evaluates debate, produces investment plan with Buy/Overweight/Hold/Underweight/Sell rating
- **Trader**: Converts investment plan into concrete transaction proposal

### Risk Debate (sequential, multi-round)
- **Aggressive Risk Analyst**: Champions high-reward strategies, challenges caution
- **Conservative Risk Analyst**: Prioritizes asset protection and stability
- **Neutral Risk Analyst**: Balanced perspective, advocates moderate strategy

### Final Decision
- **Portfolio Manager**: Synthesizes risk debate, delivers final trading decision

## Template Variables

Prompts contain Python f-string variables (e.g., `{ticker}`, `{market_research_report}`) that are populated at runtime from the pipeline state. These document what data each agent expects to receive.

## Source Repositories

- **TradingAgents**: https://github.com/TauricResearch/TradingAgents
- **TradingAgents-CN**: https://github.com/hsliuping/TradingAgents-CN
