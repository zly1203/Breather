#!/usr/bin/env node

const fs = require("fs");
const path = require("path");
const os = require("os");

const VIBE_DIR = path.join(os.homedir(), ".breather");
const HOOK_FILE = path.join(VIBE_DIR, "hook.sh");
const CLAUDE_SETTINGS = path.join(os.homedir(), ".claude", "settings.json");

function isBreatherHook(h) {
  return h && h.command === HOOK_FILE;
}

function log(msg) {
  console.log(msg);
}

function loadJSON(filepath) {
  try {
    return JSON.parse(fs.readFileSync(filepath, "utf8"));
  } catch {
    return null;
  }
}

function saveJSON(filepath, data) {
  fs.mkdirSync(path.dirname(filepath), { recursive: true });
  fs.writeFileSync(filepath, JSON.stringify(data, null, 2) + "\n");
}

function hasVibeBreakHook(hooks) {
  if (!hooks) return false;
  return hooks.some((group) => group.hooks && group.hooks.some(isBreatherHook));
}

// ── Commands ─────────────────────────────────────────────────

function init() {
  log("");
  log("  Breather — setting up...");
  log("");

  // 1. Copy hook script to ~/.breather/
  fs.mkdirSync(VIBE_DIR, { recursive: true });
  const srcHook = path.join(__dirname, "..", "hook", "hook.sh");
  fs.copyFileSync(srcHook, HOOK_FILE);
  fs.chmodSync(HOOK_FILE, 0o755);
  log("  ✓ Installed hook to ~/.breather/hook.sh");

  // 2. Register in Claude Code settings.
  let settings = loadJSON(CLAUDE_SETTINGS) || {};
  if (!settings.hooks) settings.hooks = {};
  if (!settings.hooks.PostToolUse) settings.hooks.PostToolUse = [];

  if (!hasVibeBreakHook(settings.hooks.PostToolUse)) {
    settings.hooks.PostToolUse.push({
      matcher: "",
      hooks: [{ type: "command", command: HOOK_FILE }],
    });
    saveJSON(CLAUDE_SETTINGS, settings);
    log("  ✓ Registered hook in ~/.claude/settings.json");
  } else {
    log("  · Hook already registered — skipped.");
  }

  log("");
  log("  All set! Make sure the Breather app is running.");
  log("  The hook sends events to the app on each AI interaction.");
  log("");
  log("  Commands:");
  log("    breather status     — check connection to the app");
  log("    breather uninstall  — remove hook");
  log("");
}

async function status() {
  log("");
  try {
    const resp = await fetch("http://127.0.0.1:17422/status");
    const data = await resp.json();
    if (data.active === false) {
      log("  🟢 App is running. No active session.");
    } else {
      log(`  🟢 App is running.`);
      log(`     Session: ${formatMin(data.duration_minutes)}, ${data.interaction_count} interactions`);
      log(`     Today:   ${formatMin(data.today_total_minutes)}`);
    }
  } catch {
    log("  App is not running. Start the Breather app first.");
  }
  log("");
}

function uninstall() {
  log("");
  log("  Removing Breather hook...");

  let settings = loadJSON(CLAUDE_SETTINGS);
  if (settings && settings.hooks) {
    for (const eventType of Object.keys(settings.hooks)) {
      settings.hooks[eventType] = settings.hooks[eventType].filter(
        (group) => !(group.hooks && group.hooks.some(isBreatherHook))
      );
      if (settings.hooks[eventType].length === 0) delete settings.hooks[eventType];
    }
    if (Object.keys(settings.hooks).length === 0) delete settings.hooks;
    saveJSON(CLAUDE_SETTINGS, settings);
    log("  ✓ Removed hook from settings.");
  }

  if (fs.existsSync(VIBE_DIR)) {
    fs.rmSync(VIBE_DIR, { recursive: true });
    log("  ✓ Removed ~/.breather/");
  }

  log("");
  log("  Hook removed. The app can be uninstalled separately.");
  log("");
}

function formatMin(min) {
  if (min < 60) return `${min}m`;
  const h = Math.floor(min / 60);
  const m = min % 60;
  return m > 0 ? `${h}h ${m}m` : `${h}h`;
}

const command = process.argv[2];

switch (command) {
  case "init":
    init();
    break;
  case "status":
    status();
    break;
  case "uninstall":
    uninstall();
    break;
  default:
    log("");
    log("  Breather");
    log("");
    log("  Usage:");
    log("    breather init       — install Claude Code hook");
    log("    breather status     — check app connection");
    log("    breather uninstall  — remove hook");
    log("");
    break;
}
