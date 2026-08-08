#!/usr/bin/env node
// trad-skill unified launcher: 极薄的 Node 分发器。
// 唯一职责：解析当前平台的预编译二进制（@trad-skill/<platform> optionalDependency），
// 然后 exec 它。所有逻辑（安装器 + 数据工具）均由 Rust 二进制实现
// （crates/trad-data/src/，产出的二进制名为 trad-skill）。
//
// 无参数 / 非数据子命令 → 默认 install 模式，注入 --skills-dir。
// 数据子命令 (stock/news/fundamentals/sentiment) → 原样透传，不注入 --skills-dir。
// 显式 `install` 同样注入 --skills-dir：Rust 二进制回落的 CARGO_MANIFEST_DIR
// 路径是构建机路径，在用户机器上不存在，必须显式给出包内技能目录。
//
// bunx trad-skill@latest                         → 安装到 ~/.agents/skills（默认）
// bunx trad-skill@latest --agent claude
// bunx trad-skill@latest install                 → 显式安装（同样注入 --skills-dir）
// bunx trad-skill@latest stock --symbol AAPL     → 直接取数
// npx trad-skill@latest                          → 同上（fallback）
const { execFileSync } = require('node:child_process');
const path = require('node:path');

const platform = process.platform; // win32, darwin, linux
const arch = process.arch; // x64, arm64
const platformDir = `${platform}-${arch}`;
const ext = platform === 'win32' ? '.exe' : '';
const binName = `trad-skill${ext}`;

// npm optionalDependency（bunx / npx / node_modules 上下文）
let npmBinary = null;
try {
  npmBinary = require.resolve(`@trad-skill/${platformDir}/${binName}`);
} catch {
  // 非 npm 上下文，或平台包未安装
}

if (!npmBinary) {
  console.error(`trad-skill: 未找到 ${platformDir} 平台的二进制文件。`);
  console.error('  重新安装: bunx trad-skill@latest  (或 npx trad-skill@latest)');
  console.error(`  或手动安装平台包: npm install @trad-skill/${platformDir}`);
  process.exit(1);
}

// 数据子命令：这些子命令下，launcher 不注入 --skills-dir（透传用户参数）。
const DATA_SUBCOMMANDS = new Set(['stock', 'news', 'fundamentals', 'sentiment', 'fund']);
const userArgs = process.argv.slice(2);
const isDataInvocation = userArgs[0] && DATA_SUBCOMMANDS.has(userArgs[0]);

// 包根目录（tarball 布局：<pkg>/{bin/, skills/, ...}）→ 注入源技能目录。
// 显式 `install` 同样注入：Rust 侧 clap 对重复 --skills-dir 是后者胜，
// 用户自带的 --skills-dir（在注入参数之后）会被保留。
const pkgRoot = path.resolve(__dirname, '..');
const skillsDir = path.join(pkgRoot, 'skills', 'tradingagents-analysis');

const forwardedArgs = isDataInvocation
  ? userArgs
  : [
      // 显式 `install` 可能出现在任意位置（如 `--agent claude install`）：
      // 仅当第一个位置参数是 `install` 时才剥离，避免误删 --dir install 等合法值。
      'install',
      '--skills-dir',
      skillsDir,
      ...userArgs.filter((a, i) => !(i === 0 && a === 'install')),
    ];

try {
  execFileSync(npmBinary, forwardedArgs, { stdio: 'inherit' });
} catch (e) {
  if (e.status !== undefined && e.status !== null) {
    process.exit(e.status);
  }
  if (e.code === 'ENOENT') {
    console.error(`trad-skill binary not found at ${npmBinary}. Please reinstall: bunx trad-skill@latest`);
    process.exit(1);
  }
  if (e.code === 'EACCES') {
    console.error(`trad-skill binary at ${npmBinary} is not executable.`);
    console.error(`  Try: chmod +x "${npmBinary}"`);
    process.exit(1);
  }
  throw e;
}
