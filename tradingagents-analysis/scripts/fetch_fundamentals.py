#!/usr/bin/env python3
"""
基本面数据获取工具

支持美股和A股的基本面财务数据获取。
函数式编程风格，无类定义。
用于 TradingAgents 多智能体分析流水线的基本面分析阶段。

依赖: pip install yfinance akshare pandas
"""

import argparse
import sys

import yfinance as yf
import pandas as pd

try:
    import akshare as ak
except ImportError:
    # akshare 为可选依赖，缺失时 A股 接口降级，回退到 yfinance
    ak = None


def fetch_us_fundamentals(symbol: str) -> str:
    """
    获取美股基本面数据。

    使用 yfinance 获取公司概况、利润表、资产负债表和现金流量表，
    返回 markdown 格式的综合报告。

    参数:
        symbol: 美股股票代码，如 AAPL, MSFT

    返回:
        markdown 格式的基本面报告字符串
    """
    # 初始化报告内容
    sections: list[str] = []
    sections.append(f"# {symbol} 基本面分析报告\n")

    # 创建 Ticker 对象
    ticker = yf.Ticker(symbol)

    # 获取公司概况信息
    try:
        info = ticker.info
        sections.append("## 公司概况\n")
        # 提取关键字段，缺失时用 N/A 占位
        company_name = info.get("longName", "N/A")
        sector = info.get("sector", "N/A")
        industry = info.get("industry", "N/A")
        market_cap = info.get("marketCap", "N/A")
        trailing_pe = info.get("trailingPE", "N/A")
        price_to_book = info.get("priceToBook", "N/A")
        roe = info.get("returnOnEquity", "N/A")
        revenue = info.get("totalRevenue", "N/A")
        profit_margin = info.get("profitMargins", "N/A")

        sections.append(f"- **公司名称**: {company_name}")
        sections.append(f"- **行业**: {sector} / {industry}")
        sections.append(f"- **市值**: {market_cap}")
        sections.append(f"- **市盈率 (PE)**: {trailing_pe}")
        sections.append(f"- **市净率 (PB)**: {price_to_book}")
        sections.append(f"- **净资产收益率 (ROE)**: {roe}")
        sections.append(f"- **总营收**: {revenue}")
        sections.append(f"- **利润率**: {profit_margin}")
        sections.append("")
    except Exception as e:
        # 公司概况获取失败不影响其他数据
        sections.append(f"## 公司概况\n\n> 获取失败: {e}\n")

    # 获取利润表数据
    try:
        financials = ticker.financials
        if financials is not None and not financials.empty:
            sections.append("## 利润表（最近年度）\n")
            # 取最近4列数据
            recent = financials.iloc[:, :4]
            sections.append(recent.to_markdown())
            sections.append("")
        else:
            sections.append("## 利润表\n\n> 无数据\n")
    except Exception as e:
        # 利润表获取失败不影响其他数据
        sections.append(f"## 利润表\n\n> 获取失败: {e}\n")

    # 获取资产负债表数据
    try:
        balance = ticker.balance_sheet
        if balance is not None and not balance.empty:
            sections.append("## 资产负债表（最近年度）\n")
            # 取最近4列数据
            recent = balance.iloc[:, :4]
            sections.append(recent.to_markdown())
            sections.append("")
        else:
            sections.append("## 资产负债表\n\n> 无数据\n")
    except Exception as e:
        # 资产负债表获取失败不影响其他数据
        sections.append(f"## 资产负债表\n\n> 获取失败: {e}\n")

    # 获取现金流量表数据
    try:
        cashflow = ticker.cashflow
        if cashflow is not None and not cashflow.empty:
            sections.append("## 现金流量表（最近年度）\n")
            # 取最近4列数据
            recent = cashflow.iloc[:, :4]
            sections.append(recent.to_markdown())
            sections.append("")
        else:
            sections.append("## 现金流量表\n\n> 无数据\n")
    except Exception as e:
        # 现金流量表获取失败不影响其他数据
        sections.append(f"## 现金流量表\n\n> 获取失败: {e}\n")

    return "\n".join(sections)


