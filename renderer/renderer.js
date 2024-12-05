// renderer/renderer.js

import { setupUIListeners } from "./ui.js";
import { renderPlaylists } from "./playlistManager.js";
import { renderLibraryTree } from "./libraryRenderer.js";
import { loadPlaylists } from "./playlists.js";
import { restoreLastTrack } from "./player.js"; // Import the function

window.addEventListener("DOMContentLoaded", async () => {
  console.log("DOM fully loaded and parsed. Running initialization...");
  try {
    // Load saved playlists from local storage
    loadPlaylists();
    console.log("Playlists loaded.");

    // Render playlists in the UI
    renderPlaylists();
    console.log("Playlists rendered.");

    // Initialize UI listeners
    setupUIListeners();
    console.log("UI listeners initialized.");

    // Optionally render library tree if a folder was previously selected
    const lastLibraryPath = localStorage.getItem("lastLibraryPath");
    if (lastLibraryPath) {
      console.log("Rendering library tree from last selected path:", lastLibraryPath);
      await renderLibraryTree(lastLibraryPath);
    } else {
      console.log("No previous library path found.");
    }

    // Restore the last playing track
    restoreLastTrack();

  } catch (error) {
    console.error("Error during initialization:", error);
  }
});
