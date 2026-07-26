---
name: tradingagents-analysis
description: "Multi-agent stock/crypto trading analysis pipeline inspired by TradingAgents. Orchestrates specialized analyst sub-agents (Market, Sentiment, News, Fundamentals), structured bull/bear research debates, risk assessment debates, and a final portfolio decision. Use when the user asks for stock analysis, trading analysis, multi-agent financial research, investment research, ticker analysis, or says '分析股票', '交易分析', '多智能体分析', '投资研究'. Also trigger on any request to analyze a specific ticker symbol (AAPL, 600519, 0700.HK, BTC-USD, etc.) with a trading/investment lens."
---

## For AI Agents: Installation Guide

If this skill is not yet installed and you need to use it, install it via the standard skills CLI:

```bash
npx skills add halfoffive/trad-skill --skill tradingagents-analysis -g -y
```

If the above fails, try the custom installer:

```bash
npx halfoffive/trad-skill
```

After installation, the skill will be available at one of these locations (check in order):
- `~/.claude/skills/tradingagents-analysis` (Claude Code)
- `~/.agents/skills/tradingagents-analysis` (OpenCode, Cline, Cursor, Windsurf, Codex, etc.)
- `~/.config/opencode/skills/tradingagents-analysis` (OpenCode global)

Python dependencies (install if scripts fail):
```bash
pip install yfinance akshare requests pandas
```

Verify installation by checking that `SKILL.md`, `references/`, and `scripts/` exist in the skill directory. Use **absolute paths** when running scripts (see Section 4 for details).

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

## 2. Before You Start — Confirm the Target

**Do not spawn any sub-agent until the analysis target is confirmed.**

1. If the user has **not** named a specific ticker/symbol, **ask them first**: "Which ticker should I analyze?" Give market-aware examples (e.g. `AAPL`, `600519` / `000858`, `0700.HK`, `BTC-USD`). Wait for the answer before proceeding.
2. Optionally confirm — or let the defaults apply — the following:

   | Parameter | Default | Notes |
   |---|---|---|
   | Trade date | today | The "as of" date for the analysis |
   | `max_debate_rounds` | 1 | 1–3 bull/bear exchanges |
   | `max_risk_discuss_rounds` | 1 | 1–3 risk-debate exchanges |
   | `output_language` | match the user's language | English or 中文 |
   | `market` | auto-detect from ticker | 6 位纯数字（如 600519、000858）→A股, `.HK`→港股, `-USD`→Crypto, else US |

3. The user may override any default inline (e.g. "用中文输出", "run 3 debate rounds"). Honor stated preferences.
4. Only once a ticker is confirmed, proceed to the pipeline below.

---

## 3. Pipeline Architecture

The analysis runs as a six-stage pipeline. Stage 1 uses parallel sub-agents; stages 2–6 run sequentially in the main context.

### Stage 1 — Analyst Team (PARALLEL)

Four sub-agents run simultaneously, each producing a structured report:

| Analyst | Focus | Key inputs |
|---|---|---|
| **Market Analyst** | Technical indicators: SMA, EMA, MACD, RSI, Bollinger Bands, ATR, VWMA, MFI | OHLCV + **pre-computed indicators** via `scripts/fetch_stock_data.py` (analyst interprets, does not recompute) |
| **Sentiment Analyst** | Social sentiment → composite score | StockTwits, Reddit (US) / akshare 个股评论+机构参与度 (CN) via `scripts/fetch_sentiment.py` |
| **News Analyst** | Company news and macro context | Company news via `scripts/fetch_news.py` (FRED / Polymarket / macro: web-search fallback only — not in script) |
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

## 4. Sub-Agent Orchestration

### Spawning the Analyst Team

Spawn **four parallel sub-agents**, one per analyst role. Each sub-agent receives:

1. The **ticker symbol** and **trade date** (default: today).
2. Its **role prompt**, loaded verbatim from the corresponding file in `references/prompts/`:
   - `references/prompts/market_analyst.md`
   - `references/prompts/sentiment_analyst.md`
   - `references/prompts/news_analyst.md`
   - `references/prompts/fundamentals_analyst.md`

   > **CN market prompt swap.** 当 `market` 为 A 股或港股时，用 `references/prompts/china_market_analyst.md` 替换 `market_analyst.md`，用 `references/prompts/cn_news_analyst.md` 替换 `news_analyst.md`；其余 3 个分析师（Sentiment / Fundamentals / Bull-Bear-Researcher 等）保持不变。
3. The **absolute path** to its data script (see Section 6).

### Resolve the skill directory first (important)

