#!/usr/bin/env python3
"""
新闻数据获取工具

支持美股新闻、全球宏观新闻、A股新闻的获取。
函数式编程风格，无类定义。
用于 TradingAgents 多智能体分析流水线的新闻分析阶段。

依赖: pip install yfinance requests akshare

降本增效设计：
- --limit 默认 8（原 20/源×2 源=40 条），减少注入提示词的条目数。
- 所有市场摘要统一截断到 200 字（原仅 A 股截断）。
- 每条精简为「标题 + 来源 + 一行摘要」，去掉冗余链接行。
"""

# 所有注释用中文
# 函数式编程：每个函数独立，无全局状态
# 返回值：markdown 格式的字符串（适合注入LLM提示词）

import argparse
import sys
import xml.etree.ElementTree as ET
from datetime import datetime, timedelta
from urllib.parse import quote_plus

import requests
import yfinance as yf

# 尝试导入 akshare，部分环境可能未安装
try:
    import akshare as ak
except ImportError:
    ak = None

# 新闻默认返回条数（降本：原 20 → 8）
DEFAULT_NEWS_LIMIT = 8
# 摘要最大字符数
MAX_SUMMARY_CHARS = 200


def _truncate(text: str, limit: int = MAX_SUMMARY_CHARS) -> str:
    """将文本截断到指定字符数，超出加省略号。"""
    if not text:
        return ""
    text = str(text).strip()
    if len(text) > limit:
        return text[:limit] + "..."
    return text


def _format_news_item(title: str, source: str, summary: str) -> str:
    """将单条新闻格式化为精简 markdown 文本块（标题 + 来源 + 一行摘要）。"""
    # 标题
    lines = [f"- **{title or '无标题'}**"]
    # 来源与摘要合并到一行，节省 token
    parts = []
    if source:
        parts.append(f"来源:{source}")
    if summary:
        parts.append(_truncate(summary))
    if parts:
        lines.append("  - " + " | ".join(parts))
    return "\n".join(lines)


def fetch_yfinance_news(symbol: str, days: int = 7, limit: int = DEFAULT_NEWS_LIMIT) -> str:
    """
    使用 yfinance 获取个股相关新闻。

    通过 yfinance Ticker.news 属性获取新闻列表，
    返回 markdown 格式字符串，最多返回 limit 条。

    参数:
        symbol: 股票代码，如 "AAPL", "MSFT"
        days: 获取最近多少天的新闻（默认7天）
        limit: 最多返回条数
    返回:
        markdown 格式新闻字符串或错误信息
    """
    # 使用 yfinance 获取个股新闻
    try:
        ticker = yf.Ticker(symbol)
        news_list = ticker.news

        # 检查是否有新闻数据
        if not news_list:
            return f"未找到 {symbol} 的相关新闻。"

        results = []

        for item in news_list[:limit]:
            # yfinance 新闻格式：content 嵌套结构
            content = item.get("content", item)
            title = content.get("title", "无标题")
            # 提取新闻来源
            publisher = content.get("provider", {})
            source = publisher.get("displayName", "") if isinstance(publisher, dict) else str(publisher)
            # 提取摘要
            summary = content.get("summary", content.get("description", ""))

            results.append(_format_news_item(title, source, summary))

        if not results:
            return f"未找到 {symbol} 在最近 {days} 天内的相关新闻。"

        # 组装最终 markdown 输出
        header = f"## {symbol} 相关新闻（最近 {days} 天，共 {len(results)} 条）\n\n"
        return header + "\n".join(results)

    except Exception as e:
        # 网络异常或其他错误，返回错误信息
        return f"错误: 获取 {symbol} 新闻失败 - {e}"


def fetch_google_news(query: str, days: int = 7, limit: int = DEFAULT_NEWS_LIMIT) -> str:
    """
    使用 Google News RSS 获取新闻（无需 API key）。

    通过 Google News 的公开 RSS feed 搜索新闻，
    解析 XML 返回 markdown 格式结果。

    参数:
        query: 搜索关键词，如 "Apple stock", "美联储利率"
        days: 获取最近多少天的新闻（默认7天）
        limit: 最多返回条数
    返回:
        markdown 格式新闻字符串或错误信息
    """
    # 构建 Google News RSS 搜索 URL
    encoded_query = quote_plus(query)
    url = f"https://news.google.com/rss/search?q={encoded_query}+when:{days}d&hl=en&gl=US&ceid=US:en"

    try:
        # 发送 HTTP 请求获取 RSS XML
        response = requests.get(url, timeout=15)
        response.raise_for_status()

        # 解析 XML 格式的 RSS feed
        root = ET.fromstring(response.content)
        items = root.findall(".//item")

        # 检查是否有搜索结果
        if not items:
            return f"Google News 未找到与 \"{query}\" 相关的新闻。"

        results = []
        # 限制最大返回条数
        for item in items[:limit]:
            # 从 RSS item 中提取各字段
            title = item.findtext("title", "无标题")
            source = item.findtext("source", "")
            # RSS description 通常包含 HTML，简单截取文本
            description = item.findtext("description", "")
            # 去除可能的 HTML 标签（简单处理）
            if "<" in description:
                description = description.split("<")[0].strip()

            results.append(_format_news_item(title, source, description))

        if not results:
            return f"Google News 未找到与 \"{query}\" 相关的新闻。"

        # 组装最终 markdown 输出
        header = f"## Google News: \"{query}\"（最近 {days} 天，共 {len(results)} 条）\n\n"
        return header + "\n".join(results)

    except requests.exceptions.Timeout:
        return f"错误: Google News 请求超时，请稍后重试。"
    except requests.exceptions.RequestException as e:
        return f"错误: Google News 请求失败 - {e}"
    except ET.ParseError as e:
        return f"错误: 解析 Google News RSS 响应失败 - {e}"


