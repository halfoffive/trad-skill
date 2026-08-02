#!/usr/bin/env node
// trad-skill unified launcher: 极薄的 Node 分发器。
// 唯一职责：解析当前平台的预编译二进制（@trad-skill/<platform> optionalDependency），
// 然后 exec 它。所有逻辑（安装器 + 数据工具）均由 Rust 二进制实现
// （crates/trad-data/src/，产出的二进制名为 trad-skill）。
//
// 无参数 / 非数据子命令 → 默认 install 模式，注入 --skills-dir。
// 数据子命令 (stock/news/fundamentals/sentiment/install) → 原样透传，不注入 --skills-dir。
//
// bunx trad-skill@latest                         → 安装到 ~/.agents/skills（默认）
// bunx trad-skill@latest --agent claude
// bunx trad-skill@latest stock --symbol AAPL     → 直接取数
// npx trad-skill@latest                          → 同上（fallback）
const { execFileSync } = require('node:child_process');
const path = require('node:path');
const fs = require('node:fs');

const platform = process.platform; // win32, darwin, linux
const arch = process.arch; // x64, arm64
const platformDir = `${platform}-${arch}`;
const ext = platform === 'win32' ? '.exe' : '';
const binName = `trad-skill${ext}`;

// Strategy 1: 技能目录内由 install 复制的本地二进制（bin/<platform>/...）
const localBinary = path.join(__dirname, platformDir, binName);

// Strategy 2: npm optionalDependency（bunx / npx / node_modules 上下文）
let npmBinary = null;
try {
  npmBinary = require.resolve(`@trad-skill/${platformDir}/${binName}`);
} catch {
  // 非 npm 上下文，或平台包未安装
}

const binaryPath = fs.existsSync(localBinary) ? localBinary : npmBinary;

if (!binaryPath) {
  console.error(`trad-skill: 未找到 ${platformDir} 平台的二进制文件。`);
  console.error('  重新安装: bunx trad-skill@latest  (或 npx trad-skill@latest)');
  console.error(`  或手动安装平台包: npm install @trad-skill/${platformDir}`);
  process.exit(1);
}

// 识别数据子命令：这些子命令下，launcher 不注入 --skills-dir（透传用户参数）。
// `install` 也在此列：显式 install 时用户自己负责参数。
const DATA_SUBCOMMANDS = new Set(['stock', 'news', 'fundamentals', 'sentiment', 'install']);
const firstUserArg = process.argv[2];
const isDataInvocation = firstUserArg && DATA_SUBCOMMANDS.has(firstUserArg);

// 包根目录（tarball 布局：<pkg>/{bin/, skills/, ...}）→ 注入源技能目录
const pkgRoot = path.resolve(__dirname, '..');
const skillsDir = path.join(pkgRoot, 'skills', 'tradingagents-analysis');

const forwardedArgs = isDataInvocation
  ? process.argv.slice(2)
  : ['install', '--skills-dir', skillsDir, ...process.argv.slice(2)];

try {
  execFileSync(binaryPath, forwardedArgs, { stdio: 'inherit' });
} catch (e) {
  if (e.status !== undefined && e.status !== null) {
    process.exit(e.status);
  }
  if (e.code === 'ENOENT') {
    console.error(`trad-skill binary not found at ${binaryPath}. Please reinstall: bunx trad-skill@latest`);
    process.exit(1);
  }
  throw e;
}
