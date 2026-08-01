# Data Sources Catalog

Complete catalog of data sources used in the TradingAgents multi-agent analysis pipeline.

## US Market Sources

### Yahoo Finance (yfinance)
- **Provides**: Stock price data (OHLCV), company fundamentals, financial statements, news headlines
- **API Key**: Not required (free)
- **Markets**: US, HK, global exchanges
- **Used by**: Market Analyst, Fundamentals Analyst, News Analyst
- **Fallback**: Alpha Vantage (not wired in scripts; agent uses web search fallback)

### Alpha Vantage
- **Provides**: Stock data, technical indicators, fundamental data, news
- **API Key**: Required (ALPHA_VANTAGE_API_KEY, free tier available)
- **Markets**: US primarily
- **Used by**: Market Analyst, Fundamentals Analyst, News Analyst (not wired in scripts; agent uses web search fallback)
- **Fallback**: yfinance

### FRED (Federal Reserve Economic Data)
- **Provides**: Macroeconomic indicators (CPI, Core PCE, unemployment, fed funds rate, 10Y treasury, yield curve)
- **API Key**: Required (FRED_API_KEY, free)
- **Markets**: US macro
- **Used by**: News Analyst (macro context) (not wired in scripts; agent uses web search fallback)
- **Indicators available**: cpi, core_pce, unemployment, fed_funds_rate, 10y_treasury, yield_curve

### Polymarket
- **Provides**: Prediction market probabilities for forward-looking events
- **API Key**: Not required (keyless)
- **Markets**: Global events
- **Used by**: News Analyst (not wired in scripts; agent uses web search fallback)
- **Example queries**: "Fed rate cut", "recession 2026", geopolitical events

### StockTwits
- **Provides**: Retail trader sentiment messages with Bullish/Bearish tags
- **API Key**: Not required (public API)
- **Markets**: US stocks (by cashtag)
- **Used by**: Sentiment Analyst
- **Signal**: Bullish/Bearish ratio as leading retail-sentiment indicator

### Reddit
- **Provides**: Community discussion posts with engagement metrics (upvotes, comments)
- **API Key**: Not required (public JSON API)
- **Subreddits**: r/wallstreetbets, r/stocks, r/investing
- **Used by**: Sentiment Analyst
- **Signal**: Engagement-weighted community attention

## China Market Sources (A股/港股)

### Tushare
- **Provides**: A股 daily/weekly/monthly data, financial statements, fundamentals
- **API Key**: Required (TUSHARE_TOKEN)
- **Markets**: China A-shares (Shanghai .SS, Shenzhen .SZ)
- **Used by**: China Market Analyst, Fundamentals Analyst (not wired in scripts; agent uses web search fallback)
- **Priority**: Primary for A-shares

### AKShare
- **Provides**: A股 real-time quotes, historical data, financial indicators, stock comments
- **API Key**: Not required (free, open source)
- **Markets**: China A-shares, HK stocks
- **Used by**: China Market Analyst, Sentiment Analyst (CN)
- **Uses**: stock_zh_a_hist

### Baostock
- **Provides**: A股 historical K-line data, financial reports
- **API Key**: Not required (free)
- **Markets**: China A-shares
- **Used by**: China Market Analyst (fallback) (not wired in scripts; agent uses web search fallback)
- **Priority**: Tertiary fallback

### TDX / 通达信
- **Provides**: Technical indicators, real-time quotes
- **API Key**: Not required (local data)
- **Markets**: China A-shares
- **Used by**: China Market Analyst (technical analysis) (not wired in scripts; agent uses web search fallback)

### Google News (Chinese)
- **Provides**: Chinese financial news articles
- **API Key**: Not required
- **Markets**: A-shares, HK stocks
- **Used by**: CN News Analyst
- **Note**: Adds Chinese keywords (股票, 公司, 财报) for A-share queries

### Chinese Finance Social Sentiment
- **Provides**: Chinese social media sentiment analysis
- **API Key**: Not required
- **Markets**: A-shares
- **Used by**: Sentiment Analyst (CN)

## Data Source Degradation Chains

### A-Share Data (CN)
AKShare → yfinance

### US Stock Data
yfinance (Yahoo Finance) → Eastmoney push2his (via `stock --source eastmoney`, for Yahoo-blocked regions)

### HK Stock Data
AKShare → yfinance

### News (US)
yfinance + Google News

### News (CN)
AKShare news (stock_news_em / 东方财富) → Google News (Chinese)

## Configuration

> **Note**: This configuration system belongs to the original TradingAgents framework. This skill's scripts do NOT read `data_vendors` / `tool_vendors` / API key env vars (except akshare which needs no key).

Data sources are configured via:
- Environment variables (API keys)
- `data_vendors` config dict (category-level vendor selection)
- `tool_vendors` config dict (tool-level override)

| Category | Default Vendor | Options |
|----------|---------------|---------|
| core_stock_apis | yfinance | alpha_vantage, yfinance |
| technical_indicators | yfinance | alpha_vantage, yfinance |
| fundamental_data | yfinance | alpha_vantage, yfinance |
| news_data | yfinance | alpha_vantage, yfinance |
| macro_data | fred | fred |
| prediction_markets | polymarket | polymarket |

## Implementation Notes

Data fetching is implemented in Rust (`trad-skill` binary) for all markets including US, HK, Crypto, and China A-shares.

### Rust Binary: trad-skill
- US stocks: Yahoo Finance v8 API (default; browser User-Agent + cookie/crumb handshake) — with an **Eastmoney push2his fallback channel** selectable via `stock --source eastmoney` (tries secid `105`/`106`/`107` = NASDAQ/NYSE/AMEX)
- HK stocks: Eastmoney push2his API (direct HTTP)
- Crypto: Yahoo Finance API (same as US stocks)
- Fundamentals: Yahoo Finance quoteSummary v10 (browser UA + cookie/crumb handshake) for US/Crypto; **Eastmoney** push2 + datacenter for A-shares **and HK stocks** (secid `116.{code}`; financial-indicator table gracefully degrades to "暂不可用" if Eastmoney has no HK rows)
- News: Yahoo Finance quoteSummary v10 (browser UA + cookie/crumb handshake) + Google News RSS in parallel (US); Eastmoney/Google News (CN)
- Sentiment: StockTwits + Reddit JSON API
- China A-shares: Eastmoney APIs (via Rust HTTP) — stock, fundamentals, and news all auto-route here, no Yahoo dependency

### Data channel selection (`stock --source`)

`trad-skill stock` auto-selects the channel from the symbol: US/Crypto → Yahoo Finance, A-share/HK → Eastmoney. The `--source` flag overrides this:

| `--source` | US stocks | A-shares | HK stocks | Crypto |
|---|---|---|---|---|
| _(omitted, auto)_ | Yahoo | Eastmoney | Eastmoney | Yahoo |
| `yahoo` | Yahoo | Yahoo (`.SS`/`.SZ`) | Yahoo (`.HK`) | Yahoo |
| `eastmoney` | Eastmoney (105/106/107) | Eastmoney | Eastmoney | _not supported_ |

Use `--source eastmoney` for US stocks when Yahoo Finance is unreachable in your region (typical symptoms: `Yahoo Finance 错误(...): 未知错误` or HTTP 403 from datacenter/cloud IPs).
