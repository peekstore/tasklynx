#!/usr/bin/env node

'use strict';

const os = require('os');
const fs = require('fs');
const path = require('path');
const https = require('https');
const http = require('http');
const { spawnSync } = require('child_process');

const VERSION = '0.0.1-beta-4';
const GITHUB_REPO = 'peekstore/tasklynx';
const MIN_BINARY_SIZE = 100 * 1024; // 100 KB sanity check

// ---------------------------------------------------------------------------
// Platform / architecture detection
// ---------------------------------------------------------------------------

function isTermux() {
  if (process.env.TERMUX_VERSION) return true;
  try {
    return fs.existsSync('/data/data/com.termux');
  } catch (_) {
    return false;
  }
}

/**
 * Returns { binaryName, isWindows }
 * binaryName is the bare filename as published on GitHub Releases,
 * e.g. "tasklynx-linux-amd64", "tasklynx-android-arm64-v8a", "tasklynx-windows-amd64.exe"
 */
function getBinaryName() {
  const arch = os.arch();       // x64, arm64, arm, ia32, …
  const platform = os.platform(); // linux, darwin, win32, android (rare)

  if (isTermux()) {
    // Android / Termux — map Node arch → Android ABI
    let abi;
    if (arch === 'arm64') abi = 'arm64-v8a';
    else if (arch === 'arm') abi = 'armeabi-v7a';
    else if (arch === 'x64') abi = 'x86_64';
    else throw new Error(
      `Unsupported Android architecture: ${arch}. ` +
      'Supported: arm64 (arm64-v8a), arm (armeabi-v7a), x64 (x86_64).'
    );
    return { binaryName: `tasklynx-android-${abi}`, isWindows: false };
  }

  let osName;
  if (platform === 'linux') osName = 'linux';
  else if (platform === 'darwin') osName = 'darwin';
  else if (platform === 'win32') osName = 'windows';
  else throw new Error(
    `Unsupported OS platform: ${platform}. ` +
    'Supported platforms: linux, darwin, win32, Android/Termux.'
  );

  let archName;
  if (arch === 'x64') archName = 'amd64';
  else if (arch === 'arm64') archName = 'arm64';
  else if (arch === 'arm') archName = 'arm64'; // best effort fallback
  else throw new Error(
    `Unsupported architecture: ${arch} on ${osName}. ` +
    'Supported: x64 (amd64), arm64.'
  );

  const isWindows = osName === 'windows';
  const ext = isWindows ? '.exe' : '';
  return { binaryName: `tasklynx-${osName}-${archName}${ext}`, isWindows };
}

// ---------------------------------------------------------------------------
// Cache / install directory  (~/.config/peekstore/tasklynx/bin)
// ---------------------------------------------------------------------------

function getInstallDir() {
  const platform = os.platform();
  if (isTermux() || platform === 'linux' || platform === 'darwin') {
    const xdg = process.env.XDG_CONFIG_HOME || path.join(os.homedir(), '.config');
    return path.join(xdg, 'peekstore', 'tasklynx', 'bin');
  }
  if (platform === 'win32') {
    const appData = process.env.APPDATA || path.join(os.homedir(), 'AppData', 'Roaming');
    return path.join(appData, 'peekstore', 'tasklynx', 'bin');
  }
  // Fallback
  return path.join(os.homedir(), '.config', 'peekstore', 'tasklynx', 'bin');
}

function getBinPath(isWindows) {
  const dir = getInstallDir();
  fs.mkdirSync(dir, { recursive: true });
  return path.join(dir, isWindows ? 'tasklynx.exe' : 'tasklynx');
}

function getTxPath(isWindows) {
  const dir = getInstallDir();
  return path.join(dir, isWindows ? 'tx.exe' : 'tx');
}

// ---------------------------------------------------------------------------
// Download with progress and redirect following
// ---------------------------------------------------------------------------

/**
 * Download `url` to `dest`, printing progress to stderr.
 * Follows HTTP 3xx redirects automatically (up to 10 hops).
 */
