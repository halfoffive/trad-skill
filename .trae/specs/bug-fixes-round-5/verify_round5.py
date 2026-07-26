"""Round-5 验证套件：14 个 BUG 全量检查 + round-4 回归。

每个 BUG 一个测试函数，全部通过则打印 ALL OK。
"""
import sys
import os
import re
import subprocess
import json

# 让脚本能找到本仓库的脚本
sys.path.insert(0, 'tradingagents-analysis/scripts')

PASS = "✓"
FAIL = "✗"
results = []

def check(name, ok, detail=""):
    results.append((name, ok, detail))
    mark = PASS if ok else FAIL
    print(f"  {mark} {name}" + (f"  ({detail})" if detail and not ok else ""))

# ========== R5-1: 9 个非 CN prompt 含 {get_language_instruction()} ==========
def test_r5_1():
    print("\n[R5-1] 9 个非 CN prompt 含 {get_language_instruction()}")
    non_cn = ['market_analyst', 'news_analyst', 'fundamentals_analyst', 'bull_researcher',
              'bear_researcher', 'research_manager', 'trader', 'aggressive_risk',
              'conservative_risk', 'neutral_risk', 'sentiment_analyst', 'portfolio_manager']
    cn = ['china_market_analyst', 'cn_news_analyst']
    for name in non_cn:
        path = f'tradingagents-analysis/references/prompts/{name}.md'
        with open(path, encoding='utf-8') as f:
            content = f.read()
        ok = '{get_language_instruction()}' in content
        check(f'non-CN {name}.md has var', ok)
    for name in cn:
        path = f'tradingagents-analysis/references/prompts/{name}.md'
        with open(path, encoding='utf-8') as f:
            content = f.read()
        ok = '{get_language_instruction()}' not in content
        check(f'CN {name}.md does NOT have var', ok)
    # skills/ 副本同步
    for name in non_cn:
        root = open(f'tradingagents-analysis/references/prompts/{name}.md', encoding='utf-8').read()
        skills = open(f'skills/tradingagents-analysis/references/prompts/{name}.md', encoding='utf-8').read()
        check(f'skills/ copy of {name}.md synced', root == skills)

# ========== R5-2: 3 个脚本入口 None 守卫 ==========
def test_r5_2():
    print("\n[R5-2] fetch_news/fetch_stock_data/fetch_sentiment None 守卫")
    import fetch_news, fetch_stock_data, fetch_sentiment
    # fetch_news
    check('fetch_news(None) returns str', isinstance(fetch_news.fetch_news(None), str))
    check('fetch_news(None) starts with 错误', fetch_news.fetch_news(None).startswith('错误'))
    check('fetch_news("") returns str', isinstance(fetch_news.fetch_news(""), str))
    check('fetch_news("   ") returns str', isinstance(fetch_news.fetch_news("   "), str))
    # fetch_stock_data
    r = fetch_stock_data.fetch_stock_data(None, '2024-01-01', '2024-01-02')
    check('fetch_stock_data(None,...) returns str', isinstance(r, str))
    check('fetch_stock_data(None,...) starts with 错误', r.startswith('错误'))
    # fetch_sentiment
    r = fetch_sentiment.fetch_sentiment(None)
    check('fetch_sentiment(None) returns str', isinstance(r, str))
    check('fetch_sentiment(None) starts with 错误', r.startswith('错误'))

# ========== R5-3: npm pack 不含 __pycache__/.pyc ==========
def test_r5_3():
    print("\n[R5-3] npm pack 不含 __pycache__/.pyc")
    # 创建 fake __pycache__ 验证 files 字段否定项生效
    pyc_dir = 'tradingagents-analysis/scripts/__pycache__'
    pyc_file = os.path.join(pyc_dir, 'fake.cpython-313.pyc')
    os.makedirs(pyc_dir, exist_ok=True)
    with open(pyc_file, 'w') as f:
        f.write('fake bytecode')
    try:
        r = subprocess.run(['npm', 'pack', '--dry-run'], capture_output=True, text=True, shell=True)
        output = r.stdout + r.stderr
        check('npm pack output has no __pycache__', '__pycache__' not in output)
        check('npm pack output has no .pyc', '.pyc' not in output)
        # 同时检查 CLAUDE.md / node_modules 排除
        check('npm pack output has no CLAUDE.md', 'CLAUDE.md' not in output)
        check('npm pack output has no node_modules', 'node_modules' not in output)
    finally:
        import shutil
        shutil.rmtree(pyc_dir, ignore_errors=True)
    # prepublishOnly script 存在
    pkg = json.load(open('package.json', encoding='utf-8'))
    check('package.json has prepublishOnly script', 'prepublishOnly' in pkg.get('scripts', {}))

