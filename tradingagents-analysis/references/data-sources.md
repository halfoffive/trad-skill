# Data Sources Catalog

Complete catalog of data sources used in the TradingAgents multi-agent analysis pipeline.

## US Market Sources

### Yahoo Finance (yfinance)
- **Provides**: Stock price data (OHLCV), company fundamentals, financial statements, news headlines
- **API Key**: Not required (free)
- **Markets**: US, HK, global exchanges
- **Used by**: Market Analyst, Fundamentals Analyst, News Analyst
- **Fallback**: Alpha Vantage

### Alpha Vantage
- **Provides**: Stock data, technical indicators, fundamental data, news
- **API Key**: Required (ALPHA_VANTAGE_API_KEY, free tier available)
- **Markets**: US primarily
- **Used by**: Market Analyst, Fundamentals Analyst, News Analyst
- **Fallback**: yfinance

### FRED (Federal Reserve Economic Data)
- **Provides**: Macroeconomic indicators (CPI, Core PCE, unemployment, fed funds rate, 10Y treasury, yield curve)
- **API Key**: Required (FRED_API_KEY, free)
- **Markets**: US macro
- **Used by**: News Analyst (macro context)
- **Indicators available**: cpi, core_pce, unemployment, fed_funds_rate, 10y_treasury, yield_curve

### Polymarket
- **Provides**: Prediction market probabilities for forward-looking events
- **API Key**: Not required (keyless)
- **Markets**: Global events
- **Used by**: News Analyst
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
- **Used by**: China Market Analyst, Fundamentals Analyst
- **Priority**: Primary for A-shares

### AKShare
- **Provides**: A股 real-time quotes, historical data, financial indicators, stock comments
- **API Key**: Not required (free, open source)
- **Markets**: China A-shares, HK stocks
- **Used by**: China Market Analyst, Sentiment Analyst (CN)
- **Fallback chain**: stock_bid_ask_em → stock_zh_a_spot → stock_zh_a_spot_em → stock_zh_a_hist

### Baostock
- **Provides**: A股 historical K-line data, financial reports
- **API Key**: Not required (free)
- **Markets**: China A-shares
- **Used by**: China Market Analyst (fallback)
- **Priority**: Tertiary fallback

### TDX / 通达信
- **Provides**: Technical indicators, real-time quotes
- **API Key**: Not required (local data)
- **Markets**: China A-shares
- **Used by**: China Market Analyst (technical analysis)

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
MongoDB cache → Tushare → AKShare → Baostock → TDX

### US Stock Data
yfinance → Alpha Vantage

### HK Stock Data
AKShare → yfinance

### News (US)
Yahoo Finance News → Alpha Vantage News → Google News

### News (CN)
Unified news tool (auto-detects market type) → Google News (Chinese) → AKShare news

## Configuration

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
