#!/usr/bin/env node

const os = require('os');
const fs = require('fs');
const path = require('path');
const https = require('https');
const { spawnSync } = require('child_process');

const VERSION = '0.0.1-beta-2';
const GITHUB_REPO = 'peekstore/tasklynx';

function getBinaryName() {
  const platform = os.platform();
  const arch = os.arch();

  let osName;
  if (platform === 'linux') osName = 'linux';
  else if (platform === 'darwin') osName = 'darwin';
  else if (platform === 'win32') osName = 'windows';
  else throw new Error(`Unsupported OS platform: ${platform}`);

  let archName;
  if (arch === 'x64') archName = 'amd64';
  else if (arch === 'arm64' || arch === 'arm') archName = 'arm64';
  else throw new Error(`Unsupported architecture: ${arch}`);

  const ext = osName === 'windows' ? '.exe' : '';
  return `tasklynx-${osName}-${archName}${ext}`;
}

function getCachePath(binaryName) {
  const baseDir = process.env.TASKLYNX_HOME
    ? path.join(process.env.TASKLYNX_HOME, 'bin')
    : path.join(os.homedir(), '.local', 'share', 'tasklynx', 'bin');

  const versionDir = path.join(baseDir, `v${VERSION}`);
  fs.mkdirSync(versionDir, { recursive: true });
  const isWindows = os.platform() === 'win32';
  return path.join(versionDir, isWindows ? 'tasklynx.exe' : 'tasklynx');
}

function download(url, dest) {
  return new Promise((resolve, reject) => {
    https.get(url, (res) => {
      if (res.statusCode >= 300 && res.statusCode < 400 && res.headers.location) {
        return download(res.headers.location, dest).then(resolve).catch(reject);
      }
      if (res.statusCode !== 200) {
        return reject(new Error(`Failed to download binary: HTTP ${res.statusCode} from ${url}`));
      }

      const fileStream = fs.createWriteStream(dest);
      res.pipe(fileStream);
      fileStream.on('finish', () => {
        fileStream.close(() => {
          fs.chmodSync(dest, 0o755);
          resolve();
        });
      });
      fileStream.on('error', (err) => {
        try { fs.unlinkSync(dest); } catch (_) {}
        reject(err);
      });
    }).on('error', reject);
  });
}

async function main() {
  const binaryName = getBinaryName();
  const binPath = getCachePath(binaryName);

  if (!fs.existsSync(binPath) || fs.statSync(binPath).size === 0) {
    const downloadUrl = `https://github.com/${GITHUB_REPO}/releases/download/v${VERSION}/${binaryName}`;
    process.stderr.write(`Downloading tasklynx v${VERSION} for ${os.platform()}-${os.arch()}...\n`);
    try {
      await download(downloadUrl, binPath);
    } catch (err) {
      process.stderr.write(`Error downloading tasklynx: ${err.message}\n`);
      process.exit(1);
    }
  }

  const result = spawnSync(binPath, process.argv.slice(2), {
    stdio: 'inherit'
  });

  if (result.error) {
    process.stderr.write(`Failed to execute tasklynx: ${result.error.message}\n`);
    process.exit(1);
  }

  process.exit(result.status !== null ? result.status : 0);
}

main().catch((err) => {
  process.stderr.write(`${err.stack || err.message}\n`);
  process.exit(1);
});
