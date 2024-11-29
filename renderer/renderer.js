// renderer/renderer.js

import { loadPlaylists } from "./playlists.js";
import { setupUIListeners, renderPlaylists } from "./ui.js";
import { renderLibraryTree } from "./libraryRenderer.js";

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
  } catch (error) {
    console.error("Error during initialization:", error);
  }
});