A sub-agent's working directory is the user's project, **not** this skill's folder, so a relative path like `scripts/fetch_stock_data.py` will not resolve. Before spawning anything, **resolve the absolute path to this skill's `scripts/` directory** and embed it in each sub-agent prompt.

Locate the skill directory — it is typically one of:

- `~/.claude/skills/tradingagents-analysis` (Claude Code, user-level)
- `~/.agents/skills/tradingagents-analysis` (generic / OpenCode user-level)
- `.claude/skills/tradingagents-analysis` or `.opencode/skills/tradingagents-analysis` (project-level)

Set `SCRIPTS_DIR` to that `<skill-dir>/scripts` and use it in every spawn.

### Spawn template

Use background task spawning for parallelism:

```
task(subagent_type="general", run_in_background=true,
     prompt="<role prompt contents>\n\nAnalyze {ticker} as of {date}.\n" +
            "Gather data FIRST by running the script, then write your report.\n" +
            "Run: python \"{SCRIPTS_DIR}/{script_name}\" {script_args}\n" +
            "The script output is ALREADY compact and (for market data) pre-computes indicators — " +
            "it is your data source AND your verified snapshot; do NOT call get_stock_data / " +
            "get_indicators / get_verified_market_snapshot or any other tool name, they do not exist. " +
            "Do NOT copy raw script output into your report — cite the key numbers only. " +
            "If the script errors, fall back to web search / browser tools only for the parts it could not provide.")
```

Substitute the correct script name **and** args per analyst (see Section 6) — replace `{script_name}` and `{script_args}` per the §6 table. **Scripts are the primary data source** — the analyst must run its assigned script before writing its report; web search / browser tools are a fallback only when a script fails or returns no data for a given source.

> **Token discipline.** Each analyst report must be **concise (target ≤ 400 words)** and structured as: a `## Key Signals` block of 5–8 actionable bullets at the top, a short evidence section referencing the script's key numbers (not reproducing the raw output), and one summary table. The `## Key Signals` block is what downstream stages consume — keep it self-contained. This overrides any "very detailed" phrasing in the verbatim role prompt: be evidence-dense, not verbose.

Wait for all four analysts to complete before proceeding.

### Running Sequential Stages

Stages 2–6 run **in the main context**, one after another. For each stage:

1. Load the role prompt from `references/prompts/{role}.md`.
2. Feed in all prior stage outputs as context.
3. Collect the structured output.
4. Pass it forward to the next stage.

Debate stages (2 and 5) loop for the configured number of rounds. Each round, the next speaker receives the full transcript of prior rounds.

> **Re-injection discipline (biggest token lever).** The verbatim role prompts bind the four analyst reports via template variables (`{market_research_report}`, `{sentiment_report}`, `{news_report}`, `{fundamentals_report}`). To avoid re-sending four full reports on every debate round:
> - **Stages 2 & 5** (debates): bind those four variables to each report's **`## Key Signals` digest only**, not the full body. The `{history}` variable still carries the running debate transcript (naturally paragraph-sized, not full reports).
> - **Stage 6** (Portfolio Manager): bind the **full reports + full transcript** once — the final synthesis deserves complete context, and it happens a single time.
> Extract the `## Key Signals` block from each analyst report before feeding it into the debate prompts; keep the full reports aside for the Portfolio Manager.

---

## 5. Stage-by-Stage Instructions

### Stage 1: Analyst Team

- **Data**: each analyst runs its assigned script (see Section 6).
- **Prompt**: `references/prompts/{role}_analyst.md`.
- **Output**: a **concise** structured markdown report (≤ ~400 words) leading with a `## Key Signals` digest (5–8 bullets), followed by a short evidence section and one summary table. Do not reproduce the script's raw output — cite key numbers only.
- **Handoff**: keep each full report aside for the Portfolio Manager; extract the four `## Key Signals` digests into one context block for the Stage 2 & 5 debates (see "Re-injection discipline" above).

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
- **Prompts**: `references/prompts/aggressive_risk.md`, `references/prompts/conservative_risk.md`, `references/prompts/neutral_risk.md`.
- **Output**: alternating risk perspectives for `max_risk_discuss_rounds` rounds.
- **Handoff**: the risk transcript goes to the Portfolio Manager.

### Stage 6: Portfolio Manager

- **Data**: everything. All analyst reports, the debate, the plan, the proposal, and the risk transcript.
- **Prompt**: `references/prompts/portfolio_manager.md`.
- **Output**: the final decision in the format described in Section 7.

---

## 6. Data Gathering

