#!/usr/bin/env python3
"""
Round 6 修复验证脚本。

逐 bug 验证 R6-1 ~ R6-30 的修复状态。目标：≥30 项检查全过。

使用：
    uv run --with pandas --with yfinance --with requests python verify_round6.py
"""

# 所有注释用中文
# 函数式：每个 check_* 函数返回 (bug_id, ok, detail)

import ast
import os
import re
import sys

ROOT = os.path.dirname(os.path.abspath(__file__))
TA = os.path.join(ROOT, "tradingagents-analysis")
SCRIPTS = os.path.join(TA, "scripts")
PROMPTS = os.path.join(TA, "references", "prompts")


def _read(path):
    with open(path, encoding="utf-8") as f:
        return f.read()


def _check_ast(path):
    try:
        ast.parse(_read(path))
        return True
    except SyntaxError as e:
        return False


# ===== HIGH =====

def check_r6_1():
    """R6-1: fetch_us_fundamentals 预声明 ticker=None + 短路三大报表。"""
    src = _read(os.path.join(SCRIPTS, "fetch_fundamentals.py"))
    ok = ("ticker = None" in src and
          "if ticker is None" in src and
          "financials = None" in src)
    return ("R6-1", ok, "ticker scope guard present")


def check_r6_2():
    """R6-2: fetch_yfinance_news 实现了 days 过滤。"""
    src = _read(os.path.join(SCRIPTS, "fetch_news.py"))
    ok = ("_parse_news_time" in src and
          "cutoff = datetime.now(timezone.utc) - timedelta(days=days)" in src and
          "if pub_time is not None and pub_time < cutoff" in src)
    return ("R6-2", ok, "days filter implemented")


def check_r6_3():
    """R6-3: README 三个变量替换规则与源仓库一致。"""
    src = _read(os.path.join(PROMPTS, "README.md"))
    ok = ("`stock` for equities" in src and
          "`company` for equities" in src and
          "`Company fundamentals report` for equities" in src)
    return ("R6-3", ok, "var substitution rules aligned with source")


def check_r6_4():
    """R6-4: 6 个 prompt front-matter 只列 body 中实际存在的变量。"""
    checks = {
        "china_market_analyst.md": "(none — body is static text",
        "cn_news_analyst.md": "(none — body is static text",
        "market_analyst.md": "`{get_language_instruction()}` — the only variable",
        "fundamentals_analyst.md": "`{get_language_instruction()}` — the only variable",
        "news_analyst.md": "`{asset_label}` (company/asset), `{get_language_instruction()}`",
        "sentiment_analyst.md": "`{ticker}`, `{start_date}`, `{end_date}`, `{news_block}`",
    }
    failures = []
    for fname, marker in checks.items():
        content = _read(os.path.join(PROMPTS, fname))
        if marker not in content:
            failures.append(fname)
    return ("R6-4", not failures, f"front-matter fixed: {failures or 'all ok'}")


# ===== MEDIUM =====

def check_r6_5():
    """R6-5: RSI 注释标注 Wilder ewm 简化实现。"""
    src = _read(os.path.join(SCRIPTS, "fetch_stock_data.py"))
    ok = ("Wilder 平滑法的 pandas ewm 简化实现" in src and
          "偏差约 1pp" in src)
    return ("R6-5", ok, "RSI Wilder approximation documented")


def check_r6_6():
    """R6-6: fetch_cn/hk_stock_data 加日期类型守卫。"""
    src = _read(os.path.join(SCRIPTS, "fetch_stock_data.py"))
    ok = ("if not isinstance(start_date, str) or not isinstance(end_date, str)" in src)
    # 出现 2 次（CN + HK）
    count = src.count("if not isinstance(start_date, str) or not isinstance(end_date, str)")
    return ("R6-6", ok and count == 2, f"date guards count={count} (expect 2)")


def check_r6_7():
    """R6-7: RSI avg_loss==0 → 100。"""
    src = _read(os.path.join(SCRIPTS, "fetch_stock_data.py"))
    ok = ("mask_zero_loss = avg_loss == 0" in src and
          "rsi[mask_zero_loss] = 100.0" in src)
    return ("R6-7", ok, "RSI=100 for avg_loss==0")


