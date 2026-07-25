#!/usr/bin/env python3
"""
股票行情数据获取工具

支持美股、A股、港股、加密货币的OHLCV数据获取。
函数式编程风格，无类定义。
用于 TradingAgents 多智能体分析流水线的数据采集阶段。

依赖: pip install yfinance akshare pandas

降本增效设计：
- 默认只输出最近 N 行 OHLCV（--tail，默认 30），避免整段原始 CSV 进入提示词。
- 默认用纯 pandas 预计算技术指标（--indicators，默认开），
  输出紧凑指标快照表，让大模型只做解读而非手算。
- 可选 --stats 输出区间统计（收益率、波动率、均量、52 周高低）。
"""

# 所有注释用中文
# 函数式编程：每个函数独立，无全局状态
# 返回值：格式化的字符串（适合注入LLM提示词）

import argparse
from datetime import date, timedelta

import pandas as pd
import yfinance as yf

# 尝试导入 akshare，部分环境可能未安装
try:
    import akshare as ak
except ImportError:
    ak = None


def _normalize_ohlcv(df: pd.DataFrame) -> pd.DataFrame:
    """将 OHLCV DataFrame 规整为统一列名：Date/Open/High/Low/Close/Volume。"""
    # 只保留核心列，日期列重置为普通列
    if isinstance(df.index, pd.DatetimeIndex) or "Date" not in df.columns:
        df = df.reset_index()
    rename = {
        "日期": "Date",
        "开盘": "Open",
        "最高": "High",
        "最低": "Low",
        "收盘": "Close",
        "成交量": "Volume",
    }
    df = df.rename(columns=rename)
    if "Date" in df.columns:
        df["Date"] = pd.to_datetime(df["Date"]).dt.strftime("%Y-%m-%d")
    columns = ["Date", "Open", "High", "Low", "Close", "Volume"]
    available = [c for c in columns if c in df.columns]
    return df[available].copy()


def _dataframe_to_csv(df: pd.DataFrame) -> str:
    """将 DataFrame 转换为 CSV 格式字符串，用于注入 LLM 提示词。"""
    return df.to_csv(index=False)


