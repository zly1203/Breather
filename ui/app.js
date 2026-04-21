const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

const noSession = document.getElementById("no-session");
const slider = document.getElementById("intensity-slider");
const blobContainer = document.getElementById("blob-container");
const brainStatus = document.getElementById("brain-status");

const sessionRow = document.getElementById("session-row");
const sessionValueEl = document.getElementById("session-value");
const interactionsRow = document.getElementById("interactions-row");
const interactionsValueEl = document.getElementById("interactions-value");
const todayRow = document.getElementById("today-row");
const todayValueEl = document.getElementById("today-value");

const STATE_MAP = {
  idle:  { blob: "calm",  status: "" },
  fresh: { blob: "fresh", status: "Deep in it. You got this." },
  tired: { blob: "tired", status: "The outside world misses you." },
};

// Legacy fallback: if backend doesn't provide character, infer from duration.
function inferCharacter(stats) {
  if (!stats) return "idle";
  if (stats.duration_minutes >= 60) return "tired";
  return "fresh";
}

let currentBlobState = null;

function formatMinutes(min) {
  if (min < 1) return "<1m";
  if (min < 60) return `${min}m`;
  const h = Math.floor(min / 60);
  const m = min % 60;
  return m > 0 ? `${h}h ${m}m` : `${h}h`;
}

function updateBlob(stats) {
  let stateKey;
  if (!stats) {
    stateKey = "idle";
  } else if (stats.character) {
    // Backend decides based on fatigue score vs. intensity-driven threshold.
    // "idle" from backend maps to the same key here.
    stateKey = stats.character === "idle" ? "idle" : stats.character;
  } else {
    // Fallback if backend doesn't send character yet.
    stateKey = inferCharacter(stats);
  }

  if (stateKey !== currentBlobState) {
    blobContainer.style.opacity = "0";
    setTimeout(() => {
      blobContainer.innerHTML = window.createBlobSVG(STATE_MAP[stateKey].blob);
      brainStatus.textContent = STATE_MAP[stateKey].status;
      blobContainer.style.opacity = "1";
    }, 300);
    currentBlobState = stateKey;
  }
}

function setRow(rowEl, valueEl, value, show) {
  if (show) {
    rowEl.classList.remove("hidden");
    if (valueEl && value !== undefined) valueEl.textContent = value;
  } else {
    rowEl.classList.add("hidden");
  }
}

async function refresh() {
  let sessionStats = null;
  let todayTotal = 0;

  try {
    const stats = await invoke("get_session_stats");
    if (stats && stats.duration_minutes !== undefined) {
      sessionStats = stats;
    }
  } catch {}

  try {
    todayTotal = await invoke("get_today_total");
  } catch {}

  // Header text + blob.
  if (sessionStats) {
    noSession.classList.add("hidden");
    updateBlob(sessionStats);
  } else {
    noSession.classList.remove("hidden");
    updateBlob(null);
  }

  // Activity card — always 3 rows, "—" for missing values.
  const sessionMin = sessionStats ? sessionStats.duration_minutes : 0;
  const hasSession = !!sessionStats;

  todayRow.classList.remove("hidden");
  todayValueEl.textContent = todayTotal > 0 ? formatMinutes(todayTotal) : "—";

  sessionRow.classList.remove("hidden");
  sessionValueEl.textContent = hasSession ? formatMinutes(sessionMin) : "—";

  interactionsRow.classList.remove("hidden");
  interactionsValueEl.textContent = hasSession ? sessionStats.interaction_count : "—";

}

async function loadIntensity() {
  try {
    const level = await invoke("get_intensity");
    slider.value = level;
  } catch {}
}

slider.addEventListener("input", async () => {
  const level = parseInt(slider.value);
  try {
    await invoke("set_intensity", { level });
    refresh();
  } catch {}
});

listen("session-updated", () => {
  refresh();
});

// Initial render.
updateBlob(null);
loadIntensity();
refresh();
setInterval(refresh, 10000);