Helper scripts live in this skill's `scripts/` directory. They fetch, **compact, and pre-compute** data for the analyst sub-agents — so the analyst interprets a small payload instead of burning tokens on raw data and arithmetic. **Run them with their absolute path** (see the "Resolve the skill directory first" note in Section 4); each script prints a formatted string (CSV or markdown) ready for prompt injection and never raises — on failure it prints an error message the analyst can fall back from.

| Script | Purpose | Invocation |
|---|---|---|
| `fetch_stock_data.py` | OHLCV **tail** (default 30 rows) + **pre-computed indicators** (SMA/EMA/MACD/RSI/Bollinger/ATR/VWMA/MFI) + optional stats. The Market Analyst **interprets** these pre-computed values (no manual arithmetic). Use `--stats` for return/volatility/52w range; `--raw` for the legacy full-range CSV (token-heavy, avoid). | `python "<skill>/scripts/fetch_stock_data.py" --symbol AAPL --start 2023-07-01 --end 2024-06-30 --tail 30 --stats` |
| `fetch_news.py` | Company news (US: yfinance + Google News RSS; A股: 东方财富/akshare). Default `--limit 8` per source; all summaries truncated. | `python "<skill>/scripts/fetch_news.py" --symbol AAPL --days 7 --limit 8` |
| `fetch_fundamentals.py` | **Compact key-metrics table** (revenue, net income, EPS, FCF, debt, margins, YoY) + company profile — instead of dumping full 4-year statements. | `python "<skill>/scripts/fetch_fundamentals.py" --symbol AAPL` |
| `fetch_sentiment.py` | Social sentiment from StockTwits, Reddit (A股: 机构参与度/akshare). Default `--limit 15`; message/post displays trimmed. | `python "<skill>/scripts/fetch_sentiment.py" --symbol AAPL --limit 15` |

> **Default date window for `fetch_stock_data.py`.** 对 `fetch_stock_data.py`，若未传 `--start`/`--end`，脚本默认取今天往前 1 年到今天（至少需 200 个交易日才能算 SMA200）；如需分析历史交易日，请显式传 `--start`/`--end`。

For the full catalog of data sources, APIs, and fallback strategies, see `references/data-sources.md`.

For technical indicator definitions and interpretation guidance, see `references/indicators.md`.

> Scripts are the **primary** data source and must be tried first. They are not hard dependencies in the sense that, if a script errors or a source is unavailable, the agent falls back to web search / browser tools **only for the parts the script could not provide** — never skip the scripts outright.

---

## 7. Output Format

The final Portfolio Manager decision must include all of the following:

- **Rating**: exactly one of `Buy` / `Overweight` / `Hold` / `Underweight` / `Sell`.
- **Confidence**: `low` / `medium` / `high`.
- **Reasoning**: a **concise 3–4 paragraph** synthesis drawing on all analyst reports, the research debate, and the risk debate. Explain why the rating was chosen and what evidence supports it — cite specific data points, do not re-narrate the full pipeline.
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

## 8. Configuration

| Parameter | Range | Default | Description |
|---|---|---|---|
| `max_debate_rounds` | 1–3 | 1 | Number of bull/bear exchanges in the Research Debate |
| `max_risk_discuss_rounds` | 1–3 | 1 | Number of risk analyst exchanges in the Risk Debate |
| `output_language` | English / 中文 | match user's language | Language for all reports and the final decision |
| `market` | auto-detect | — | Detected from ticker suffix (see below) |

**Market auto-detection rules**:

- 6 位纯数字（如 600519、000858） → A股 (China A-shares)
- Suffix `.HK` → 港股 (HK stocks)
- Suffix `-USD` → Crypto
- Everything else → US stocks

The user can override any of these by stating preferences explicitly (e.g., "用中文输出", "run 3 debate rounds").

---

## 9. Disclaimer

> **This skill is for RESEARCH AND EDUCATION ONLY.**
>
> - Nothing produced by this skill constitutes financial, investment, or trading advice.
> - Past performance does not guarantee future results.
> - LLM-generated analysis is non-deterministic and may contain factual errors, hallucinated data, or flawed reasoning.
> - Always consult a qualified, licensed financial advisor before making any investment decision.
> - The authors and contributors of this skill accept no liability for any losses incurred from acting on its output.

---

## 10. Credits

- Based on [TauricResearch/TradingAgents](https://github.com/TauricResearch/TradingAgents) (Apache 2.0).
- China market enhancements from [hsliuping/TradingAgents-CN](https://github.com/hsliuping/TradingAgents-CN).
- Paper: "TradingAgents: Multi-Agents LLM Financial Trading Framework" ([arXiv:2412.20138](https://arxiv.org/abs/2412.20138)).