def compute_indicators(df: pd.DataFrame) -> str:
    """
    用纯 pandas 计算技术指标，输出紧凑快照表。

    指标：SMA(50/200)、EMA(10)、MACD/信号/柱、RSI(14)、
    Bollinger(20,2) 中轨与上下轨、ATR(14)、VWMA(20)、MFI(14)。

    参数:
        df: 含 Open/High/Low/Close/Volume 列的 DataFrame
    返回:
        紧凑指标快照 markdown 字符串
    """
    # 缺少必要列时直接返回提示，不抛异常
    need = ["High", "Low", "Close", "Volume"]
    if not all(c in df.columns for c in need):
        return "## 技术指标\n\n> 数据列不全，无法计算指标。\n"

    try:
        close = df["Close"].astype(float)
        high = df["High"].astype(float)
        low = df["Low"].astype(float)
        volume = df["Volume"].astype(float)

        # 移动平均
        sma50 = close.rolling(50).mean()
        sma200 = close.rolling(200).mean()
        ema10 = close.ewm(span=10, adjust=False).mean()

        # MACD：12/26 EMA 差值，信号线为 9 EMA
        ema12 = close.ewm(span=12, adjust=False).mean()
        ema26 = close.ewm(span=26, adjust=False).mean()
        macd = ema12 - ema26
        signal = macd.ewm(span=9, adjust=False).mean()
        hist = macd - signal

        # RSI(14)：Wilder 平滑法
        delta = close.diff()
        gain = delta.clip(lower=0)
        loss = -delta.clip(upper=0)
        avg_gain = gain.ewm(alpha=1 / 14, min_periods=14, adjust=False).mean()
        avg_loss = loss.ewm(alpha=1 / 14, min_periods=14, adjust=False).mean()
        rs = avg_gain / avg_loss.replace(0, pd.NA)
        rsi = 100 - 100 / (1 + rs)

        # Bollinger(20, 2)
        boll_mid = close.rolling(20).mean()
        boll_std = close.rolling(20).std()
        boll_ub = boll_mid + 2 * boll_std
        boll_lb = boll_mid - 2 * boll_std

        # ATR(14)：Wilder
        prev_close = close.shift(1)
        tr = pd.concat(
            [
                high - low,
                (high - prev_close).abs(),
                (low - prev_close).abs(),
            ],
            axis=1,
        ).max(axis=1)
        atr = tr.ewm(alpha=1 / 14, min_periods=14, adjust=False).mean()

        # VWMA(20)：成交量加权移动平均
        vwma = (close * volume).rolling(20).sum() / volume.rolling(20).sum()

        # MFI(14)：资金流量指标
        tp = (high + low + close) / 3
        mf = tp * volume
        pos = mf.where(tp > tp.shift(1), 0.0)
        neg = mf.where(tp < tp.shift(1), 0.0)
        pos_sum = pos.rolling(14).sum()
        neg_sum = neg.rolling(14).sum()
        mfr = pos_sum / neg_sum.replace(0, pd.NA)
        mfi = 100 - 100 / (1 + mfr)

        # 取最后一行的最新值
        last = len(df) - 1
        px = close.iloc[last]

        def _val(series):
            v = series.iloc[last]
            return round(float(v), 4) if pd.notna(v) else "N/A"

        # 趋势信号判定
        # 金叉/死叉：SMA50 vs SMA200
        cross = "N/A"
        if pd.notna(sma50.iloc[last]) and pd.notna(sma200.iloc[last]):
            if sma50.iloc[last] > sma200.iloc[last]:
                cross = "金叉(多头排列)"
            else:
                cross = "死叉(空头排列)"
        # RSI 超买/超卖
        rsi_state = "中性"
        if pd.notna(rsi.iloc[last]):
            if rsi.iloc[last] >= 70:
                rsi_state = "超买"
            elif rsi.iloc[last] <= 30:
                rsi_state = "超卖"
        # 布林位置
        boll_pos = "中轨附近"
        if pd.notna(boll_ub.iloc[last]) and pd.notna(boll_lb.iloc[last]):
            if px >= boll_ub.iloc[last]:
                boll_pos = "触及/突破上轨(超买区)"
            elif px <= boll_lb.iloc[last]:
                boll_pos = "触及/跌破下轨(超卖区)"
        # MACD 方向
        macd_state = "N/A"
        if pd.notna(macd.iloc[last]) and pd.notna(signal.iloc[last]):
            macd_state = "MACD>信号(多头)" if macd.iloc[last] > signal.iloc[last] else "MACD<信号(空头)"

        # 组装紧凑指标快照表
        lines = [
            "## 技术指标快照（脚本预计算）\n",
            "| 指标 | 最新值 | 信号 |",
            "|---|---|---|",
            f"| 收盘价 | {round(float(px), 4)} | — |",
            f"| SMA50 / SMA200 | {_val(sma50)} / {_val(sma200)} | {cross} |",
            f"| EMA10 | {_val(ema10)} | 短期动能 |",
            f"| MACD / 信号 / 柱 | {_val(macd)} / {_val(signal)} / {_val(hist)} | {macd_state} |",
            f"| RSI(14) | {_val(rsi)} | {rsi_state} |",
            f"| Boll 中轨/上轨/下轨 | {_val(boll_mid)} / {_val(boll_ub)} / {_val(boll_lb)} | {boll_pos} |",
            f"| ATR(14) | {_val(atr)} | 波动率参考 |",
            f"| VWMA(20) | {_val(vwma)} | 量价趋势 |",
            f"| MFI(14) | {_val(mfi)} | 资金流向 |",
        ]
        return "\n".join(lines) + "\n"
    except Exception as e:
        # 计算失败返回错误信息，不抛异常
        return f"## 技术指标\n\n> 指标计算失败: {e}\n"


