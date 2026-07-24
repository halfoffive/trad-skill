# TradingAgents Skill Bug 修复 - The Implementation Plan (Decomposed and Prioritized Task List)

## [x] Task 1: 修复 install.mjs 中 OpenCode 安装路径错误
- **Priority**: high
- **Depends On**: None
- **Description**: 
  - 将 install.mjs 中 `opencode` 对应的路径从 `~/.opencode/skills` 改为 `~/.config/opencode/skills`，与 SKILL.md 第23行描述保持一致
  - 文件: /workspace/install.mjs
- **Acceptance Criteria Addressed**: [AC-1]
- **Test Requirements**:
  - `programmatic` TR-1.1: 检查 install.mjs 第21行路径包含 `.config/opencode` 而非 `.opencode`
  - `human-judgement` TR-1.2: 验证路径与 SKILL.md 文档中列出的三个安装位置之一匹配
- **Notes**: 只修改 opencode 路径，claude 和 agents 路径保持不变

## [x] Task 2: 修复 fetch_stock_data.py docstring 拼写错误
- **Priority**: high
- **Depends On**: None
- **Description**: 
  - 将 _normalize_ohlcv 函数 docstring 中的 "olumnolume" 修正为 "Volume"
  - 需要同时修改两个目录: /workspace/tradingagents-analysis/scripts/ 和 /workspace/skills/tradingagents-analysis/scripts/
- **Acceptance Criteria Addressed**: [AC-4, AC-6]
- **Test Requirements**:
  - `programmatic` TR-2.1: grep 检查不包含 "olumnolume" 字符串
  - `programmatic` TR-2.2: 两个目录的 diff 为空
  - `programmatic` TR-2.3: Python 语法检查通过
- **Notes**: 两个文件必须完全一致

## [x] Task 3: 修复 fetch_fundamentals.py 中 _fmt_num 返回类型错误
- **Priority**: high
- **Depends On**: None
- **Description**: 
  - 修改 _fmt_num 函数，确保始终返回 str 类型：将 `return round(float(v), 2)` 改为 `return str(round(float(v), 2))` 或格式化字符串
  - 同时为 fetch_cn_fundamentals 函数添加显式的 `if ak is not None:` 检查，保持与其他脚本一致的代码模式
  - 需要同时修改两个目录
- **Acceptance Criteria Addressed**: [AC-3, AC-5, AC-6]
- **Test Requirements**:
  - `programmatic` TR-3.1: 检查 _fmt_num 函数的 return 语句对数字返回 str 类型
  - `programmatic` TR-3.2: fetch_cn_fundamentals 在调用 akshare API 前检查 `ak is not None`
  - `programmatic` TR-3.3: 两个目录的 diff 为空
  - `programmatic` TR-3.4: Python 语法检查通过
- **Notes**: 保持函数式风格和中文注释

## [x] Task 4: 修复 fetch_sentiment.py 中多个问题
- **Priority**: high
- **Depends On**: None
- **Description**: 
  - 为 fetch_cn_sentiment 函数添加显式的 `if ak is not None:` 检查
  - 修改 fetch_sentiment 函数（美股分支），正确处理 `<unavailable>` 返回值：不要直接拼接，而是转为友好的错误提示块（类似 fetch_news.py 的模式）
  - 需要同时修改两个目录
- **Acceptance Criteria Addressed**: [AC-2, AC-5, AC-6]
- **Test Requirements**:
  - `programmatic` TR-4.1: fetch_cn_sentiment 在调用 akshare API 前检查 `ak is not None`
  - `programmatic` TR-4.2: fetch_sentiment 对 `<unavailable>` 结果进行包装，不直接输出裸字符串
  - `programmatic` TR-4.3: 两个目录的 diff 为空
  - `programmatic` TR-4.4: Python 语法检查通过
  - `human-judgement` TR-4.5: 降级模式与 fetch_news.py 风格一致（错误消息以 "> " 开头的块引用格式）
- **Notes**: 参考 fetch_news.py 第242-253行的优雅降级模式

## [x] Task 5: 最终验证所有修复
- **Priority**: high
- **Depends On**: [Task 1, Task 2, Task 3, Task 4]
- **Description**: 
  - 运行完整的验证套件
  - 确认所有 bug 已修复且未引入新问题
- **Acceptance Criteria Addressed**: [AC-1, AC-2, AC-3, AC-4, AC-5, AC-6, AC-7]
- **Test Requirements**:
  - `programmatic` TR-5.1: 所有 4 个 Python 脚本通过 ast.parse 语法检查
  - `programmatic` TR-5.2: diff -r 两个 tradingagents-analysis 目录结果为空
  - `programmatic` TR-5.3: install.mjs 路径正确
  - `programmatic` TR-5.4: grep 确认不包含 "olumnolume" 拼写错误
  - `programmatic` TR-5.5: 验证 _fmt_num 对数字返回字符串（通过简单的 Python 调用测试）
  - `human-judgement` TR-5.6: 代码审查确认所有修复符合现有代码风格
