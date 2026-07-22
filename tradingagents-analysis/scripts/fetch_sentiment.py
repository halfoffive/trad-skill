#!/usr/bin/env python3
"""
市场情绪数据获取工具

支持 StockTwits、Reddit、A股评论的情绪数据获取。
函数式编程风格，无类定义。
用于 TradingAgents 多智能体分析流水线的情绪分析阶段。

依赖: pip install requests akshare
"""

import argparse
import sys

import requests
import akshare as ak


def fetch_stocktwits(symbol: str, limit: int = 30) -> str:
    """
    获取 StockTwits 情绪数据。

    使用 StockTwits 公开 API（无需 API key），
    提取消息体和看涨/看跌标签，计算情绪比例。

    参数:
        symbol: 股票代码，如 AAPL
        limit: 获取消息数量上限，默认 30

    返回:
        格式化的情绪报告字符串，API 不可用时返回 "<unavailable>"
    """
    # 构造 StockTwits API 请求地址
    url = f"https://api.stocktwits.com/api/2/streams/symbol/{symbol}.json"

    try:
        # 发送 GET 请求，设置超时
        resp = requests.get(url, timeout=10)
        resp.raise_for_status()
        data = resp.json()
    except Exception:
        # API 不可用时返回占位符
        return "<unavailable>"

    # 提取消息列表
    messages = data.get("messages", [])
    if not messages:
        return f"# StockTwits 情绪 ({symbol})\n\n> 无消息数据\n"

    # 统计看涨/看跌数量
    bullish_count = 0
    bearish_count = 0
    neutral_count = 0

    # 收集最近消息文本
    recent_messages: list[str] = []

    for msg in messages[:limit]:
        # 提取情绪标签
        entities = msg.get("entities", {})
        sentiment = entities.get("sentiment", {})
        basic = sentiment.get("basic", "") if sentiment else ""

        if basic == "Bullish":
            bullish_count += 1
        elif basic == "Bearish":
            bearish_count += 1
        else:
            neutral_count += 1

        # 提取消息体文本
        body = msg.get("body", "")
        if body:
            recent_messages.append(f"- {body[:200]}")

    # 计算看涨/看跌比例
    total_tagged = bullish_count + bearish_count
    if total_tagged > 0:
        bullish_pct = round(bullish_count / total_tagged * 100, 1)
        bearish_pct = round(bearish_count / total_tagged * 100, 1)
    else:
        bullish_pct = 0.0
        bearish_pct = 0.0

    # 构建报告
    sections: list[str] = []
    sections.append(f"# StockTwits 情绪 ({symbol})\n")
    sections.append("## 情绪统计\n")
    sections.append(f"- **看涨 (Bullish)**: {bullish_count} ({bullish_pct}%)")
    sections.append(f"- **看跌 (Bearish)**: {bearish_count} ({bearish_pct}%)")
    sections.append(f"- **中性/未标注**: {neutral_count}")
    sections.append("")
    sections.append("## 最近消息\n")
    sections.extend(recent_messages[:15])

    return "\n".join(sections)


def fetch_reddit_sentiment(symbol: str, days: int = 7) -> str:
    """
    获取 Reddit 情绪数据。

    使用 Reddit 公开 JSON API（无需 OAuth），
    搜索 r/wallstreetbets, r/stocks, r/investing 的相关帖子。

    参数:
        symbol: 股票代码，如 AAPL
        days: 搜索时间范围（天），默认 7

    返回:
        格式化的帖子列表字符串，被拦截时返回 "<unavailable>"
    """
    # 设置 User-Agent 避免被 Reddit 拦截
    headers = {
        "User-Agent": "TradingAgents-Skill/1.0"
    }

    # 要搜索的子版块列表
    subreddits = ["wallstreetbets", "stocks", "investing"]

    # 根据天数确定时间范围参数
    time_filter = "week" if days <= 7 else "month"

    # 收集所有帖子
    all_posts: list[dict] = []

    for subreddit in subreddits:
        # 构造 Reddit 搜索 API 地址
        url = (
            f"https://www.reddit.com/r/{subreddit}/search.json"
            f"?q={symbol}&sort=new&t={time_filter}&limit=10"
        )

        try:
            # 发送请求，设置超时
            resp = requests.get(url, headers=headers, timeout=10)
            resp.raise_for_status()
            data = resp.json()

            # 提取帖子数据
            children = data.get("data", {}).get("children", [])
            for child in children:
                post_data = child.get("data", {})
                all_posts.append({
                    "title": post_data.get("title", ""),
                    "score": post_data.get("score", 0),
                    "num_comments": post_data.get("num_comments", 0),
                    "subreddit": subreddit,
                })
        except Exception:
            # 单个子版块失败不影响其他
            continue

    # 如果没有获取到任何帖子
    if not all_posts:
        return "<unavailable>"

    # 按互动量（分数 + 评论数）降序排序
    all_posts.sort(key=lambda p: p["score"] + p["num_comments"], reverse=True)

    # 构建报告
    sections: list[str] = []
    sections.append(f"# Reddit 情绪 ({symbol})\n")
    sections.append(f"搜索范围: r/wallstreetbets, r/stocks, r/investing（最近 {days} 天）\n")
    sections.append("## 热门帖子（按互动量排序）\n")

    for post in all_posts[:20]:
        # 格式化每个帖子信息
        title = post["title"][:100]
        score = post["score"]
        comments = post["num_comments"]
        sub = post["subreddit"]
        sections.append(f"- [{sub}] {title} (⬆{score} | 💬{comments})")

    return "\n".join(sections)