def compute_stats(df: pd.DataFrame) -> str:
    """
    输出区间统计：区间收益率、年化波动率、均量、52 周高低。

    参数:
        df: 含 OHLCV 列的 DataFrame
    返回:
        紧凑统计 markdown 字符串
    """
    if "Close" not in df.columns or df.empty:
        return "## 区间统计\n\n> 数据不足。\n"
    try:
        close = df["Close"].astype(float)
        first = close.iloc[0]
        last = close.iloc[-1]
        ret = (last / first - 1) * 100 if first else float("nan")
        # 日对数收益 → 年化波动率（252 交易日）
        daily_ret = close.pct_change().dropna()
        vol = daily_ret.std() * (252 ** 0.5) * 100 if len(daily_ret) > 1 else float("nan")
        avg_vol = df["Volume"].astype(float).mean() if "Volume" in df.columns else float("nan")
        # 52 周高低：取最近约 252 个交易日
        window = close.tail(252) if len(close) >= 252 else close
        hi = window.max()
        lo = window.min()
        lines = [
            "## 区间统计\n",
            f"- 区间收益率: {round(float(ret), 2)}%",
            f"- 年化波动率: {round(float(vol), 2)}%",
            f"- 日均成交量: {int(avg_vol) if pd.notna(avg_vol) else 'N/A'}",
            f"- 52周(或区间)高/低: {round(float(hi), 4)} / {round(float(lo), 4)}",
        ]
        return "\n".join(lines) + "\n"
    except Exception as e:
        return f"## 区间统计\n\n> 统计计算失败: {e}\n"


def fetch_us_stock_data(symbol: str, start_date: str, end_date: str) -> str:
    """
    获取美股 OHLCV 数据。

    使用 yfinance 获取指定时间范围的日线数据，
    返回 CSV 格式字符串。网络失败时返回错误信息。

    参数:
        symbol: 美股代码，如 "AAPL", "MSFT"
        start_date: 起始日期，格式 "YYYY-MM-DD"
        end_date: 结束日期，格式 "YYYY-MM-DD"
    返回:
        CSV 格式字符串或错误信息
    """
    # 使用 yfinance 获取美股历史行情
    try:
        ticker = yf.Ticker(symbol)
        df = ticker.history(start=start_date, end=end_date)
        # 检查是否获取到数据
        if df.empty:
            return f"错误: 未获取到 {symbol} 在 {start_date} 至 {end_date} 的数据，请检查代码和日期范围。"
        df = _normalize_ohlcv(df)
        return _dataframe_to_csv(df)
    except Exception as e:
        # 网络异常或其他错误，返回错误信息而非抛异常
        return f"错误: 获取美股 {symbol} 数据失败 - {e}"


def fetch_cn_stock_data(symbol: str, start_date: str, end_date: str) -> str:
    """
    获取A股 OHLCV 数据。

    优先使用 akshare 的 stock_zh_a_hist 接口，
    失败时降级为 yfinance（自动添加 .SS 或 .SZ 后缀）。

    参数:
        symbol: A股代码，纯6位数字如 "600519"
        start_date: 起始日期，格式 "YYYY-MM-DD"
        end_date: 结束日期，格式 "YYYY-MM-DD"
    返回:
        CSV 格式字符串或错误信息
    """
    # 将日期格式从 YYYY-MM-DD 转为 akshare 需要的 YYYYMMDD
    ak_start = start_date.replace("-", "")
    ak_end = end_date.replace("-", "")

    # 优先尝试 akshare 获取A股数据
    if ak is not None:
        try:
            df = ak.stock_zh_a_hist(
                symbol=symbol,
                period="daily",
                start_date=ak_start,
                end_date=ak_end,
                adjust="qfq",
            )
            if df is not None and not df.empty:
                df = _normalize_ohlcv(df)
                return _dataframe_to_csv(df)
        except Exception:
            # akshare 失败，继续降级到 yfinance
            pass

    # 降级方案：使用 yfinance，根据代码前缀判断交易所
    # 6开头为上海（.SS），0/3开头为深圳（.SZ）
    if symbol.startswith("6"):
        yf_symbol = f"{symbol}.SS"
    else:
        yf_symbol = f"{symbol}.SZ"

    try:
        ticker = yf.Ticker(yf_symbol)
        df = ticker.history(start=start_date, end=end_date)
        if df.empty:
            return f"错误: 未获取到A股 {symbol} 在 {start_date} 至 {end_date} 的数据。"
        df = _normalize_ohlcv(df)
        return _dataframe_to_csv(df)
    except Exception as e:
        return f"错误: 获取A股 {symbol} 数据失败（akshare 和 yfinance 均不可用）- {e}"


