#!/usr/bin/env python3
"""
基本面数据获取工具

支持美股和A股的基本面财务数据获取。
函数式编程风格，无类定义。
用于 TradingAgents 多智能体分析流水线的基本面分析阶段。

依赖: pip install yfinance akshare pandas

降本增效设计：
- 不再用整表倾倒 4 年×全部行项的宽表（数十行，token 开销大）。
- 改为输出精挑细选的关键指标表（营收/净利润/EPS/总资产/总负债/
  经营现金流/自由现金流/毛利率/净利率 + 营收与净利 YoY），约 10 行。
- 公司概况块保留（已紧凑）。
"""

import argparse

import yfinance as yf
import pandas as pd

try:
    import akshare as ak
except ImportError:
    # akshare 为可选依赖，缺失时 A股 接口降级，回退到 yfinance
    ak = None


def _fmt_num(v) -> str:
    """格式化数值：缺失/N/A 返回 'N/A'，否则四舍五入保留 2 位。"""
    if v is None or (isinstance(v, float) and pd.isna(v)):
        return "N/A"
    try:
        return str(round(float(v), 2))
    except (TypeError, ValueError):
        return str(v)


def _yoy(series: pd.Series) -> str:
    """计算序列最近一年的同比变化（百分比）。"""
    try:
        vals = pd.to_numeric(series, errors="coerce").dropna()
        if len(vals) < 2:
            return "N/A"
        cur = vals.iloc[0]
        prev = vals.iloc[1]
        if not prev:
            return "N/A"
        return f"{round((cur / prev - 1) * 100, 2)}%"
    except Exception:
        return "N/A"


def _build_us_metric_table(financials: pd.DataFrame, balance: pd.DataFrame,
                           cashflow: pd.DataFrame) -> str:
    """
    从 yfinance 三大报表中抽取关键行项，构建紧凑关键指标表。
    行项标签取 yfinance 实际返回的英文名。
    """
    # 行项映射：显示名 -> yfinance 行标签
    rows = [
        ("营收", financials, "Total Revenue"),
        ("净利润", financials, "Net Income"),
        ("摊薄EPS", financials, "Diluted EPS"),
        ("毛利", financials, "Gross Profit"),
        ("总资产", balance, "Total Assets"),
        ("总负债", balance, "Total Debt"),
        ("股东权益", balance, "Stockholders Equity"),
        ("经营现金流", cashflow, "Operating Cash Flow"),
        ("自由现金流", cashflow, "Free Cash Flow"),
    ]

    # 收集所有出现过的年份列（按时间倒序，取最近 4 列）
    all_cols: list = []
    seen = set()
    for _, df, _ in rows:
        if df is None:
            continue
        for c in df.columns:
            key = str(c)
            if key not in seen:
                seen.add(key)
                all_cols.append(c)
    # 取最近 4 年并保持倒序
    all_cols = list(all_cols)[:4]

    if not all_cols:
        return "> 无可用年度数据\n"

    # 表头：指标 | 年1 | 年2 | ... | YoY
    year_labels = []
    for c in all_cols:
        try:
            year_labels.append(str(c).split(" ")[0][:10])
        except Exception:
            year_labels.append(str(c)[:10])

    lines = ["| 指标 | " + " | ".join(year_labels) + " | YoY(营收/净利) |"]
    lines.append("|" + "---|" * (len(year_labels) + 2))

    # 取行项的值
    def _row_vals(display: str, df, label: str) -> list:
        if df is None or label not in df.index:
            return [display] + ["N/A"] * len(all_cols) + [""]
        series = df.loc[label]
        vals = []
        for c in all_cols:
            if c in series.index:
                vals.append(_fmt_num(series[c]))
            else:
                vals.append("N/A")
        # 营收与净利附加 YoY
        yoy = ""
        if display == "营收" or display == "净利润":
            yoy = _yoy(series)
        return [display] + vals + [yoy]

    for display, df, label in rows:
        lines.append("| " + " | ".join(str(x) for x in _row_vals(display, df, label)) + " |")

    return "\n".join(lines) + "\n"


