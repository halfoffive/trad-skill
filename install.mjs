#!/usr/bin/env node
// trad-skill 安装器：把 tradingagents-analysis/ 技能复制到目标 agent 的 skills 目录。
// 零依赖、纯 Node ESM。用法：npx halfoffive/trad-skill [--dir <path>] [--agent claude|agents|opencode]

import { fileURLToPath } from 'node:url';
import { createRequire } from 'node:module';
import path from 'node:path';
import os from 'node:os';
import fs from 'node:fs';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const home = os.homedir();

// 源技能目录：本仓库 skills/ 下的 tradingagents-analysis/
const SKILL_NAME = 'tradingagents-analysis';
const SRC_DIR = path.join(__dirname, 'skills', SKILL_NAME);

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
const platformKey = `${os.platform()}-${os.arch()}`;
const binExt = os.platform() === 'win32' ? '.exe' : '';
let binarySource = null;

try {
  fs.mkdirSync(parentDir, { recursive: true });
  if (fs.existsSync(destDir)) {
    fs.rmSync(destDir, { recursive: true, force: true });
  }
  fs.cpSync(SRC_DIR, destDir, { recursive: true });

  // 复制 trad-data wrapper（始终需要）
  const srcWrapper = path.join(__dirname, 'bin', 'trad-data-wrapper.js');
  const destBinDir = path.join(destDir, 'bin');
  fs.mkdirSync(destBinDir, { recursive: true });
  fs.copyFileSync(srcWrapper, path.join(destBinDir, 'trad-data-wrapper.js'));

  // 从 npm optionalDependency 平台包解析二进制
  const require2 = createRequire(import.meta.url);
  try {
    binarySource = require2.resolve(`@trad-skill/${platformKey}/trad-data${binExt}`);
  } catch (e) {
    // 平台包未安装（可选依赖可能因网络/平台不匹配而跳过）
  }

  if (binarySource) {
    const destPlatformDir = path.join(destBinDir, platformKey);
    fs.mkdirSync(destPlatformDir, { recursive: true });
    fs.copyFileSync(binarySource, path.join(destPlatformDir, `trad-data${binExt}`));
  }
} catch (e) {
  fail(`安装失败：${e.message}`);
}

console.log(`✓ 已安装 ${SKILL_NAME} → ${destDir}`);
console.log('');
console.log('下一步：');
if (binarySource) {
  console.log(`  ✓ trad-data 二进制已安装 (${platformKey})`);
} else {
  console.log(`  ⚠ 平台包 @trad-skill/${platformKey} 未安装，数据工具不可用。`);
  console.log(`    运行 npm install @trad-skill/${platformKey} 或重新安装。`);
}
console.log('  1. 重启你的 AI agent / 开一个新会话，让它加载该技能。');
console.log('  2. 触发分析，例如："分析 AAPL" 或 "Analyze 600519" 。');
console.log('');
console.log('数据工具（Rust二进制）:');
console.log('  trad-data market --symbol AAPL');
console.log('  trad-data fundamentals --symbol AAPL');
console.log('  trad-data news --symbol AAPL');
console.log('  trad-data sentiment --symbol AAPL');
