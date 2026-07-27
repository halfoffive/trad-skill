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
yfinance

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

Data fetching is implemented in Rust (`trad-data` binary) for US/HK/Crypto markets.
China A-share market uses Python scripts (akshare) as fallback.

### Rust Binary: trad-data
- US stocks: Yahoo Finance v8/v10 API (direct HTTP)
- HK stocks: Eastmoney push2his API (direct HTTP)
- Crypto: Yahoo Finance API (same as US stocks)
- News: Yahoo Finance + Google News RSS
- Sentiment: StockTwits + Reddit JSON API

### Python Fallback (China A-shares)
- A-share OHLCV: AKShare (Eastmoney API wrapper)
- A-share fundamentals: AKShare (Sina Finance + Eastmoney)
- A-share news: AKShare (Eastmoney search API)
- A-share sentiment: AKShare (Eastmoney comment API)