def fetch_hk_stock_data(symbol: str, start_date: str, end_date: str) -> str:
    """
    获取港股 OHLCV 数据。

    优先使用 akshare 的 stock_hk_hist 接口，
    失败时降级为 yfinance（添加 .HK 后缀）。

    参数:
        symbol: 港股代码，5位数字如 "00700"
        start_date: 起始日期，格式 "YYYY-MM-DD"
        end_date: 结束日期，格式 "YYYY-MM-DD"
    返回:
        CSV 格式字符串或错误信息
    """
    # 将日期格式从 YYYY-MM-DD 转为 akshare 需要的 YYYYMMDD
    ak_start = start_date.replace("-", "")
    ak_end = end_date.replace("-", "")

    # 优先尝试 akshare 获取港股数据
    if ak is not None:
        try:
            df = ak.stock_hk_hist(
                symbol=symbol,
                period="daily",
                start_date=ak_start,
                end_date=ak_end,
                adjust="qfq",
            )
            if df is not None and not df.empty:
                df = _normalize_ohlcv(df)
                return _dataframe_to_csv(df)
        except Exception:
            # akshare 失败，降级到 yfinance
            pass

    # 降级方案：使用 yfinance 加 .HK 后缀
    yf_symbol = f"{symbol}.HK"
    try:
        ticker = yf.Ticker(yf_symbol)
        df = ticker.history(start=start_date, end=end_date)
        if df.empty:
            return f"错误: 未获取到港股 {symbol} 在 {start_date} 至 {end_date} 的数据。"
        df = _normalize_ohlcv(df)
        return _dataframe_to_csv(df)
    except Exception as e:
        return f"错误: 获取港股 {symbol} 数据失败（akshare 和 yfinance 均不可用）- {e}"


def fetch_stock_df(symbol: str, start_date: str, end_date: str) -> pd.DataFrame:
    """
    统一获取 OHLCV DataFrame（内部用，供指标/统计复用）。
    失败时返回空 DataFrame。
    """
    # 去除首尾空格
    symbol = symbol.strip()
    # 复用各市场抓取逻辑，再把 CSV 解析回 DataFrame
    csv_text = fetch_stock_data(symbol, start_date, end_date)
    if csv_text.startswith("错误"):
        return pd.DataFrame()
    try:
        from io import StringIO

        return pd.read_csv(StringIO(csv_text))
    except Exception:
        return pd.DataFrame()


def fetch_stock_data(symbol: str, start_date: str, end_date: str) -> str:
    """
    统一股票数据获取入口，自动检测市场类型。

    检测逻辑:
        - 6位纯数字 → A股
        - 以 .HK 结尾或5位纯数字 → 港股
        - 以 -USD 结尾 → 加密货币（用 yfinance）
        - 其他 → 美股

    参数:
        symbol: 股票/加密货币代码
        start_date: 起始日期，格式 "YYYY-MM-DD"
        end_date: 结束日期，格式 "YYYY-MM-DD"
    返回:
        CSV 格式字符串或错误信息
    """
    # 去除首尾空格
    symbol = symbol.strip()

    # 判断是否为加密货币（以 -USD 结尾）
    if symbol.upper().endswith("-USD"):
        return fetch_us_stock_data(symbol, start_date, end_date)

    # 判断是否为港股（以 .HK 结尾）
    if symbol.upper().endswith(".HK"):
        # 去掉 .HK 后缀，传入纯数字代码
        hk_code = symbol[:-3].zfill(5)
        return fetch_hk_stock_data(hk_code, start_date, end_date)

    # 判断是否为纯数字代码
    if symbol.isdigit():
        # 6位数字 → A股
        if len(symbol) == 6:
            return fetch_cn_stock_data(symbol, start_date, end_date)
        # 5位数字 → 港股
        if len(symbol) == 5:
            return fetch_hk_stock_data(symbol, start_date, end_date)

    # 默认按美股处理
    return fetch_us_stock_data(symbol, start_date, end_date)