def fetch_cn_fundamentals(symbol: str) -> str:
    """
    获取A股基本面数据。

    使用 akshare 获取财务分析指标和个股基本信息，
    如果 akshare 失败则降级使用 yfinance（加 .SS/.SZ 后缀）。

    参数:
        symbol: A股股票代码，如 600519, 000001

    返回:
        markdown 格式的基本面报告字符串
    """
    # 初始化报告内容
    sections: list[str] = []
    sections.append(f"# {symbol} A股基本面分析报告\n")

    # 标记是否有任何数据获取成功
    has_data = False

    # 获取财务分析指标
    try:
        df_indicator = ak.stock_financial_analysis_indicator(symbol=symbol)
        if df_indicator is not None and not df_indicator.empty:
            has_data = True
            sections.append("## 财务分析指标\n")
            # 取最近几行数据展示
            recent = df_indicator.head(8)
            sections.append(recent.to_markdown(index=False))
            sections.append("")
        else:
            sections.append("## 财务分析指标\n\n> 无数据\n")
    except Exception as e:
        # 财务指标获取失败，记录错误
        sections.append(f"## 财务分析指标\n\n> 获取失败: {e}\n")

    # 获取个股基本信息
    try:
        df_info = ak.stock_individual_info_em(symbol=symbol)
        if df_info is not None and not df_info.empty:
            has_data = True
            sections.append("## 个股基本信息\n")
            sections.append(df_info.to_markdown(index=False))
            sections.append("")
        else:
            sections.append("## 个股基本信息\n\n> 无数据\n")
    except Exception as e:
        # 个股信息获取失败，记录错误
        sections.append(f"## 个股基本信息\n\n> 获取失败: {e}\n")

    # 降级策略：如果 akshare 全部失败，尝试 yfinance
    if not has_data:
        sections.append("\n> akshare 数据获取失败，尝试 yfinance 降级方案...\n")
        # 根据代码前缀判断交易所后缀
        if symbol.startswith("6"):
            yf_symbol = f"{symbol}.SS"  # 上海证券交易所
        else:
            yf_symbol = f"{symbol}.SZ"  # 深圳证券交易所

        try:
            ticker = yf.Ticker(yf_symbol)
            info = ticker.info
            sections.append("## 公司概况（yfinance 降级）\n")
            company_name = info.get("longName", "N/A")
            market_cap = info.get("marketCap", "N/A")
            trailing_pe = info.get("trailingPE", "N/A")
            price_to_book = info.get("priceToBook", "N/A")

            sections.append(f"- **公司名称**: {company_name}")
            sections.append(f"- **市值**: {market_cap}")
            sections.append(f"- **市盈率 (PE)**: {trailing_pe}")
            sections.append(f"- **市净率 (PB)**: {price_to_book}")
            sections.append("")
        except Exception as e:
            # yfinance 降级也失败
            sections.append(f"> yfinance 降级也失败: {e}\n")

    return "\n".join(sections)


def fetch_fundamentals(symbol: str) -> str:
    """
    统一基本面数据获取入口。

    自动检测市场类型：
    - 6位纯数字 → A股
    - 其他 → 美股

    参数:
        symbol: 股票代码

    返回:
        markdown 格式的基本面报告字符串
    """
    # 判断是否为A股代码（6位纯数字）
    if symbol.isdigit() and len(symbol) == 6:
        return fetch_cn_fundamentals(symbol)
    else:
        # 默认按美股处理
        return fetch_us_fundamentals(symbol)


if __name__ == "__main__":
    # 命令行参数解析
    parser = argparse.ArgumentParser(
        description="基本面数据获取工具 - 支持美股和A股"
    )
    parser.add_argument(
        "--symbol",
        type=str,
        required=True,
        help="股票代码，如 AAPL（美股）或 600519（A股）",
    )
    args = parser.parse_args()

    # 调用统一入口获取基本面数据
    result = fetch_fundamentals(args.symbol)
    print(result)
