#!/usr/bin/env node
// trad-data wrapper: 检测平台并调用对应的预编译二进制
// 双路径解析：1) install 子命令复制到技能目录的本地二进制  2) npm optionalDependency 平台包
const { execFileSync } = require('node:child_process');
const path = require('node:path');
const fs = require('node:fs');

const platform = process.platform; // win32, darwin, linux
const arch = process.arch; // x64, arm64
const platformDir = `${platform}-${arch}`;
const ext = platform === 'win32' ? '.exe' : '';

// Strategy 1: binary installed by install.mjs into skill's bin/<platform>/
const localBinary = path.join(__dirname, platformDir, `trad-data${ext}`);

// Strategy 2: npm optionalDependency (when run via npx or from node_modules)
let npmBinary = null;
try {
  npmBinary = require.resolve(`@trad-skill/${platformDir}/trad-data${ext}`);
} catch (e) {
  // Not in npm context or platform package not installed
}

const binaryPath = fs.existsSync(localBinary) ? localBinary : npmBinary;

if (!binaryPath) {
  console.error(`trad-data: 未找到 ${platformDir} 平台的二进制文件。`);
  console.error('  重新安装: npx trad-skill');
  console.error(`  或手动安装平台包: npm install @trad-skill/${platformDir}`);
  process.exit(1);
}

try {
  execFileSync(binaryPath, process.argv.slice(2), { stdio: 'inherit' });
} catch (e) {
  if (e.code === 'ENOENT') {
    console.error(`trad-data binary not found at ${binaryPath}. Please reinstall: npx trad-skill`);
    process.exit(1);
  }
  throw e;
}