def fetch_cn_news(symbol: str, days: int = 7, limit: int = DEFAULT_NEWS_LIMIT) -> str:
    """
    获取A股个股相关新闻。

    优先使用 akshare 的 stock_news_em 接口获取东方财富新闻，
    失败时降级为 Google News 中文搜索。

    参数:
        symbol: A股代码，纯6位数字如 "600519"
        days: 获取最近多少天的新闻（默认7天）
        limit: 最多返回条数
    返回:
        markdown 格式新闻字符串或错误信息
    """
    # 优先尝试 akshare 获取A股新闻
    if ak is not None:
        try:
            df = ak.stock_news_em(symbol=symbol)
            if df is not None and not df.empty:
                results = []
                # 限制最大条数
                for _, row in df.head(limit).iterrows():
                    title = str(row.get("新闻标题", "无标题"))
                    source = str(row.get("文章来源", ""))
                    summary = str(row.get("新闻内容", ""))
                    results.append(_format_news_item(title, source, summary))

                if results:
                    header = f"## A股 {symbol} 相关新闻（共 {len(results)} 条）\n\n"
                    return header + "\n".join(results)
        except Exception:
            # akshare 失败，降级到 Google News
            pass

    # 降级方案：使用 Google News 搜索中文关键词
    # 用股票代码作为搜索词
    query = f"{symbol} A股"
    return fetch_google_news(query, days, limit=limit)


def fetch_news(symbol: str, days: int = 7, limit: int = DEFAULT_NEWS_LIMIT) -> str:
    """
    统一新闻获取入口，自动检测市场类型。

    检测逻辑:
        - 6位纯数字 → A股，使用 fetch_cn_news
        - 其他 → 美股/港股，组合 yfinance 新闻 + Google News

    参数:
        symbol: 股票代码
        days: 获取最近多少天的新闻（默认7天）
        limit: 每个来源最多返回条数（默认 8）
    返回:
        markdown 格式新闻字符串
    """
    # 去除首尾空格
    symbol = symbol.strip()

    # 判断是否为A股（6位纯数字）
    if symbol.isdigit() and len(symbol) == 6:
        return fetch_cn_news(symbol, days, limit=limit)

    # 非A股：组合 yfinance 个股新闻和 Google News 搜索
    sections = []

    # 第一部分：yfinance 个股新闻
    yf_news = fetch_yfinance_news(symbol, days, limit=limit)
    if not yf_news.startswith("错误"):
        sections.append(yf_news)

    # 第二部分：Google News 补充搜索
    google_news = fetch_google_news(symbol, days, limit=limit)
    if not google_news.startswith("错误"):
        sections.append(google_news)

    # 如果两部分都失败，返回 yfinance 的错误信息
    if not sections:
        return yf_news

    # 合并所有新闻来源
    return "\n---\n\n".join(sections)


if __name__ == "__main__":
    # 命令行参数解析
    parser = argparse.ArgumentParser(
        description="新闻数据获取工具 - 支持美股/A股/全球宏观新闻（默认精简 8 条）"
    )
    parser.add_argument(
        "--symbol",
        type=str,
        required=True,
        help="股票代码，如 AAPL、600519、00700",
    )
    parser.add_argument(
        "--days",
        type=int,
        default=7,
        help="获取最近多少天的新闻（默认7天）",
    )
    parser.add_argument(
        "--limit",
        type=int,
        default=DEFAULT_NEWS_LIMIT,
        help=f"每个来源最多返回条数（默认 {DEFAULT_NEWS_LIMIT}）",
    )

    args = parser.parse_args()

    # 调用统一入口获取新闻并输出
    result = fetch_news(args.symbol, args.days, limit=args.limit)
    print(result)
