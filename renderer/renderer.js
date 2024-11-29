// renderer/renderer.js

import { loadPlaylists } from "./playlists.js";
import { setupUIListeners, renderPlaylists } from "./ui.js";

window.addEventListener("DOMContentLoaded", () => {
  console.log("DOM fully loaded and parsed. Running initialization...");
  try {
    loadPlaylists();
    console.log("Playlists loaded.");
    renderPlaylists();
    console.log("Playlists rendered.");
    setupUIListeners();
    console.log("UI listeners initialized.");
  } catch (error) {
    console.error("Error during initialization:", error);
  }
});

