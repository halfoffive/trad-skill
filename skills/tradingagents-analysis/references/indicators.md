# Technical Indicators Reference

Indicators available to the Market Analyst, organized by category.
The `trad-skill stock` binary pre-computes a snapshot of 8 indicators; the Market Analyst **interprets** these pre-computed values.

> **These indicators are pre-computed by the binary.** Run `trad-skill stock --symbol <ticker> --start <s> --end <e>` and the output already contains a compact indicator snapshot (latest values + trend signals). The Market Analyst **interprets** these pre-computed values — it does not call a separate tool or recompute them by hand. There is no `get_stock_data` / `get_indicators` tool; the `trad-skill` binary is the data source.

## Selection Guidelines
- Interpret indicators that provide diverse and complementary information
- Avoid redundancy (e.g., do not select both RSI and StochRSI)
- Explain why each indicator is suitable for the given market context
- The binary fetches the OHLCV and computes the indicators in one call

## Moving Averages

### SMA50 / SMA200 — 50/200 SMA
A medium-term trend indicator.
- **Usage**: Identify trend direction and serve as dynamic support/resistance.
- **Tips**: It lags price; combine with faster indicators for timely signals.
- **Snapshot row**: `SMA50 / SMA200` — the `信号` column reports 金叉(多头排列) when SMA50 > SMA200, 死叉(空头排列) otherwise.

### EMA10 — 10 EMA
A responsive short-term average.
- **Usage**: Capture quick shifts in momentum and potential entry points.
- **Tips**: Prone to noise in choppy markets; use alongside longer averages for filtering false signals.

## MACD Related

### MACD / 信号 / 柱 — MACD
Computes momentum via differences of EMAs.
- **Usage**: Look for crossovers and divergence as signals of trend changes.
- **Tips**: Confirm with other indicators in low-volatility or sideways markets.
- **Snapshot row**: `MACD / 信号 / 柱` — the `信号` column reports 多头 (MACD > 信号) or 空头.

## Momentum Indicators

### RSI(14) — RSI
Measures momentum to flag overbought/oversold conditions.
- **Usage**: Apply 70/30 thresholds and watch for divergence to signal reversals.
- **Tips**: In strong trends, RSI may remain extreme; always cross-check with trend analysis.
- **Snapshot row**: `RSI(14)` — the `信号` column reports 超买 (≥70), 超卖 (≤30), or 中性.

## Volatility Indicators

### Boll 中轨 — Bollinger Middle
A 20 SMA serving as the basis for Bollinger Bands.
- **Usage**: Acts as a dynamic benchmark for price movement.
- **Tips**: Combine with the upper and lower bands to effectively spot breakouts or reversals.

### Boll 上轨 — Bollinger Upper Band
Typically 2 standard deviations above the middle line.
- **Usage**: Signals potential overbought conditions and breakout zones.
- **Tips**: Confirm signals with other tools; prices may ride the band in strong trends.

### Boll 下轨 — Bollinger Lower Band
Typically 2 standard deviations below the middle line.
- **Usage**: Indicates potential oversold conditions.
- **Tips**: Use additional analysis to avoid false reversal signals.
- **Snapshot row**: `Boll 中轨/上轨/下轨` — the `信号` column reports 触及/突破上轨(超买区), 触及/跌破下轨(超卖区), or 中轨附近.

### ATR(14) — ATR
Averages true range to measure volatility.
- **Usage**: Set stop-loss levels and adjust position sizes based on current market volatility.
- **Tips**: It's a reactive measure, so use it as part of a broader risk management strategy.

## Volume-Based Indicators

### VWMA(20) — VWMA
A moving average weighted by volume.
- **Usage**: Confirm trends by integrating price action with volume data.
- **Tips**: Watch for skewed results from volume spikes; use in combination with other volume analyses.

### MFI(14) — MFI (Money Flow Index)
A momentum indicator that uses both price and volume to measure buying and selling pressure.
- **Usage**: Identify overbought (>80) or oversold (<20) conditions and confirm the strength of trends or reversals.
- **Tips**: Use alongside RSI or MACD to confirm signals; divergence between price and MFI can indicate potential reversals.

## Verified Market Snapshot

The `trad-skill stock` output is the source of truth for any exact OHLCV, price-level, or indicator-value claim. If a web-search fallback conflicts with the binary output, flag the discrepancy rather than inventing a reconciled number. Do not claim historical validation, support/resistance bounces, or exact percentage moves unless they are directly supported by the binary output with concrete dates and prices.

## Note on MFI (R6-23)

The source repo's `market_analyst.py` indicator list (verbatim in `references/prompts/market_analyst.md`) does **not** include MFI — trad-skill inherits this verbatim. However, the `trad-skill stock` binary **does** pre-compute MFI(14) and include it in the snapshot table. The Market Analyst should treat the binary's MFI row as a supplementary indicator (volume-weighted momentum, >80 overbought / <20 oversold), interpreting it alongside RSI/MACD as documented in the `MFI(14)` section above. This verbatim-vs-binary mismatch is documented as a known limitation; do not modify `market_analyst.md` (verbatim constraint).

## Note on RSI / Bollinger implementation details (R6-5, R6-7, R6-8)

- **RSI(14)** uses an `ewm(adjust=False, alpha=1/14)` Wilder smoothing approximation (`indicators.rs`). The standard Wilder RSI seeds with `mean(gain[1:15])` (14-period SMA); the ewm approximation seeds with `gain[0]`. Bias is ~1pp near the 30/70 thresholds — cross-check with MACD/CCI when RSI is near a threshold before declaring overbought/oversold. Continuous-up periods (avg_loss==0) correctly return RSI=100 (not NA); all-flat periods return 50.
- **Bollinger Bands(20, 2)** use population standard deviation (`ddof=0`), matching the StockCharts/TradingView convention. Sample std (`ddof=1`, pandas default) would make bands ~2.6% wider.
- **MFI(14)** handles 0/0 and X/0 edge cases explicitly: all-flat → 50 (neutral); continuous net inflow → 100; continuous net outflow → 0.