def fetch_us_fundamentals(symbol: str) -> str:
    """
    获取美股基本面数据。

    使用 yfinance 获取公司概况与三大报表，
    返回精简关键指标 markdown 报告（替代整表倾倒）。

    参数:
        symbol: 美股股票代码，如 AAPL, MSFT

    返回:
        markdown 格式的基本面报告字符串
    """
    # 防御性规整：去首尾空格（契约：函数不抛异常）
    symbol = (symbol or "").strip()
    # 初始化报告内容
    sections: list[str] = []
    sections.append(f"# {symbol} 基本面（精简）\n")

    # 创建 Ticker 对象 + 获取公司概况信息（紧凑）
    # yf.Ticker 对空串会抛 ValueError，需包在 try/except 内（与 fetch_us_stock_data 一致）
    try:
        ticker = yf.Ticker(symbol)
        info = ticker.info
        sections.append("## 公司概况\n")
        # 提取关键字段，缺失时用 N/A 占位
        fields = {
            "公司名称": info.get("longName", "N/A"),
            "行业": f"{info.get('sector', 'N/A')} / {info.get('industry', 'N/A')}",
            "市值": info.get("marketCap", "N/A"),
            "市盈率(PE)": info.get("trailingPE", "N/A"),
            "市净率(PB)": info.get("priceToBook", "N/A"),
            "ROE": info.get("returnOnEquity", "N/A"),
            "总营收": info.get("totalRevenue", "N/A"),
            "利润率": info.get("profitMargins", "N/A"),
        }
        for k, v in fields.items():
            sections.append(f"- **{k}**: {v}")
        sections.append("")
    except Exception as e:
        # 公司概况获取失败不影响其他数据
        sections.append(f"## 公司概况\n\n> 获取失败: {e}\n")

    # 取三大报表（不整表倾倒，只用于抽关键行项）
    try:
        financials = ticker.financials
    except Exception:
        financials = None
    try:
        balance = ticker.balance_sheet
    except Exception:
        balance = None
    try:
        cashflow = ticker.cashflow
    except Exception:
        cashflow = None

    # 关键指标表
    try:
        if (financials is not None and not financials.empty) or \
           (balance is not None and not balance.empty) or \
           (cashflow is not None and not cashflow.empty):
            sections.append("## 关键财务指标（最近年度，脚本抽取）\n")
            sections.append(_build_us_metric_table(financials, balance, cashflow))
        else:
            sections.append("## 关键财务指标\n\n> 无数据\n")
    except Exception as e:
        sections.append(f"## 关键财务指标\n\n> 抽取失败: {e}\n")

    return "\n".join(sections)


def fetch_cn_fundamentals(symbol: str) -> str:
    """
    获取A股基本面数据。

    使用 akshare 获取财务分析指标和个股基本信息（精简输出），
    如果 akshare 失败则降级使用 yfinance（加 .SS/.SZ 后缀）。

    参数:
        symbol: A股股票代码，如 600519, 000001

    返回:
        markdown 格式的基本面报告字符串
    """
    # 初始化报告内容
    sections: list[str] = []
    sections.append(f"# {symbol} A股基本面（精简）\n")

    # 标记是否有任何数据获取成功
    has_data = False

    # 获取财务分析指标（只取关键列、最近若干行，避免宽表倾倒）
    if ak is not None:
        try:
            df_indicator = ak.stock_financial_analysis_indicator(symbol=symbol)
            if df_indicator is not None and not df_indicator.empty:
                has_data = True
                sections.append("## 财务分析指标（最近 4 期，精简）\n")
                # 取最近 4 行，且最多 6 列
                recent = df_indicator.tail(4).iloc[:, :6]
                sections.append(recent.to_string(index=False))
                sections.append("")
            else:
                sections.append("## 财务分析指标\n\n> 无数据\n")
        except Exception as e:
            # 财务指标获取失败，记录错误
            sections.append(f"## 财务分析指标\n\n> 获取失败: {e}\n")
    else:
        sections.append("## 财务分析指标\n\n> akshare 未安装，跳过\n")

    # 获取个股基本信息
    if ak is not None:
        try:
            df_info = ak.stock_individual_info_em(symbol=symbol)
            if df_info is not None and not df_info.empty:
                has_data = True
                sections.append("## 个股基本信息（精简）\n")
                # 只取前 10 行避免长表
                recent = df_info.head(10)
                sections.append(recent.to_string(index=False))
                sections.append("")
            else:
                sections.append("## 个股基本信息\n\n> 无数据\n")
        except Exception as e:
            # 个股信息获取失败，记录错误
            sections.append(f"## 个股基本信息\n\n> 获取失败: {e}\n")
    else:
        sections.append("## 个股基本信息\n\n> akshare 未安装，跳过\n")

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
    # 契约守卫：非字符串 / 空串 / 纯空白返回错误字符串，不抛异常
    # （fetch_us_fundamentals 的 yf.Ticker 对空串抛 ValueError，需在此拦截）
    if not isinstance(symbol, str):
        return f"错误: 无效的股票代码 {symbol!r}"
    symbol = symbol.strip()
    if not symbol:
        return "错误: 股票代码不能为空"
    # 判断是否为A股代码（6位纯数字）
    if symbol.isdigit() and len(symbol) == 6:
        return fetch_cn_fundamentals(symbol)
    else:
        # 默认按美股处理
        return fetch_us_fundamentals(symbol)


if __name__ == "__main__":
    # 命行参数解析
    parser = argparse.ArgumentParser(
        description="基本面数据获取工具 - 支持美股和A股（默认精简关键指标表）"
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