# ========== R5-4: install.mjs --dir 相对路径打印绝对路径 ==========
def test_r5_4():
    print("\n[R5-4] install.mjs --dir 相对路径打印绝对路径")
    tmp = os.path.join(os.getcwd(), '.tmp_r5_4')
    os.makedirs(tmp, exist_ok=True)
    try:
        rel = os.path.relpath(tmp)
        r = subprocess.run(['node', 'install.mjs', '--dir', rel], capture_output=True, text=True)
        check('--dir <rel> exit 0', r.returncode == 0, r.stderr)
        matches = re.findall(r'python "([^"]+)"', r.stdout)
        check('found python abs paths', len(matches) > 0)
        if matches:
            check('first path is absolute', os.path.isabs(matches[0]), matches[0])
            check('first path uses /', '/' in matches[0])
    finally:
        import shutil
        shutil.rmtree(os.path.join(tmp, 'tradingagents-analysis'), ignore_errors=True)
        os.rmdir(tmp)

# ========== R5-5: README_CN 无 .SS/.SZ 后缀残留 ==========
def test_r5_5():
    print("\n[R5-5] README_CN .SS/.SZ 残留")
    with open('README_CN.md', encoding='utf-8') as f:
        content = f.read()
    # 找 A股特别说明 章节
    a_share_section = content[content.find('### A股特别说明'):content.find('### 港股特别说明')]
    check('A股说明章节不含 ".SS 后缀为上海"', '.SS 后缀为上海' not in a_share_section)
    check('A股说明章节含 "6 位代码前缀自动判断"', '6 位代码前缀自动判断' in a_share_section)

# ========== R5-6: install.mjs mkdirSync 在 try/catch 内 ==========
def test_r5_6():
    print("\n[R5-6] install.mjs mkdirSync 在 try/catch 内（invalid path → 友好错误）")
    r = subprocess.run(['node', 'install.mjs', '--dir', 'Z:\\bad:path'],
                       capture_output=True, text=True)
    check('invalid path exit 1', r.returncode == 1)
    # 不应含裸 Node 堆栈（at Object.<anonymous> 等）
    has_stack = 'at Object.' in r.stderr or 'at Object.' in r.stdout
    check('no raw Node stack trace', not has_stack)
    check('has friendly 安装失败 msg', '安装失败' in (r.stderr + r.stdout))

# ========== R5-7: CHANGELOG "11 个 ghost tools" ==========
def test_r5_7():
    print("\n[R5-7] CHANGELOG 1.3.4 章节 '11 个 ghost tools'")
    with open('CHANGELOG.md', encoding='utf-8') as f:
        content = f.read()
    # 在 1.3.4 章节范围内检查
    section_start = content.find('## [1.3.4]')
    section_end = content.find('## [1.3.2]')
    section = content[section_start:section_end]
    check('1.3.4 section has "11 个 ghost tools"', '11 个 ghost tools' in section)
    check('1.3.4 section does NOT have "9 个 ghost tools"', '9 个 ghost tools' not in section)

# ========== R5-8: README 相对链接 404 ==========
def test_r5_8():
    print("\n[R5-8] README 相对链接修复")
    for readme in ['README.md', 'README_CN.md']:
        with open(readme, encoding='utf-8') as f:
            content = f.read()
        bad = '](references/data-sources.md)' in content
        good = '](tradingagents-analysis/references/data-sources.md)' in content
        check(f'{readme} no bad relative link', not bad)
        check(f'{readme} has fixed link', good)

# ========== R5-9: prompts/README {investment_plan} 流向 ==========
def test_r5_9():
    print("\n[R5-9] prompts/README {investment_plan} 流向")
    with open('tradingagents-analysis/references/prompts/README.md', encoding='utf-8') as f:
        content = f.read()
    bad = 'Research Manager output → Trader input + Risk Debate input' in content
    check('no "+ Risk Debate input" for {investment_plan}', not bad)
    check('clarifies Risk Debate does NOT receive investment_plan',
          'Risk Debate does NOT receive' in content)