def check_r6_8():
    """R6-8: Bollinger std(ddof=0)。"""
    src = _read(os.path.join(SCRIPTS, "fetch_stock_data.py"))
    ok = "close.rolling(20).std(ddof=0)" in src
    return ("R6-8", ok, "Bollinger ddof=0")


def check_r6_9():
    """R6-9: SKILL.md §4 Stage 6 措辞改为 out-of-template context。"""
    src = _read(os.path.join(TA, "SKILL.md"))
    ok = ("append the four full analyst reports as out-of-template context" in src and
          "does **not** define `{market_research_report}`" in src)
    return ("R6-9", ok, "Stage 6 re-injection wording fixed")


def check_r6_10():
    """R6-10: SKILL.md §4 CN swap "3个" → "2个"。"""
    src = _read(os.path.join(TA, "SKILL.md"))
    ok = "其余 2 个分析师（Sentiment / Fundamentals）" in src
    return ("R6-10", ok, "CN swap count 3→2")


def check_r6_11():
    """R6-11: README 加 {instrument_context} Note。"""
    src = _read(os.path.join(PROMPTS, "README.md"))
    ok = ("Note on `{instrument_context}` (R6-11)" in src and
          "token efficiency" in src)
    return ("R6-11", ok, "instrument_context Note added")


def check_r6_12():
    """R6-12: README 加 phantom variables Note。"""
    src = _read(os.path.join(PROMPTS, "README.md"))
    ok = ("Note on phantom variables (R6-12)" in src and
          "`{current_date}`, `{tool_names}`, `{system_message}`" in src)
    return ("R6-12", ok, "phantom variables Note added")


# ===== LOW =====

def check_r6_13():
    """R6-13: fetch_news.py datetime import 不再是死代码（_parse_news_time 使用）。"""
    src = _read(os.path.join(SCRIPTS, "fetch_news.py"))
    ok = ("from datetime import datetime, timedelta, timezone" in src and
          "_parse_news_time" in src and
          "datetime.fromtimestamp" in src and
          "timedelta(days=days)" in src)
    return ("R6-13", ok, "datetime import is alive (used by _parse_news_time)")


def check_r6_14():
    """R6-14: _val 用 math.isfinite 排除 inf。"""
    src = _read(os.path.join(SCRIPTS, "fetch_stock_data.py"))
    ok = ("import math" in src and
          "math.isfinite(float(v))" in src)
    return ("R6-14", ok, "_val rejects inf via math.isfinite")


def check_r6_15():
    """R6-15: fetch_stocktwits docstring "默认 15"。"""
    src = _read(os.path.join(SCRIPTS, "fetch_sentiment.py"))
    # 找 fetch_stocktwits 函数内的 docstring
    m = re.search(r"def fetch_stocktwits.*?\"\"\"(.*?)\"\"\"", src, re.DOTALL)
    if not m:
        return ("R6-15", False, "fetch_stocktwits docstring not found")
    doc = m.group(1)
    ok = "默认 15" in doc and "默认 30" not in doc
    return ("R6-15", ok, "docstring says 默认 15")


def check_r6_16():
    """R6-16: fetch_news + fetch_sentiment 负数钳制。"""
    news = _read(os.path.join(SCRIPTS, "fetch_news.py"))
    sent = _read(os.path.join(SCRIPTS, "fetch_sentiment.py"))
    news_ok = (news.count("days = max(1, int(days))") >= 2 and  # yfinance + google + cn
               news.count("limit = max(0, int(limit))") >= 3)
    sent_ok = (sent.count("limit = max(0, int(limit))") >= 1 and
               sent.count("days = max(1, int(days))") >= 1)
    return ("R6-16", news_ok and sent_ok, f"news={news_ok} sent={sent_ok}")


def check_r6_17():
    """R6-17: compute_stats 注释改为"日百分比收益"。"""
    src = _read(os.path.join(SCRIPTS, "fetch_stock_data.py"))
    ok = "日百分比收益" in src and "日对数收益" not in src
    return ("R6-17", ok, "comment says 日百分比收益")


