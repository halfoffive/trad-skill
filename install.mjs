#!/usr/bin/env node
// trad-skill 安装器：把 tradingagents-analysis/ 技能复制到目标 agent 的 skills 目录。
// 零依赖、纯 Node ESM。用法：npx halfoffive/trad-skill [--dir <path>] [--agent claude|agents|opencode]

import { fileURLToPath } from 'node:url';
import path from 'node:path';
import os from 'node:os';
import fs from 'node:fs';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const home = os.homedir();

// 源技能目录：本脚本同级的 tradingagents-analysis/
const SKILL_NAME = 'tradingagents-analysis';
const SRC_DIR = path.join(__dirname, SKILL_NAME);

// 不同 agent 的默认 skills 目录
const AGENT_DIRS = {
  claude: path.join(home, '.claude', 'skills'),
  agents: path.join(home, '.agents', 'skills'),
  opencode: path.join(home, '.config', 'opencode', 'skills'),
};

function parseArgs(argv) {
  const out = { dir: null, agent: null };
  for (let i = 0; i < argv.length; i++) {
    let a = argv[i];
    let inline = null;
    const eq = a.indexOf('=');
    if (eq > 0 && a.startsWith('--')) {
      inline = a.slice(eq + 1);
      a = a.slice(0, eq);
    }
    if (a === '--dir') {
      const v = inline !== null ? inline : argv[++i];
      if (v === undefined || v === '' || v.startsWith('--')) fail('--dir 需要一个路径参数');
      out.dir = v;
    } else if (a === '--agent') {
      const v = inline !== null ? inline : argv[++i];
      if (v === undefined || v === '' || v.startsWith('--')) fail('--agent 需要一个名称（claude|agents|opencode）');
      if (!AGENT_DIRS[v]) fail(`未知 --agent: ${v} (支持: ${Object.keys(AGENT_DIRS).join('|')})`);
      out.agent = v;
    } else if (a === '-h' || a === '--help') {
      out.help = true;
    } else {
      fail(`未知参数：${a}`);
    }
  }
  return out;
}

function help() {
  console.log(`trad-skill — 安装 tradingagents-analysis 技能

用法:
  npx halfoffive/trad-skill              安装到 ~/.claude/skills (Claude Code)
  npx halfoffive/trad-skill --agent agents   安装到 ~/.agents/skills
  npx halfoffive/trad-skill --dir <path> 安装到指定目录

选项:
  --dir <path>        目标 skills 目录的父目录
  --agent <name>      claude | agents | opencode
  -h, --help          显示帮助
`);
}

function fail(msg) {
  console.error(`✗ ${msg}`);
  console.error('  运行 npx halfoffive/trad-skill --help 查看用法。');
  process.exit(1);
}

const args = parseArgs(process.argv.slice(2));
if (args.help) { help(); process.exit(0); }

// 校验源目录存在且包含 SKILL.md
if (!fs.existsSync(SRC_DIR)) {
  fail(`找不到打包的技能目录：${SRC_DIR}（安装包可能不完整）`);
}
const srcSkill = path.join(SRC_DIR, 'SKILL.md');
if (!fs.existsSync(srcSkill)) {
  fail(`${SRC_DIR} 下缺少 SKILL.md，不是有效的技能包`);
}

// 确定目标父目录
let parentDir;
if (args.dir && args.agent) {
  // --dir 和 --agent 语义互斥：--dir 是任意自定义路径，--agent 是预置目标。
  // 同时指定会让"用哪个"产生歧义（曾有人 --dir foo --agent opencode 期望装到 foo
  // 但实际走 --agent 分支装到 ~/.config/opencode/skills）。直接 fail，不静默选一个。
  fail('不能同时指定 --dir 和 --agent（--dir 为自定义路径，--agent 为预置目标）');
}
if (args.dir) {
  // 展开 ~ 为 home 目录
  if (args.dir === '~') args.dir = home;
  else if (args.dir.startsWith('~/')) args.dir = path.join(home, args.dir.slice(2));
  else if (args.dir.startsWith('~\\')) args.dir = path.join(home, args.dir.slice(2));
  parentDir = args.dir;
} else if (args.agent) {
  parentDir = AGENT_DIRS[args.agent];
  if (!parentDir) fail(`未知 --agent 值：${args.agent}（可选：claude | agents | opencode）`);
} else {
  // 默认 Claude Code
  parentDir = AGENT_DIRS.claude;
}

// 用 path.resolve 而非 path.join：--dir ./foo 相对路径时 destDir 也是绝对路径，
// 与下方 scriptsDir 输出一致，避免 L126 显示相对路径误导用户（R6-28）
const destDir = path.resolve(parentDir, SKILL_NAME);

// 幂等：先删旧目录再复制（含 mkdirSync 父目录创建，统一在 try/catch 内）
// 旧版本 mkdirSync 在 try/catch 之外，权限不足 / 路径非法时会抛裸 Node 堆栈；
// 移入既有 try/catch 走 fail() 友好提示。
try {
  fs.mkdirSync(parentDir, { recursive: true });
  if (fs.existsSync(destDir)) {
    fs.rmSync(destDir, { recursive: true, force: true });
  }
  fs.cpSync(SRC_DIR, destDir, {
    recursive: true,
    filter: (src) => path.basename(src) !== '__pycache__',
  });
} catch (e) {
  fail(`安装失败：${e.message}`);
}

const scriptsDir = path.join(destDir, 'scripts');
console.log(`✓ 已安装 ${SKILL_NAME} → ${destDir}`);
console.log('');
console.log('下一步：');
console.log(`  1. 安装 Python 依赖（脚本运行需要）:`);
console.log('     pip install yfinance akshare requests pandas');
console.log('  2. 重启你的 AI agent / 开一个新会话，让它加载该技能。');
console.log('  3. 触发分析，例如："分析 AAPL" 或 "Analyze 600519" 。');
console.log('');
console.log('脚本（绝对路径，子代理调用时使用）:');
for (const s of ['fetch_stock_data.py', 'fetch_news.py', 'fetch_fundamentals.py', 'fetch_sentiment.py']) {
  // path.resolve 把 scriptsDir 与脚本名合并为绝对路径（相对 scriptsDir 时也能解析）
  // 旧版 path.join 在 parentDir 为相对路径（如 --dir ./foo）时输出 ./foo/.../script.py，
  // 子代理 CWD 不在仓库根会找不到。resolve 后始终是绝对路径，且用 '/' 分隔便于跨平台复制粘贴。
  const abs = path.resolve(scriptsDir, s).split(path.sep).join('/');
  console.log(`  python "${abs}" ...`);
}
