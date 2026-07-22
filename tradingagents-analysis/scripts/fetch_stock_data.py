#!/usr/bin/env python3
"""
股票行情数据获取工具

支持美股、A股、港股的OHLCV数据获取。
函数式编程风格，无类定义。
用于 TradingAgents 多智能体分析流水线的数据采集阶段。

依赖: pip install yfinance akshare pandas
"""

# 所有注释用中文
# 函数式编程：每个函数独立，无全局状态
# 返回值：格式化的字符串（适合注入LLM提示词）

import argparse
import sys

import pandas as pd
import yfinance as yf

# 尝试导入 akshare，部分环境可能未安装
try:
    import akshare as ak
except ImportError:
    ak = None


def _dataframe_to_csv(df: pd.DataFrame) -> str:
    """将 DataFrame 转换为 CSV 格式字符串，用于注入 LLM 提示词。"""
    # 只保留 OHLCV 核心列
    columns = ["Date", "Open", "High", "Low", "Close", "Volume"]
    available = [c for c in columns if c in df.columns]
    return df[available].to_csv(index=False)


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
        # 重置索引，将日期变为普通列
        df = df.reset_index()
        # 只保留日期部分，去掉时间
        df["Date"] = pd.to_datetime(df["Date"]).dt.strftime("%Y-%m-%d")
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
                # akshare 返回的列名是中文，需要重命名
                rename_map = {
                    "日期": "Date",
                    "开盘": "Open",
                    "最高": "High",
                    "最低": "Low",
                    "收盘": "Close",
                    "成交量": "Volume",
                }
                df = df.rename(columns=rename_map)
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
        df = df.reset_index()
        df["Date"] = pd.to_datetime(df["Date"]).dt.strftime("%Y-%m-%d")
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
                # akshare 港股返回的列名也是中文
                rename_map = {
                    "日期": "Date",
                    "开盘": "Open",
                    "最高": "High",
                    "最低": "Low",
                    "收盘": "Close",
                    "成交量": "Volume",
                }
                df = df.rename(columns=rename_map)
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
        df = df.reset_index()
        df["Date"] = pd.to_datetime(df["Date"]).dt.strftime("%Y-%m-%d")
        return _dataframe_to_csv(df)
    except Exception as e:
        return f"错误: 获取港股 {symbol} 数据失败（akshare 和 yfinance 均不可用）- {e}"


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


if __name__ == "__main__":
    # 命令行参数解析
    parser = argparse.ArgumentParser(
        description="股票行情数据获取工具 - 支持美股/A股/港股/加密货币"
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
        required=True,
        help="起始日期，格式 YYYY-MM-DD",
    )
    parser.add_argument(
        "--end",
        type=str,
        required=True,
        help="结束日期，格式 YYYY-MM-DD",
    )

    args = parser.parse_args()

    # 调用统一入口获取数据并输出
    result = fetch_stock_data(args.symbol, args.start, args.end)
    print(result)
