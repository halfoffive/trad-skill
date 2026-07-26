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

## Template Variable Substitution

The verbatim role prompts contain LangChain-style template variables (`{ticker}`, `{current_date}`, `{instrument_context}`, `{get_language_instruction()}`, etc.) that the original TradingAgents framework substituted at runtime via LangChain prompt partials. **This skill does not use LangChain** — the main agent must substitute these variables **before** spawning each sub-agent, otherwise the sub-agent receives literal `{...}` strings in its prompt.

The table below defines the substitution for every variable found across `references/prompts/*.md` (30 unique variables). The main agent should apply these when constructing each spawn prompt (see SKILL.md §4 spawn template).

### Identity & labels

| Variable | Substitute with |
|---|---|
| `{ticker}` | The ticker symbol (e.g. `AAPL`, `600519`, `0700.HK`, `BTC-USD`). |
| `{target_label}` | The literal word `stock` for equities (US/CN/HK) or `asset` for crypto (`-USD` tickers). Used in bull/bear prompts as the debate subject noun (e.g. "advocating for investing in the {target_label}" → "advocating for investing in the stock"). Matches source `bull_researcher.py:20` / `bear_researcher.py:24`. |
| `{asset_label}` | The literal word `company` for equities or `asset` for crypto. Used in `news_analyst.md` (e.g. "for {asset_label}-specific news" → "for company-specific news"). Matches source `news_analyst.py:17`. |
| `{company_name}` | The company name if known from `fetch_fundamentals.py` profile (`longName`); otherwise the ticker. |
| `{fundamentals_label}` | The full label string `Company fundamentals report` for equities, or `Asset fundamentals report (may be unavailable for crypto)` for crypto. Used as a section header in bull/bear prompts (e.g. "{fundamentals_label}: {fundamentals_report}"). Matches source `bull_researcher.py:22-25` / `bear_researcher.py:26-29`. |

### Dates

| Variable | Substitute with |
|---|---|
| `{current_date}` | Today's date in `YYYY-MM-DD` format (the trade date). |
| `{start_date}` | Analysis window start (default: today − 365 days; see SKILL.md §6). |
| `{end_date}` | Analysis window end (default: today). |

### Context, language & tooling

| Variable | Substitute with |
|---|---|
| `{instrument_context}` | A one-line market context: `Market: <US / A股 / 港股 / Crypto>; Ticker: <symbol>; Trade date: <YYYY-MM-DD>` (and company name if known). |
| `{get_language_instruction()}` | English (default): **empty string** (no instruction injected — matches source `agent_utils.py:52-65`). Non-English: ` Write your entire response in <lang>.` (note the leading space, matching source). trad-skill does **not** inject any "Respond in English" string when the language is English, to stay verbatim-faithful to the source behavior. |
| `{tool_names}` | Empty string, or `python <skill>/scripts/<script>.py` for the analyst's assigned script. The script is the only tool. |
| `{system_message}` | Empty string (no separate system message; the role prompt itself is the system message). |
| `{NO_EXTERNAL_TOOLS}` | Empty string — **not** set. The spawn template permits web-search / browser fallback when a script fails or returns no data. (Setting it would conflict with the fallback policy.) |
| `{lessons_line}` | Empty string (this skill does not maintain a lessons-learned line between runs). |

> **Note on `{instrument_context}` (R6-11).** trad-skill uses a compact one-line market context for token efficiency. The source repo's `build_instrument_context` (`agent_utils.py:122-169`) produces a fuller paragraph that emphasizes ticker preservation and identity anchoring (anti-hallucination wording added in source #814). trad-skill intentionally drops this anti-hallucination wording to save tokens; agents should rely on SKILL.md §2 (confirm the ticker first) as the anti-hallucination gate. If you observe ticker/company confusion in analyst output, consider expanding `{instrument_context}` to include the source's full paragraph.

> **Note on whitespace before `{get_language_instruction()}` (R6-24).** trad-skill normalizes the whitespace immediately before `{get_language_instruction()}` to a single blank line for readability. The source repo concatenates `+ get_language_instruction()` directly after the prompt string (no blank line). When English (empty substitution), this results in a trailing blank line in trad-skill vs. no trailing blank line in source — cosmetically different but semantically identical. For non-English, the blank line + leading space in the substituted string produces clean paragraph separation.

### Data reports (bound to stage outputs)

These variables bind to the outputs of prior pipeline stages. See SKILL.md §4 "Re-injection discipline" for the digest-vs-full-report strategy.

