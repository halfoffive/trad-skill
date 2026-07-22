---
name: tradingagents-analysis
description: "Multi-agent stock/crypto trading analysis pipeline inspired by TradingAgents. Orchestrates specialized analyst sub-agents (Market, Sentiment, News, Fundamentals), structured bull/bear research debates, risk assessment debates, and a final portfolio decision. Use when the user asks for stock analysis, trading analysis, multi-agent financial research, investment research, ticker analysis, or says '分析股票', '交易分析', '多智能体分析', '投资研究'. Also trigger on any request to analyze a specific ticker symbol (AAPL, 600519, 0700.HK, BTC-USD, etc.) with a trading/investment lens."
---

# TradingAgents Multi-Agent Analysis

## 1. Overview

This skill replicates the TradingAgents multi-agent pipeline, drawing on the open-source frameworks from TauricResearch/TradingAgents and hsliuping/TradingAgents-CN. It orchestrates a team of specialized sub-agents to produce a structured, well-reasoned investment decision for any given ticker.

**Output**: one of five ratings — Buy / Overweight / Hold / Underweight / Sell — accompanied by detailed reasoning, key risks, and potential catalysts.

**Supported markets**:

- US stocks (AAPL, MSFT, TSLA…)
- China A-shares / A股 (600519, 000858…)
- HK stocks / 港股 (0700.HK, 9988.HK…)
- Crypto (BTC-USD, ETH-USD…)

> **RESEARCH AND EDUCATION ONLY.** Nothing this skill produces constitutes financial advice. See the Disclaimer section below.

---

## 2. Pipeline Architecture

The analysis runs as a six-stage pipeline. Stages 1 uses parallel sub-agents; stages 2–6 run sequentially in the main context.

### Stage 1 — Analyst Team (PARALLEL)

Four sub-agents run simultaneously, each producing a structured report:

| Analyst | Focus | Key inputs |
|---|---|---|
| **Market Analyst** | Technical indicators: SMA, EMA, MACD, RSI, Bollinger Bands, ATR, VWMA | OHLCV price data via `scripts/fetch_stock_data.py` |
| **Sentiment Analyst** | Social and headline sentiment → composite score | StockTwits, Reddit, news headlines via `scripts/fetch_sentiment.py` |
| **News Analyst** | Company news, global macro, FRED indicators, prediction markets | News feeds via `scripts/fetch_news.py` |
| **Fundamentals Analyst** | Financial statements: balance sheet, cashflow, income statement | Financials via `scripts/fetch_fundamentals.py` |

### Stage 2 — Research Debate (SEQUENTIAL, 1–3 rounds)

- **Bull Researcher** argues FOR investing, citing all four analyst reports.
- **Bear Researcher** argues AGAINST investing, citing the same reports.
- They alternate for `max_debate_rounds` exchanges (default: 1).

### Stage 3 — Research Manager

Judges the debate and produces a structured investment plan with a preliminary rating.

### Stage 4 — Trader

Converts the investment plan into a concrete transaction proposal (entry/exit levels, position sizing guidance, time horizon).

### Stage 5 — Risk Debate (SEQUENTIAL, 1–3 rounds)

Three risk perspectives alternate:

- **Aggressive Analyst**: champions high-reward strategies.
- **Conservative Analyst**: prioritizes capital preservation.
- **Neutral Analyst**: balances risk and reward.

They exchange for `max_risk_discuss_rounds` rounds (default: 1).

### Stage 6 — Portfolio Manager

Reviews the trader's proposal and all risk perspectives. Issues the **final decision**: Buy / Overweight / Hold / Underweight / Sell, with confidence level and full reasoning.

---

## 3. Sub-Agent Orchestration

### Spawning the Analyst Team

Spawn **four parallel sub-agents**, one per analyst role. Each sub-agent receives:

1. The **ticker symbol** and **trade date** (default: today).
2. Its **role prompt**, loaded verbatim from the corresponding file in `references/prompts/`:
   - `references/prompts/market_analyst.md`
   - `references/prompts/sentiment_analyst.md`
   - `references/prompts/news_analyst.md`
   - `references/prompts/fundamentals_analyst.md`
3. Access to the relevant **data script** in `scripts/` (see Section 5).

Use background task spawning for parallelism:

```
task(subagent_type="general", run_in_background=true,
     prompt="<role prompt contents>\n\nAnalyze {ticker} as of {date}.\nUse scripts/{script}.py to gather data.")
```

Wait for all four analysts to complete before proceeding.

### Running Sequential Stages

Stages 2–6 run **in the main context**, one after another. For each stage:

1. Load the role prompt from `references/prompts/{role}.md`.
2. Feed in all prior stage outputs as context.
3. Collect the structured output.
4. Pass it forward to the next stage.

Debate stages (2 and 5) loop for the configured number of rounds. Each round, the next speaker receives the full transcript of prior rounds.

---

## 4. Stage-by-Stage Instructions

### Stage 1: Analyst Team