# ========== R5-10: prompts/README CN prompt 归因 ==========
def test_r5_10():
    print("\n[R5-10] prompts/README CN prompt 不引用 ghost tools")
    with open('tradingagents-analysis/references/prompts/README.md', encoding='utf-8') as f:
        content = f.read()
    check('has CN note for Market Analyst', 'china_market_analyst.md` does NOT reference' in content)
    check('has CN note for News Analyst', 'cn_news_analyst.md` does NOT reference' in content)

# ========== R5-11: _fmt_num 统一 NA 处理 ==========
def test_r5_11():
    print("\n[R5-11] _fmt_num 统一 NA 处理")
    import pandas as pd
    import numpy as np
    import fetch_fundamentals as ff
    check('_fmt_num(pd.NA) == "N/A"', ff._fmt_num(pd.NA) == 'N/A', ff._fmt_num(pd.NA))
    check('_fmt_num(None) == "N/A"', ff._fmt_num(None) == 'N/A')
    check('_fmt_num(np.nan) == "N/A"', ff._fmt_num(np.nan) == 'N/A')
    check('_fmt_num(pd.NaT) == "N/A"', ff._fmt_num(pd.NaT) == 'N/A')
    check('_fmt_num(3.14159) == "3.14"', ff._fmt_num(3.14159) == '3.14')

# ========== R5-12: fetch_stock_df None 守卫 ==========
def test_r5_12():
    print("\n[R5-12] fetch_stock_df None 守卫")
    import pandas as pd
    import fetch_stock_data
    r = fetch_stock_data.fetch_stock_df(None, '2024-01-01', '2024-01-02')
    check('fetch_stock_df(None,...) returns DataFrame', isinstance(r, pd.DataFrame))
    check('fetch_stock_df(None,...) empty', r.empty)
    r2 = fetch_stock_data.fetch_stock_df("", '2024-01-01', '2024-01-02')
    check('fetch_stock_df("",...) empty', r2.empty)
    r3 = fetch_stock_data.fetch_stock_df("   ", '2024-01-01', '2024-01-02')
    check('fetch_stock_df("   ",...) empty', r3.empty)

# ========== R5-13: .npmignore 含 node_modules + CLAUDE.md ==========
def test_r5_13():
    print("\n[R5-13] .npmignore 含 node_modules + CLAUDE.md")
    with open('.npmignore', encoding='utf-8') as f:
        content = f.read()
    check('.npmignore has node_modules/', 'node_modules/' in content)
    check('.npmignore has CLAUDE.md', 'CLAUDE.md' in content)

# ========== R5-14: install.mjs --dir + --agent 互斥 ==========
def test_r5_14():
    print("\n[R5-14] install.mjs --dir + --agent 互斥")
    r = subprocess.run(['node', 'install.mjs', '--dir', 'foo', '--agent', 'opencode'],
                       capture_output=True, text=True)
    check('--dir + --agent exit 1', r.returncode == 1)
    check('error msg mentions 不能同时指定', '不能同时指定' in (r.stderr + r.stdout))

# ========== round-4 回归：fetch_fundamentals None/空串守卫 ==========
def test_round4_regression():
    print("\n[Round-4 regression] fetch_fundamentals None/空串守卫")
    import fetch_fundamentals as ff
    check('fetch_fundamentals(None) returns str', isinstance(ff.fetch_fundamentals(None), str))
    check('fetch_fundamentals("") returns str', isinstance(ff.fetch_fundamentals(""), str))
    check('fetch_fundamentals(None) starts with 错误', ff.fetch_fundamentals(None).startswith('错误'))

# ========== run all ==========
if __name__ == '__main__':
    print("=" * 70)
    print("Round-5 verification suite (14 BUGs + round-4 regression)")
    print("=" * 70)
    test_r5_1()
    test_r5_2()
    test_r5_3()
    test_r5_4()
    test_r5_5()
    test_r5_6()
    test_r5_7()
    test_r5_8()
    test_r5_9()
    test_r5_10()
    test_r5_11()
    test_r5_12()
    test_r5_13()
    test_r5_14()
    test_round4_regression()

    print("\n" + "=" * 70)
    total = len(results)
    passed = sum(1 for _, ok, _ in results if ok)
    failed = total - passed
    print(f"Total: {total}, Passed: {passed}, Failed: {failed}")
    if failed:
        print("\nFAILED:")
        for name, ok, detail in results:
            if not ok:
                print(f"  ✗ {name}  ({detail})")
        sys.exit(1)
    else:
        print("\nALL OK")
