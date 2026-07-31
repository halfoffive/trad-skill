#!/usr/bin/env node
// trad-skill launcher: 极薄的 Node 分发器。
// 唯一职责：解析当前平台的预编译二进制（@trad-skill/<platform> optionalDependency），
// 然后以 `install` 子命令 + --skills-dir 调用它。真正的安装逻辑全部由 Rust 实现
// （crates/trad-data/src/install.rs 的 install 子命令）。
//
// bunx trad-skill          → 安装到 ~/.agents/skills（默认通用 agent 目录）
// bunx trad-skill --agent claude
// npx trad-skill           → 同上（fallback）
const { execFileSync } = require('node:child_process');
const path = require('node:path');
const fs = require('node:fs');

const platform = process.platform; // win32, darwin, linux
const arch = process.arch; // x64, arm64
const platformDir = `${platform}-${arch}`;
const ext = platform === 'win32' ? '.exe' : '';

// Strategy 1: 技能目录内由 install 子命令复制的本地二进制（bin/<platform>/...）
const localBinary = path.join(__dirname, platformDir, `trad-data${ext}`);

// Strategy 2: npm optionalDependency（bunx / npx / node_modules 上下文）
let npmBinary = null;
try {
  npmBinary = require.resolve(`@trad-skill/${platformDir}/trad-data${ext}`);
} catch (e) {
  // 非 npm 上下文，或平台包未安装（optionalDependency 可能因网络/平台不匹配跳过）
}

const binaryPath = fs.existsSync(localBinary) ? localBinary : npmBinary;

if (!binaryPath) {
  console.error(`trad-skill: 未找到 ${platformDir} 平台的二进制文件。`);
  console.error('  重新安装: bunx trad-skill  (或 npx trad-skill)');
  console.error(`  或手动安装平台包: npm install @trad-skill/${platformDir}`);
  process.exit(1);
}

// 包根目录（tarball 布局：<pkg>/{bin/, skills/, ...}）→ 注入源技能目录
const pkgRoot = path.resolve(__dirname, '..');
const skillsDir = path.join(pkgRoot, 'skills', 'tradingagents-analysis');

try {
  execFileSync(
    binaryPath,
    ['install', '--skills-dir', skillsDir, ...process.argv.slice(2)],
    { stdio: 'inherit' },
  );
} catch (e) {
  if (e.status !== undefined && e.status !== null) {
    // 子进程已自行打印错误，按其退出码退出
    process.exit(e.status);
  }
  if (e.code === 'ENOENT') {
    console.error(`trad-data binary not found at ${binaryPath}. Please reinstall: bunx trad-skill`);
    process.exit(1);
  }
  throw e;
}