def fetch_cn_sentiment(symbol: str) -> str:
    """
    获取A股市场情绪数据。

    使用 akshare 获取个股评论和机构参与度数据。

    参数:
        symbol: A股股票代码，如 600519

    返回:
        格式化的情绪报告字符串，失败时返回 "<unavailable>"
    """
    # 初始化报告内容
    sections: list[str] = []
    sections.append(f"# A股情绪分析 ({symbol})\n")

    # 标记是否有数据获取成功
    has_data = False

    # 获取个股评论数据
    try:
        df_comment = ak.stock_comment_em(symbol=symbol)
        if df_comment is not None and not df_comment.empty:
            has_data = True
            sections.append("## 个股评论\n")
            # 展示最近的评论数据
            recent = df_comment.head(10)
            sections.append(recent.to_markdown(index=False))
            sections.append("")
        else:
            sections.append("## 个股评论\n\n> 无数据\n")
    except Exception:
        # 评论数据获取失败
        sections.append("## 个股评论\n\n> 获取失败\n")

    # 降级：获取机构参与度数据
    try:
        df_detail = ak.stock_comment_detail_zlkp_jgcyd_em(symbol=symbol)
        if df_detail is not None and not df_detail.empty:
            has_data = True
            sections.append("## 机构参与度\n")
            # 展示最近的机构参与度数据
            recent = df_detail.head(10)
            sections.append(recent.to_markdown(index=False))
            sections.append("")
        else:
            sections.append("## 机构参与度\n\n> 无数据\n")
    except Exception:
        # 机构参与度获取失败
        sections.append("## 机构参与度\n\n> 获取失败\n")

    # 如果所有数据源都失败
    if not has_data:
        return "<unavailable>"

    return "\n".join(sections)


def fetch_sentiment(symbol: str) -> str:
    """
    统一市场情绪数据获取入口。

    自动检测市场类型：
    - A股（6位纯数字）→ 获取A股评论和机构参与度
    - 美股/其他 → 组合 StockTwits + Reddit 情绪数据

    参数:
        symbol: 股票代码

    返回:
        综合情绪报告字符串
    """
    # 判断是否为A股代码（6位纯数字）
    if symbol.isdigit() and len(symbol) == 6:
        return fetch_cn_sentiment(symbol)

    # 美股/其他：组合多个数据源
    sections: list[str] = []
    sections.append(f"# {symbol} 综合情绪分析报告\n")

    # 获取 StockTwits 情绪数据
    stocktwits_result = fetch_stocktwits(symbol)
    sections.append(stocktwits_result)
    sections.append("\n---\n")

    # 获取 Reddit 情绪数据
    reddit_result = fetch_reddit_sentiment(symbol)
    sections.append(reddit_result)

    return "\n".join(sections)


if __name__ == "__main__":
    # 命令行参数解析
    parser = argparse.ArgumentParser(
        description="市场情绪数据获取工具 - 支持 StockTwits、Reddit、A股评论"
    )
    parser.add_argument(
        "--symbol",
        type=str,
        required=True,
        help="股票代码，如 AAPL（美股）或 600519（A股）",
    )
    parser.add_argument(
        "--limit",
        type=int,
        default=30,
        help="StockTwits 消息获取数量上限（默认 30）",
    )
    args = parser.parse_args()

    # 调用统一入口获取情绪数据
    result = fetch_sentiment(args.symbol)
    print(result)