def check_r6_18():
    """R6-18: MFI 0/0 → 50, X/0 → 100/0。"""
    src = _read(os.path.join(SCRIPTS, "fetch_stock_data.py"))
    ok = ("mask_both_zero" in src and
          "mfi[mask_both_zero] = 50.0" in src and
          "mfi[mask_neg_zero] = 100.0" in src and
          "mfi[mask_pos_zero] = 0.0" in src)
    return ("R6-18", ok, "MFI edge cases handled")


def check_r6_19():
    """R6-19: fetch_sentiment 加 symbol URL 编码注释。"""
    src = _read(os.path.join(SCRIPTS, "fetch_sentiment.py"))
    ok = ("symbol 未做 URL 编码" in src and
          src.count("symbol 未做 URL 编码") >= 2)  # stocktwits + reddit
    return ("R6-19", ok, "URL encoding limitation documented")


def check_r6_20():
    """R6-20: fetch_yfinance_news content.get 类型守卫。"""
    src = _read(os.path.join(SCRIPTS, "fetch_news.py"))
    ok = ("if not isinstance(content, dict)" in src and
          "content = {}" in src)
    return ("R6-20", ok, "content type guard present")


def check_r6_21():
    """R6-21: build_compact_report tail=None 守卫。"""
    src = _read(os.path.join(SCRIPTS, "fetch_stock_data.py"))
    ok = "tail = max(0, int(tail)) if tail is not None else 0" in src
    return ("R6-21", ok, "tail=None guard")


def check_r6_22():
    """R6-22: fetch_stock_data + fetch_fundamentals 加北交所/B股注释。"""
    sd = _read(os.path.join(SCRIPTS, "fetch_stock_data.py"))
    fd = _read(os.path.join(SCRIPTS, "fetch_fundamentals.py"))
    sd_ok = "未覆盖北交所（8 开头" in sd
    fd_ok = "未覆盖北交所（8 开头" in fd
    return ("R6-22", sd_ok and fd_ok, f"stock_data={sd_ok} fundamentals={fd_ok}")


def check_r6_23():
    """R6-23: indicators.md 加 MFI Note。"""
    src = _read(os.path.join(TA, "references", "indicators.md"))
    ok = "Note on MFI (R6-23)" in src
    return ("R6-23", ok, "MFI Note added to indicators.md")


def check_r6_24():
    """R6-24: README 加 whitespace before {get_language_instruction()} Note。"""
    src = _read(os.path.join(PROMPTS, "README.md"))
    ok = "Note on whitespace before `{get_language_instruction()}` (R6-24)" in src
    return ("R6-24", ok, "whitespace Note added")


def check_r6_25():
    """R6-25: SKILL.md §5 Stage 4 加 trader.md 2-block Note。"""
    src = _read(os.path.join(TA, "SKILL.md"))
    ok = "Note (R6-25)" in src and "separate `## System Message` and `## User Message`" in src
    return ("R6-25", ok, "trader.md 2-block Note added")


def check_r6_26():
    """R6-26: README {get_language_instruction()} English → 空字符串。"""
    src = _read(os.path.join(PROMPTS, "README.md"))
    ok = ("**empty string** (no instruction injected" in src and
          "` Write your entire response in <lang>.`" in src)
    return ("R6-26", ok, "get_language_instruction English=empty aligned with source")


def check_r6_27():
    """R6-27: SKILL.md §6 表格列出 --indicators/--no-indicators/--no-stats。"""
    src = _read(os.path.join(TA, "SKILL.md"))
    ok = ("`--indicators`/`--no-indicators`" in src and
          "`--stats`/`--no-stats`" in src and
          "`--tail N`" in src)
    return ("R6-27", ok, "all flags listed in §6 table")


def check_r6_28():
    """R6-28: install.mjs destDir 用 path.resolve。"""
    src = _read(os.path.join(ROOT, "install.mjs"))
    ok = "const destDir = path.resolve(parentDir, SKILL_NAME)" in src
    return ("R6-28", ok, "destDir uses path.resolve")


