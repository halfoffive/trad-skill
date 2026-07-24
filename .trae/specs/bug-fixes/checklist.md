# TradingAgents Skill Bug 修复 - Verification Checklist

- [x] Checkpoint 1: install.mjs 中 opencode 路径为 `~/.config/opencode/skills`，而非 `~/.opencode/skills`
- [x] Checkpoint 2: fetch_stock_data.py 中 _normalize_ohlcv 的 docstring 不再包含 "olumnolume" 拼写错误
- [x] Checkpoint 3: fetch_fundamentals.py 中 _fmt_num 函数对数值输入返回 str 类型（而非 float）
- [x] Checkpoint 4: fetch_fundamentals.py 中 fetch_cn_fundamentals 在调用 akshare API 前显式检查 `ak is not None`
- [x] Checkpoint 5: fetch_sentiment.py 中 fetch_cn_sentiment 在调用 akshare API 前显式检查 `ak is not None`
- [x] Checkpoint 6: fetch_sentiment.py 中美股分支正确处理 `<unavailable>`，输出友好的错误提示块而非裸占位符
- [x] Checkpoint 7: 所有 4 个 Python 脚本通过 ast.parse 语法检查
- [x] Checkpoint 8: /workspace/skills/tradingagents-analysis/ 与 /workspace/tradingagents-analysis/ 两个目录完全一致（diff 为空）
- [x] Checkpoint 9: 修复未引入新的依赖或 import
- [x] Checkpoint 10: 代码风格保持一致（中文注释、函数式风格、无 class 定义）
