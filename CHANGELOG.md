# Changelog

All notable changes to this project will be documented in this file.

## [1.0.0] - 2026-07-22

### Added
- SKILL.md: Multi-agent trading analysis pipeline with sub-agent orchestration
- 14 verbatim agent role prompts in `references/prompts/` (from TradingAgents + TradingAgents-CN)
- Data source catalog (`references/data-sources.md`): 12 sources covering US, A-share, and HK markets
- Technical indicator reference (`references/indicators.md`): 13 indicators across 5 categories
- Python data-fetching scripts (functional style, Chinese comments):
  - `fetch_stock_data.py` — OHLCV via yfinance + akshare
  - `fetch_news.py` — news via yfinance + Google News RSS + akshare
  - `fetch_fundamentals.py` — financials via yfinance + akshare
  - `fetch_sentiment.py` — sentiment via StockTwits + Reddit + akshare
- Bilingual documentation: README.md (English) + README_CN.md (中文)
- AGENTS.md for AI agent onboarding
- Language switch buttons in both READMEs
- `npx skill add` one-click install support

### Design Decisions
- Skill files live in `tradingagents-analysis/` subfolder (clean repo root for meta files)
- Prompts are verbatim extracts — never paraphrased from memory
- Scripts return formatted strings (for LLM prompt injection), never raise exceptions
- No class-based Python; functional style throughout