| Variable | Bound to |
|---|---|
| `{market_research_report}` | Market Analyst report. In debate stages (2 & 5): bind to the `## Key Signals` digest only. In Portfolio Manager: bind full report. |
| `{sentiment_report}` | Sentiment Analyst report (same digest/full rule). |
| `{news_report}` | News Analyst report (same digest/full rule). |
| `{fundamentals_report}` | Fundamentals Analyst report (same digest/full rule). |
| `{history}` | The running debate transcript (paragraph-sized, not full reports). |
| `{investment_plan}` | Research Manager output → Trader input. (Risk Debate does NOT receive `{investment_plan}` — it receives `{trader_decision}`/`{trader_plan}` per the row below.) |
| `{trader_decision}` / `{trader_plan}` | Trader output → Risk Debate input + Portfolio Manager input. |
| `{research_plan}` | Research Manager output → Portfolio Manager input. |
| `{current_response}` | The previous bull/bear argument in the current Research Debate round. |
| `{current_aggressive_response}` | The previous Aggressive Risk argument in the current Risk Debate round. |
| `{current_conservative_response}` | The previous Conservative Risk argument in the current Risk Debate round. |
| `{current_neutral_response}` | The previous Neutral Risk argument in the current Risk Debate round. |

### Pre-fetched data blocks (Sentiment Analyst only)

The `sentiment_analyst.md` prompt expects three pre-fetched data blocks. The original framework fetched these before invoking the LLM; this skill fetches them via `fetch_sentiment.py` at spawn time.

| Variable | Substitute with |
|---|---|
| `{stocktwits_block}` | The StockTwits section of `fetch_sentiment.py` output (US market). For CN market, use the akshare 个股评论 section (semantic equivalent: retail social sentiment). |
| `{reddit_block}` | The Reddit section of `fetch_sentiment.py` output (US market). For CN market, use the akshare 机构参与度 section (semantic equivalent: institutional flow signal). |
| `{news_block}` | **Not provided by `fetch_sentiment.py`.** The sentiment script covers social signals only. If the Sentiment Analyst needs news context for cross-source divergence analysis, it should use web-search fallback, or leave the block empty and note "news block not pre-fetched — see News Analyst report". The News Analyst separately covers news in Stage 1. |

## Tool-Name Override

Some verbatim prompts instruct the agent to call tools like `get_stock_data`, `get_news`, `get_fundamentals`, etc. **These tools do not exist in this skill** — the verbatim text is preserved per the AGENTS.md "never paraphrase" rule, but the actual data source is the Python script. SKILL.md §4 overrides these references at spawn time.

### Market Analyst tools (`market_analyst.md` / `china_market_analyst.md`)

> Note: Only `market_analyst.md` references these ghost tools. The CN counterpart `china_market_analyst.md` does NOT reference any `get_*` tools (verified by grep) — it describes its data source inline (akshare 行情), so the override below applies to the EN prompt only.

- `get_stock_data` / `get_indicators` / `get_verified_market_snapshot` → `python "<skill>/scripts/fetch_stock_data.py" --symbol <ticker> [--start ... --end ...] [--tail ...] [--stats]`
- The script output is the "verified snapshot" — it already pre-computes indicators (see `indicators.md`).
- The analyst must run the script before writing its report; web search / browser tools are a fallback only for parts the script could not provide.

### News Analyst tools (`news_analyst.md` / `cn_news_analyst.md`)

> Note: Only `news_analyst.md` references these ghost tools. The CN counterpart `cn_news_analyst.md` does NOT reference any `get_*` tools (verified by grep) — it describes its data source inline (akshare 新闻 / Google News), so the override below applies to the EN prompt only.

- `get_news(ticker, start_date, end_date)` → `python "<skill>/scripts/fetch_news.py" --symbol <ticker> --days 7 --limit 8`
- `get_global_news(curr_date, look_back_days, limit)` → no separate script; the same `fetch_news.py` output includes macro headlines where available. Use web-search fallback for broader macro context.
- `get_macro_indicators(indicator, curr_date, look_back_days)` → **no script** (FRED not wired). Use web-search fallback.
- `get_prediction_markets(topic, limit)` → **no script** (Polymarket not wired). Use web-search fallback.

### Fundamentals Analyst tools (`fundamentals_analyst.md`)

- `get_fundamentals` → `python "<skill>/scripts/fetch_fundamentals.py" --symbol <ticker>`
- `get_balance_sheet` / `get_cashflow` / `get_income_statement` → **no separate script**. These statements are already distilled into the compact key-metrics table (revenue, net income, EPS, total assets, total debt, operating cash flow, free cash flow) by `fetch_fundamentals.py`. If the analyst needs full statement detail, use web-search fallback.

### Sentiment Analyst tools (`sentiment_analyst.md`)

- The verbatim prompt expects pre-fetched `{news_block}` / `{stocktwits_block}` / `{reddit_block}` data blocks (see "Template Variable Substitution" below). The data source is `python "<skill>/scripts/fetch_sentiment.py" --symbol <ticker> --limit 15`; the script output fills the StockTwits and Reddit blocks. The news block is not provided by the sentiment script (the News Analyst separately covers news).

Do not modify the verbatim prompt files. The overrides live in `SKILL.md` §4, `indicators.md`, and the table below.

## Source Repositories

- **TradingAgents**: https://github.com/TauricResearch/TradingAgents
- **TradingAgents-CN**: https://github.com/hsliuping/TradingAgents-CN
