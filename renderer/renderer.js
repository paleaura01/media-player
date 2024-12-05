// renderer/renderer.js

import { setupUIListeners } from "./ui.js";
import { renderPlaylists } from "./playlistManager.js";
import { loadPlaylists } from "./playlists.js";
import { restoreLastTrack } from "./player.js";

window.addEventListener("DOMContentLoaded", async () => {
  try {
    console.log("Initializing application...");

    loadPlaylists();
    renderPlaylists();
    setupUIListeners();
    restoreLastTrack();

    console.log("Application initialized successfully.");
  } catch (error) {
    console.error("Error initializing application:", error);
  }
});
