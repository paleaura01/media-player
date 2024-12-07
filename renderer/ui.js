// renderer/ui.js

import {
  handleCreatePlaylist,
  renderPlaylists,
  loadLastUsedPlaylist,
  getCurrentPlaylist,
} from "./playlistManager.js";
import { setupDragAndDrop } from "./dragAndDrop.js";
import { renderPlaylistTracks } from "./trackManager.js";
import { savePlaylists, getPlaylist } from "./playlists.js";
import {
  playTrack,
  nextTrack,
  prevTrack,
  toggleShuffle,
  toggleRepeat,
} from "./player.js";

export function setupUIListeners() {
  try {
    // Event listeners for player controls
    document.getElementById("play").addEventListener("click", () => {
      playTrack();
    });

    document.getElementById("next").addEventListener("click", () => {
      nextTrack();
    });

    document.getElementById("prev").addEventListener("click", () => {
      prevTrack();
    });

    document.getElementById("shuffle").addEventListener("click", (e) => {
      const shuffleOn = toggleShuffle();
      e.target.textContent = shuffleOn ? "Shuffle On" : "Shuffle Off";
    });

    document.getElementById("repeat").addEventListener("click", (e) => {
      const repeatOn = toggleRepeat();
      e.target.textContent = repeatOn ? "Repeat On" : "Repeat Off";
    });

    const modal = document.getElementById("modal");
    const createButton = document.getElementById("create-playlist");
    const cancelButton = document.getElementById("cancel-playlist");
    const addToPlaylistBtn = document.getElementById("add-to-playlist");

    document.getElementById("new-playlist").addEventListener("click", () => {
      openModal();
    });

    cancelButton.addEventListener("click", () => {
      closeModal();
    });

    createButton.addEventListener("click", () => {
      const nameInput = document.getElementById("playlist-name");
      const playlistName = nameInput?.value.trim();
      if (playlistName) {
        handleCreatePlaylist(playlistName);
        closeModal();
      } else {
        alert("Playlist name cannot be empty!");
      }
    });

    addToPlaylistBtn.addEventListener("click", async () => {
      try {
        const selectedFiles = await window.electron.selectFiles();
        if (selectedFiles?.length > 0) {
          const currentPlaylist = getCurrentPlaylist();
          if (!currentPlaylist) {
            alert("Please select or create a playlist first.");
            return;
          }

          const playlist = getPlaylist(currentPlaylist);
          selectedFiles.forEach((filePath) => {
            if (!playlist.some((track) => track.path === filePath)) {
              playlist.push({ name: filePath.split("\\").pop(), path: filePath });
            }
          });

          savePlaylists();
          renderPlaylistTracks(currentPlaylist);
        }
      } catch (error) {
        console.error("Error adding files to playlist:", error);
      }
    });

    loadLastUsedPlaylist();
    renderPlaylists();
    setupDragAndDrop();
  } catch (error) {
    console.error("Error initializing UI listeners:", error);
  }
}

function openModal() {
  const modal = document.getElementById("modal");
  modal?.classList.replace("modal-hidden", "modal-visible");
}

function closeModal() {
  const modal = document.getElementById("modal");
  modal?.classList.replace("modal-visible", "modal-hidden");
}