// renderer/ui.js

import {
  handleCreatePlaylist,
  renderPlaylists,
  loadLastUsedPlaylist,
  getCurrentPlaylist,
} from "./playlistManager.js";
import { setupDragAndDrop } from "./dragAndDrop.js";
import { renderPlaylistTracks } from "./trackManager.js";
import { savePlaylists, getPlaylist, addTrackToPlaylist } from "./playlists.js";
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
    document.getElementById("play").addEventListener("click", playTrack);
    document.getElementById("next").addEventListener("click", nextTrack);
    document.getElementById("prev").addEventListener("click", prevTrack);

    document.getElementById("shuffle").addEventListener("click", (e) => {
      const shuffleOn = toggleShuffle();
      e.target.textContent = shuffleOn ? "Shuffle On" : "Shuffle Off";
    });

    document.getElementById("repeat").addEventListener("click", (e) => {
      const repeatOn = toggleRepeat();
      e.target.textContent = repeatOn ? "Repeat On" : "Repeat Off";
    });

    // Modal and playlist actions
    const modal = document.getElementById("modal");
    const createButton = document.getElementById("create-playlist");
    const cancelButton = document.getElementById("cancel-playlist");
    const addToPlaylistBtn = document.getElementById("add-to-playlist");

    document.getElementById("new-playlist").addEventListener("click", openModal);
    cancelButton.addEventListener("click", closeModal);
    createButton.addEventListener("click", createNewPlaylist);

    addToPlaylistBtn.addEventListener("click", addFilesToPlaylist);

    // Initialize playlists and drag-and-drop functionality
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

function createNewPlaylist() {
  const nameInput = document.getElementById("playlist-name");
  const playlistName = nameInput?.value.trim();
  if (playlistName) {
    handleCreatePlaylist(playlistName);
    closeModal();
  } else {
    alert("Playlist name cannot be empty!");
  }
}

async function addFilesToPlaylist() {
  try {
    console.log("'Add to Playlist' button clicked.");

    // Open file selection dialog
    const selectedFiles = await window.electron.selectFiles();

    if (!selectedFiles || selectedFiles.length === 0) {
      console.warn("No files selected.");
      return;
    }

    const currentPlaylist = getCurrentPlaylist();
    if (!currentPlaylist) {
      alert("Please select or create a playlist first.");
      return;
    }

    const playlist = getPlaylist(currentPlaylist);

    // Add files to the playlist
    selectedFiles.forEach((filePath) => {
      const track = { name: filePath.split("\\").pop(), path: filePath };
      if (addTrackToPlaylist(currentPlaylist, track)) {
        console.log(`Added track to playlist: ${track.name}`);
      } else {
        console.warn(`Track already exists in the playlist: ${track.name}`);
      }
    });

    savePlaylists();
    renderPlaylistTracks(currentPlaylist);
    console.log("Tracks added to playlist successfully.");
  } catch (error) {
    console.error("Error adding files to playlist:", error);
  }
}