- **Data**: each analyst runs its assigned script (see Section 5).
- **Prompt**: `references/prompts/{role}_analyst.md`.
- **Output**: a structured markdown report with findings, data tables, and a summary assessment.
- **Handoff**: collect all four reports into a single context block for Stage 2.

### Stage 2: Research Debate

- **Data**: the four analyst reports from Stage 1.
- **Prompts**: `references/prompts/bull_researcher.md` and `references/prompts/bear_researcher.md`.
- **Output**: alternating argument paragraphs, each citing specific data points from the analyst reports.
- **Handoff**: the full debate transcript goes to the Research Manager.

### Stage 3: Research Manager

- **Data**: the debate transcript.
- **Prompt**: `references/prompts/research_manager.md`.
- **Output**: a structured investment plan including preliminary rating, thesis, and conditions.
- **Handoff**: the plan goes to the Trader.

### Stage 4: Trader

- **Data**: the investment plan.
- **Prompt**: `references/prompts/trader.md`.
- **Output**: a transaction proposal with entry/exit levels, position sizing, and time horizon.
- **Handoff**: the proposal goes to the Risk Debate.

### Stage 5: Risk Debate

- **Data**: the transaction proposal plus all analyst reports.
- **Prompts**: `references/prompts/aggressive_analyst.md`, `references/prompts/conservative_analyst.md`, `references/prompts/neutral_analyst.md`.
- **Output**: alternating risk perspectives for `max_risk_discuss_rounds` rounds.
- **Handoff**: the risk transcript goes to the Portfolio Manager.

### Stage 6: Portfolio Manager

- **Data**: everything. All analyst reports, the debate, the plan, the proposal, and the risk transcript.
- **Prompt**: `references/prompts/portfolio_manager.md`.
- **Output**: the final decision in the format described in Section 6.

---

## 5. Data Gathering

Helper scripts live in `scripts/`. They fetch and format data for the analyst sub-agents.

| Script | Purpose |
|---|---|
| `scripts/fetch_stock_data.py` | OHLCV price history, technical indicator computation |
| `scripts/fetch_news.py` | Company news, macro news, FRED economic indicators |
| `scripts/fetch_fundamentals.py` | Financial statements: income, balance sheet, cashflow |
| `scripts/fetch_sentiment.py` | Social sentiment from StockTwits, Reddit, headline analysis |

For the full catalog of data sources, APIs, and fallback strategies, see `references/data-sources.md`.

For technical indicator definitions and interpretation guidance, see `references/indicators.md`.

> Scripts are helpers, not hard dependencies. If a script fails or a data source is unavailable, the agent can fall back to web search, browser tools, or any other available method to gather the needed data.

---

## 6. Output Format

The final Portfolio Manager decision must include all of the following:

- **Rating**: exactly one of `Buy` / `Overweight` / `Hold` / `Underweight` / `Sell`.
- **Confidence**: `low` / `medium` / `high`.
- **Reasoning**: multi-paragraph synthesis drawing on all analyst reports, the research debate, and the risk debate. Explain why the rating was chosen and what evidence supports it.
- **Key Risks**: bullet list of the most significant downside scenarios.
- **Catalysts**: bullet list of events or conditions that could move the thesis.

Close with a summary table:

```markdown
| Field | Value |
|---|---|
| Ticker | {ticker} |
| Date | {analysis_date} |
| Rating | {rating} |
| Confidence | {confidence} |
| Market | {US / A股 / 港股 / Crypto} |
```

---

## 7. Configuration

| Parameter | Range | Default | Description |
|---|---|---|---|
| `max_debate_rounds` | 1–3 | 1 | Number of bull/bear exchanges in the Research Debate |
| `max_risk_discuss_rounds` | 1–3 | 1 | Number of risk analyst exchanges in the Risk Debate |
| `output_language` | English / 中文 | match user's language | Language for all reports and the final decision |
| `market` | auto-detect | — | Detected from ticker suffix (see below) |

**Market auto-detection rules**:

- Suffix `.SS` or `.SZ` → A股 (China A-shares)
- Suffix `.HK` → 港股 (HK stocks)
- Suffix `-USD` → Crypto
- Everything else → US stocks

The user can override any of these by stating preferences explicitly (e.g., "用中文输出", "run 3 debate rounds").

---

## 8. Disclaimer

> **This skill is for RESEARCH AND EDUCATION ONLY.**
>
> - Nothing produced by this skill constitutes financial, investment, or trading advice.
> - Past performance does not guarantee future results.
> - LLM-generated analysis is non-deterministic and may contain factual errors, hallucinated data, or flawed reasoning.
> - Always consult a qualified, licensed financial advisor before making any investment decision.
> - The authors and contributors of this skill accept no liability for any losses incurred from acting on its output.

---

## 9. Credits

- Based on [TauricResearch/TradingAgents](https://github.com/TauricResearch/TradingAgents) (Apache 2.0).
- China market enhancements from [hsliuping/TradingAgents-CN](https://github.com/hsliuping/TradingAgents-CN).
- Paper: "TradingAgents: Multi-Agents LLM Financial Trading Framework" ([arXiv:2412.20138](https://arxiv.org/abs/2412.20138)).
