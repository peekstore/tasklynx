const { spawn } = require('child_process');
const http = require('http');
const fs = require('fs');
const path = require('path');
const puppeteer = require('puppeteer-core');

const ROOT_DIR = path.resolve(__dirname, '..');
const SCREENSHOTS_DIR = path.join(ROOT_DIR, 'Screenshots');
const DATA_DIR = path.join(ROOT_DIR, 'target', 'screenshot_data');
const PORT = 8989;

// Ensure output dirs exist
fs.mkdirSync(SCREENSHOTS_DIR, { recursive: true });
fs.mkdirSync(DATA_DIR, { recursive: true });

// Realistic demo database
const sampleData = {
  version: 1,
  current_branch: "main",
  branches: ["main", "feature/auth-v2", "release/v0.1.0"],
  tasks: [
    {
      id: "7b4e9f12-0000-4000-8000-000000000001",
      title: "Implement OAuth2 / OIDC authentication flow",
      description: "Support GitHub, Google, and Gitlab identity providers with secure state tokens.",
      project: "auth",
      tags: ["security", "api", "oauth"],
      status: "doing",
      priority: "urgent",
      due: "2026-11-05T23:59:59Z",
      branch: "main",
      commit: null,
      parent: null,
      estimate_minutes: 180,
      created_at: "2026-10-27T10:00:00Z",
      updated_at: "2026-10-28T09:30:00Z",
      completed_at: null
    },
    {
      id: "7b4e9f12-0000-4000-8000-000000000002",
      title: "Add SQLite / PostgreSQL storage backend adapter",
      description: "Abstract store interface to support both single-file JSON and relational databases.",
      project: "storage",
      tags: ["database", "backend", "sql"],
      status: "todo",
      priority: "high",
      due: "2026-11-12T23:59:59Z",
      branch: "main",
      commit: null,
      parent: null,
      estimate_minutes: 240,
      created_at: "2026-10-27T11:00:00Z",
      updated_at: "2026-10-27T11:00:00Z",
      completed_at: null
    },
    {
      id: "7b4e9f12-0000-4000-8000-000000000003",
      title: "Design terminal-native Ratatui Kanban layout",
      description: "Full-screen keyboard-navigable 3-column board with vim keybindings.",
      project: "tui",
      tags: ["ui", "ratatui", "terminal"],
      status: "done",
      priority: "high",
      due: "2026-10-28T18:00:00Z",
      branch: "main",
      commit: "3536f697",
      parent: null,
      estimate_minutes: 120,
      created_at: "2026-10-27T12:00:00Z",
      updated_at: "2026-10-28T14:00:00Z",
      completed_at: "2026-10-28T14:00:00Z"
    },
    {
      id: "7b4e9f12-0000-4000-8000-000000000004",
      title: "Multi-platform distribution (APT, npm, pipx, choco, cargo)",
      description: "Automate packaging across APT, npm, pipx, Chocolatey, Cargo, and shell installers.",
      project: "packaging",
      tags: ["devops", "ci", "release"],
      status: "doing",
      priority: "high",
      due: "2026-10-29T23:59:59Z",
      branch: "main",
      commit: null,
      parent: null,
      estimate_minutes: 90,
      created_at: "2026-10-28T08:00:00Z",
      updated_at: "2026-10-28T15:00:00Z",
      completed_at: null
    },
    {
      id: "7b4e9f12-0000-4000-8000-000000000005",
      title: "Cross-process file lock with timeout and stale recovery",
      description: "Advisory flock implementation preventing concurrent write race conditions.",
      project: "core",
      tags: ["concurrency", "safety", "fs"],
      status: "done",
      priority: "urgent",
      due: "2026-10-28T16:00:00Z",
      branch: "main",
      commit: "9028283f",
      parent: null,
      estimate_minutes: 60,
      created_at: "2026-10-27T14:00:00Z",
      updated_at: "2026-10-28T16:30:00Z",
      completed_at: "2026-10-28T16:30:00Z"
    },
    {
      id: "7b4e9f12-0000-4000-8000-000000000006",
      title: "Interactive Web UI drag-and-drop task reordering",
      description: "Allow moving cards between columns with HTML5 Drag & Drop API and automatic sync.",
      project: "web",
      tags: ["frontend", "drag-drop"],
      status: "todo",
      priority: "normal",
      due: "2026-11-20T23:59:59Z",
      branch: "main",
      commit: null,
      parent: null,
      estimate_minutes: 150,
      created_at: "2026-10-28T09:00:00Z",
      updated_at: "2026-10-28T09:00:00Z",
      completed_at: null
    },
    {
      id: "7b4e9f12-0000-4000-8000-000000000007",
      title: "Shell completion generators (Bash, Zsh, Fish, PowerShell)",
      description: "Generate static completions via clap_complete.",
      project: "cli",
      tags: ["cli", "ux", "completions"],
      status: "done",
      priority: "normal",
      due: "2026-10-28T17:00:00Z",
      branch: "main",
      commit: "dfc6050a",
      parent: null,
      estimate_minutes: 45,
      created_at: "2026-10-27T16:00:00Z",
      updated_at: "2026-10-28T17:00:00Z",
      completed_at: "2026-10-28T17:00:00Z"
    }
  ]
};