function download(url, dest, redirectsLeft = 10) {
  return new Promise((resolve, reject) => {
    if (redirectsLeft === 0) {
      return reject(new Error('Too many redirects while downloading binary.'));
    }

    const lib = url.startsWith('https') ? https : http;
    const req = lib.get(url, (res) => {
      // Follow redirects
      if (res.statusCode >= 300 && res.statusCode < 400 && res.headers.location) {
        res.resume(); // drain
        return download(res.headers.location, dest, redirectsLeft - 1)
          .then(resolve)
          .catch(reject);
      }

      if (res.statusCode !== 200) {
        res.resume();
        return reject(new Error(
          `HTTP ${res.statusCode} downloading binary from:\n  ${url}\n` +
          'Check that the release exists and the version is correct.'
        ));
      }

      const total = parseInt(res.headers['content-length'] || '0', 10);
      let received = 0;
      let lastPercent = -1;

      const fileStream = fs.createWriteStream(dest);

      res.on('data', (chunk) => {
        received += chunk.length;
        if (total > 0) {
          const pct = Math.floor((received / total) * 100);
          if (pct !== lastPercent) {
            lastPercent = pct;
            process.stderr.write(`\r  Progress: ${pct}% (${formatBytes(received)} / ${formatBytes(total)})  `);
          }
        } else {
          process.stderr.write(`\r  Downloaded: ${formatBytes(received)}  `);
        }
      });

      res.pipe(fileStream);

      fileStream.on('finish', () => {
        process.stderr.write('\n');
        fileStream.close(() => {
          try { fs.chmodSync(dest, 0o755); } catch (_) {}
          resolve(received);
        });
      });

      fileStream.on('error', (err) => {
        try { fs.unlinkSync(dest); } catch (_) {}
        reject(err);
      });
    });

    req.on('error', (err) => {
      try { fs.unlinkSync(dest); } catch (_) {}
      reject(new Error(`Network error while downloading binary: ${err.message}`));
    });
  });
}

function formatBytes(bytes) {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(2)} MB`;
}

// ---------------------------------------------------------------------------
// Symlink / copy helpers
// ---------------------------------------------------------------------------

function ensureTxAlias(binPath, isWindows) {
  const txPath = getTxPath(isWindows);
  try {
    if (fs.existsSync(txPath)) {
      fs.unlinkSync(txPath);
    }
    if (isWindows) {
      // On Windows, copy the binary
      fs.copyFileSync(binPath, txPath);
    } else {
      fs.symlinkSync(binPath, txPath);
    }
  } catch (err) {
    // tx alias is non-fatal
    process.stderr.write(`Warning: could not create tx alias at ${txPath}: ${err.message}\n`);
  }
}

// ---------------------------------------------------------------------------
// Self-update: delete cached binary so it re-downloads on next invocation
// ---------------------------------------------------------------------------

function selfUpdate() {
  let { isWindows } = getBinaryName(); // may throw if unsupported platform
  const binPath = getBinPath(isWindows);
  const txPath = getTxPath(isWindows);

  let deleted = false;
  for (const p of [binPath, txPath]) {
    if (fs.existsSync(p)) {
      fs.unlinkSync(p);
      deleted = true;
    }
  }
  if (deleted) {
    process.stderr.write(`Removed cached binaries. Re-run tasklynx to download v${VERSION} fresh.\n`);
  } else {
    process.stderr.write(`No cached binary found (already clean). Run tasklynx to download v${VERSION}.\n`);
  }
  process.exit(0);
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

async function main() {
  const args = process.argv.slice(2);

  // --version flag (no binary needed)
  if (args.includes('--version') || args.includes('-V')) {
    process.stdout.write(`tasklynx v${VERSION}\n`);
    process.exit(0);
  }

  // self update (--self-update OR "self update" subcommand)
  if (
    args.includes('--self-update') ||
    (args.length >= 2 && args[0] === 'self' && args[1] === 'update')
  ) {
    selfUpdate(); // exits
  }

  let binaryName, isWindows;
  try {
    ({ binaryName, isWindows } = getBinaryName());
  } catch (err) {
    process.stderr.write(`Error: ${err.message}\n`);
    process.exit(1);
  }

  const binPath = getBinPath(isWindows);

  // Download if missing or too small
  const needsDownload =
    !fs.existsSync(binPath) ||
    fs.statSync(binPath).size < MIN_BINARY_SIZE;

  if (needsDownload) {
    const downloadUrl =
      `https://github.com/${GITHUB_REPO}/releases/download/v${VERSION}/${binaryName}`;
    process.stderr.write(
      `Downloading tasklynx v${VERSION} (${binaryName})...\n` +
      `  From: ${downloadUrl}\n` +
      `  To:   ${binPath}\n`
    );

    try {
      const bytes = await download(downloadUrl, binPath);
      if (bytes < MIN_BINARY_SIZE) {
        fs.unlinkSync(binPath);
        throw new Error(
          `Downloaded file is too small (${formatBytes(bytes)}). ` +
          'The release binary may not exist for this platform, or the URL is incorrect.'
        );
      }
      process.stderr.write(`Download complete (${formatBytes(bytes)}).\n`);
      ensureTxAlias(binPath, isWindows);
    } catch (err) {
      process.stderr.write(`\nError: ${err.message}\n`);
      process.exit(1);
    }
  }

  // Execute the cached binary
  const result = spawnSync(binPath, args, { stdio: 'inherit' });

  if (result.error) {
    process.stderr.write(
      `Failed to execute ${binPath}: ${result.error.message}\n` +
      'Try running with --self-update to re-download the binary.\n'
    );
    process.exit(1);
  }

  process.exit(result.status !== null ? result.status : 0);
}

main().catch((err) => {
  process.stderr.write(`Unexpected error: ${err.stack || err.message}\n`);
  process.exit(1);
});
