// renderer/ui.js

import { 
  handleCreatePlaylist, 
  renderPlaylists, 
  loadLastUsedPlaylist, 
  getCurrentPlaylist 
} from "./playlistManager.js"; // Added getCurrentPlaylist import
import { renderLibraryTree } from "./libraryRenderer.js";
import { setupDragAndDrop } from "./dragAndDrop.js";
import { renderPlaylistTracks } from "./trackManager.js"; // Ensure this is imported
import { savePlaylists, getPlaylist } from "./playlists.js"; // Ensure these are imported

export function setupUIListeners() {
  try {
    const modal = document.getElementById("modal");
    const createButton = document.getElementById("create-playlist");
    const cancelButton = document.getElementById("cancel-playlist");
    const addLibraryBtn = document.getElementById("add-library");
    const addToPlaylistBtn = document.getElementById("add-to-playlist");

    document.getElementById("new-playlist").addEventListener("click", openModal);
    createButton.addEventListener("click", () => {
      const nameInput = document.getElementById("playlist-name");
      if (nameInput) {
        handleCreatePlaylist(nameInput.value.trim());
        closeModal();
      }
    });
    cancelButton.addEventListener("click", closeModal);

    addLibraryBtn.addEventListener("click", async () => {
      console.log("'Add to Library' button clicked");
      try {
        await renderLibraryTree();
      } catch (error) {
        console.error("Error rendering library tree:", error);
      }
    });

    addToPlaylistBtn.addEventListener("click", async () => {
      try {
        const selectedFiles = await window.electron.selectFiles();
        if (selectedFiles && selectedFiles.length > 0) {
          const currentPlaylist = getCurrentPlaylist(); // Get the currently selected playlist
          if (!currentPlaylist) {
            alert("Please select or create a playlist first.");
            return;
          }

          const playlist = getPlaylist(currentPlaylist); // Fetch the current playlist
          selectedFiles.forEach((filePath) => {
            // Check if the track is already in the playlist
            if (!playlist.some((track) => track.path === filePath)) {
              playlist.push({ name: filePath.split("\\").pop(), path: filePath });
            }
          });

          savePlaylists(); // Save updated playlists to local storage
          renderPlaylistTracks(currentPlaylist); // Re-render the updated playlist
        }
      } catch (error) {
        console.error("Error adding files to playlist:", error);
      }
    });

    // Load the last used playlist
    loadLastUsedPlaylist();
    renderPlaylists();
    setupDragAndDrop(); // Initialize drag-and-drop
  } catch (error) {
    console.error("Error initializing UI listeners:", error);
  }
}

function openModal() {
  const modal = document.getElementById("modal");
  modal?.classList.add("modal-visible");
}

function closeModal() {
  const modal = document.getElementById("modal");
  modal?.classList.remove("modal-visible");
  const nameInput = document.getElementById("playlist-name");
  if (nameInput) nameInput.value = "";
}
