/**
 * Character states with fade transitions.
 *   idle  → cup  (tea cup, no session)
 *   fresh → lamp (working, fatigue below threshold)
 *   tired → cat  (fatigue at or above threshold; slider controls the cutoff)
 */

const BLOB_IMAGES = {
  calm: "cup.png",
  fresh: "lamp.png",
  tired: "cat.png",
};

function createBlobSVG(stateKey) {
  const src = BLOB_IMAGES[stateKey];
  if (!src) return "";
  return `<img src="${src}" alt="" class="blob-img blob-${stateKey}" draggable="false">`;
}

// Export for use in app.js
window.createBlobSVG = createBlobSVG;