def check_r6_29():
    """R6-29: README.md market 默认值 ... → —。"""
    src = _read(os.path.join(ROOT, "README.md"))
    # 找 market 行
    m = re.search(r"\| `market` \|.*?\| (.*?) \|", src)
    if not m:
        return ("R6-29", False, "market row not found")
    val = m.group(1)
    ok = val == "—" and "..." not in val
    return ("R6-29", ok, f"market default='{val}'")


def check_r6_30():
    """R6-30: README_CN.md 港股 "4-5位数字"。"""
    src = _read(os.path.join(ROOT, "README_CN.md"))
    ok = "4-5 位数字" in src and "0700.HK 或 00700.HK" in src
    return ("R6-30", ok, "HK code description unified")


# ===== 附加检查 =====

def check_ast_all():
    """所有 4 个脚本语法 OK（两份副本）。"""
    files = [
        os.path.join(SCRIPTS, "fetch_stock_data.py"),
        os.path.join(SCRIPTS, "fetch_news.py"),
        os.path.join(SCRIPTS, "fetch_sentiment.py"),
        os.path.join(SCRIPTS, "fetch_fundamentals.py"),
        os.path.join(ROOT, "skills", "tradingagents-analysis", "scripts", "fetch_stock_data.py"),
        os.path.join(ROOT, "skills", "tradingagents-analysis", "scripts", "fetch_news.py"),
        os.path.join(ROOT, "skills", "tradingagents-analysis", "scripts", "fetch_sentiment.py"),
        os.path.join(ROOT, "skills", "tradingagents-analysis", "scripts", "fetch_fundamentals.py"),
    ]
    failures = [os.path.basename(f) for f in files if not _check_ast(f)]
    return ("AST", not failures, f"all 8 files parse OK: {failures or 'ok'}")


def check_copies_in_sync():
    """两份副本内容一致（排除 __pycache__）。"""
    import filecmp
    root_dir = os.path.join(TA)
    skills_dir = os.path.join(ROOT, "skills", "tradingagents-analysis")
    ignore = {"__pycache__"}
    mismatches = []
    for root, dirs, files in os.walk(root_dir):
        dirs[:] = [d for d in dirs if d not in ignore]
        for f in files:
            rel = os.path.relpath(os.path.join(root, f), root_dir)
            f1 = os.path.join(root_dir, rel)
            f2 = os.path.join(skills_dir, rel)
            if not os.path.exists(f2):
                mismatches.append(f"only in root: {rel}")
            elif not filecmp.cmp(f1, f2, shallow=False):
                mismatches.append(f"differ: {rel}")
    # 反向检查
    for root, dirs, files in os.walk(skills_dir):
        dirs[:] = [d for d in dirs if d not in ignore]
        for f in files:
            rel = os.path.relpath(os.path.join(root, f), skills_dir)
            f1 = os.path.join(root_dir, rel)
            if not os.path.exists(f1):
                mismatches.append(f"only in skills: {rel}")
    return ("SYNC", not mismatches, f"copies in sync: {mismatches or 'ok'}")


def main():
    checks = [
        check_r6_1, check_r6_2, check_r6_3, check_r6_4,
        check_r6_5, check_r6_6, check_r6_7, check_r6_8, check_r6_9, check_r6_10,
        check_r6_11, check_r6_12,
        check_r6_13, check_r6_14, check_r6_15, check_r6_16, check_r6_17,
        check_r6_18, check_r6_19, check_r6_20, check_r6_21, check_r6_22,
        check_r6_23, check_r6_24, check_r6_25, check_r6_26, check_r6_27,
        check_r6_28, check_r6_29, check_r6_30,
        check_ast_all, check_copies_in_sync,
    ]
    results = [c() for c in checks]
    passed = sum(1 for _, ok, _ in results if ok)
    total = len(results)
    print(f"\n=== Round 6 Verification: {passed}/{total} passed ===\n")
    for bug_id, ok, detail in results:
        mark = "✓" if ok else "✗"
        print(f"  {mark} {bug_id}: {detail}")
    print()
    if passed != total:
        print(f"FAILED: {total - passed} check(s) failed")
        sys.exit(1)
    print("ALL CHECKS PASSED")


if __name__ == "__main__":
    main()