def build_compact_report(symbol: str, start_date: str, end_date: str, tail: int,
                         indicators: bool, stats: bool) -> str:
    """
    构建精简报告：OHLCV tail + (可选)统计 + (可选)指标快照。
    替代整段原始 CSV，大幅降低注入提示词的 token 量。
    """
    # 钳制负数 tail，避免 df.tail() 抛 ValueError（契约：函数不抛异常）
    tail = max(0, int(tail))
    # 仅触发一次网络请求，拿到完整 CSV 文本
    csv_text = fetch_stock_data(symbol, start_date, end_date)
    # 抓取失败直接返回错误信息，避免重复请求
    if csv_text.startswith("错误"):
        return csv_text
    # 解析 CSV 为 DataFrame（指标/统计需要历史窗口）
    try:
        from io import StringIO

        df = pd.read_csv(StringIO(csv_text))
    except Exception:
        return csv_text
    if df.empty:
        return csv_text

    sections: list[str] = [f"# {symbol} 行情（{start_date} 至 {end_date}）\n"]

    # 统计与指标需要完整历史窗口，先用完整 df 计算
    if stats:
        sections.append(compute_stats(df))
    if indicators:
        sections.append(compute_indicators(df))

    # 尾部 OHLCV（仅展示最近 tail 行，供分析师核对价位）
    tail_df = df.tail(tail)
    sections.append(f"## 最近 {len(tail_df)} 行 OHLCV\n")
    sections.append("```csv")
    sections.append(_dataframe_to_csv(tail_df).strip())
    sections.append("```\n")

    return "\n".join(sections)


if __name__ == "__main__":
    # 命令行参数解析
    parser = argparse.ArgumentParser(
        description="股票行情数据获取工具 - 支持美股/A股/港股/加密货币（默认输出精简指标快照）"
    )
    parser.add_argument(
        "--symbol",
        type=str,
        required=True,
        help="股票代码，如 AAPL、600519、00700、BTC-USD",
    )
    parser.add_argument(
        "--start",
        type=str,
        default=None,
        help="起始日期，格式 YYYY-MM-DD（默认：今天往前 365 天，确保覆盖 SMA200 所需的 ~200 个交易日）",
    )
    parser.add_argument(
        "--end",
        type=str,
        default=None,
        help="结束日期，格式 YYYY-MM-DD（默认：今天）",
    )
    parser.add_argument(
        "--tail",
        type=int,
        default=30,
        help="只输出最近 N 行 OHLCV（默认 30，避免整段原始 CSV 进入提示词）",
    )
    parser.add_argument(
        "--indicators",
        dest="indicators",
        action="store_true",
        default=True,
        help="预计算技术指标快照（默认开启）",
    )
    parser.add_argument(
        "--no-indicators",
        dest="indicators",
        action="store_false",
        help="关闭技术指标预计算",
    )
    parser.add_argument(
        "--stats",
        dest="stats",
        action="store_true",
        default=False,
        help="输出区间统计（收益率/波动率/均量/52周高低）",
    )
    parser.add_argument(
        "--raw",
        action="store_true",
        default=False,
        help="输出整段原始 CSV（兼容旧行为，token 开销大，慎用）",
    )

    args = parser.parse_args()

    # 默认日期窗口：--end 缺省=今天，--start 缺省=今天往前 365 天
    # （SKILL.md §6 指引：至少需 200 个交易日才能算 SMA200，1 年足够）
    end_date = args.end if args.end else date.today().isoformat()
    start_date = args.start if args.start else (date.today() - timedelta(days=365)).isoformat()

    if args.raw:
        # 旧行为：整段原始 CSV
        result = fetch_stock_data(args.symbol, start_date, end_date)
    else:
        # 默认：精简报告（指标 + 尾部 OHLCV）
        result = build_compact_report(
            args.symbol, start_date, end_date,
            tail=args.tail, indicators=args.indicators, stats=args.stats,
        )
    print(result)
