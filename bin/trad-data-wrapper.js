#!/usr/bin/env node
// trad-data wrapper: 检测平台并调用对应的预编译二进制
const { execFileSync } = require('node:child_process');
const path = require('node:path');
const os = require('node:os');

const platform = os.platform(); // win32, darwin, linux
const arch = os.arch(); // x64, arm64

const platformDir = `${platform}-${arch}`;
const ext = platform === 'win32' ? '.exe' : '';
const binaryPath = path.join(__dirname, platformDir, `trad-data${ext}`);

try {
  execFileSync(binaryPath, process.argv.slice(2), { stdio: 'inherit' });
} catch (e) {
  if (e.code === 'ENOENT') {
    console.error(`trad-data binary not found for ${platform}-${arch}`);
    console.error('Falling back to Python scripts. Install with: pip install yfinance akshare requests pandas');
    process.exit(1);
  }
  throw e;
}