// Write tasks.json
fs.writeFileSync(path.join(DATA_DIR, 'tasks.json'), JSON.stringify(sampleData, null, 2));

function waitForServer(port, retries = 30) {
  return new Promise((resolve, reject) => {
    const check = (remaining) => {
      const req = http.get(`http://127.0.0.1:${port}/`, (res) => {
        if (res.statusCode === 200) return resolve();
        setTimeout(() => check(remaining - 1), 200);
      });
      req.on('error', () => {
        if (remaining <= 0) return reject(new Error('Server did not start in time'));
        setTimeout(() => check(remaining - 1), 200);
      });
    };
    check(retries);
  });
}

(async () => {
  console.log('Starting Tasklynx web server...');
  const binary = path.join(ROOT_DIR, 'target', 'release', 'tasklynx');
  const server = spawn(binary, ['web', '--port', String(PORT), '--host', '127.0.0.1'], {
    env: {
      ...process.env,
      TASKLYNX_HOME: DATA_DIR,
    },
    stdio: 'inherit'
  });

  try {
    await waitForServer(PORT);
    console.log(`Server listening on http://127.0.0.1:${PORT}`);

    console.log('Launching headless Chromium with Puppeteer...');
    const chromeCandidates = [
      process.env.CHROME_BIN,
      process.env.CHROMIUM_PATH,
      '/usr/bin/chromium',
      '/usr/bin/chromium-browser',
      '/usr/bin/google-chrome',
      '/usr/bin/google-chrome-stable'
    ].filter(Boolean);
    const executablePath = chromeCandidates.find(p => fs.existsSync(p)) || '/usr/bin/chromium';

    const browser = await puppeteer.launch({
      executablePath,
      args: [
        '--no-sandbox',
        '--disable-setuid-sandbox',
        '--disable-dev-shm-usage',
        '--disable-gpu',
        '--window-size=1600,1000'
      ],
      defaultViewport: {
        width: 1500,
        height: 950,
        deviceScaleFactor: 2
      }
    });

    const page = await browser.newPage();
    await page.goto(`http://127.0.0.1:${PORT}`, { waitUntil: 'networkidle0' });
    await new Promise(r => setTimeout(r, 1000));

    // 1. Full Web Dashboard screenshot
    const webPath = path.join(SCREENSHOTS_DIR, 'web.png');
    await page.screenshot({ path: webPath, fullPage: false });
    console.log(`Saved ${webPath}`);

    // 2. Focused Kanban Board screenshot
    const boardPath = path.join(SCREENSHOTS_DIR, 'board.png');
    const boardElement = await page.$('.board');
    if (boardElement) {
      await boardElement.screenshot({ path: boardPath });
    } else {
      await page.screenshot({ path: boardPath });
    }
    console.log(`Saved ${boardPath}`);

    // 3. Modal / Add Task UI screenshot
    await page.click('header.top button'); // Click "+ add"
    await new Promise(r => setTimeout(r, 600));

    // Type sample into form so it looks like an active user session
    await page.type('#title', 'Refactor CLI flags with clap derive');
    await page.type('#description', 'Standardize short and long argument flags across all subcommands.');
    await page.type('#tags', 'cli,refactor');

    const homePath = path.join(SCREENSHOTS_DIR, 'home.png');
    await page.screenshot({ path: homePath });
    console.log(`Saved ${homePath}`);

    await browser.close();
    console.log('All real screenshots captured successfully!');
  } finally {
    server.kill();
    // Clean up temporary data
    fs.rmSync(DATA_DIR, { recursive: true, force: true });
  }
})();
