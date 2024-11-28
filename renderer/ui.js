// renderer/ui.js

import { playlists, addPlaylist, deletePlaylist, getPlaylist, savePlaylists } from "./playlists.js";
import { selectFolderOrFiles } from "./library.js";

let currentPlaylist = null;

// Initialize UI Listeners
export function setupUIListeners() {
  // Modal controls
  const modal = document.getElementById("modal");
  const createButton = document.getElementById("create-playlist");
  const cancelButton = document.getElementById("cancel-playlist");

  // Button to open modal
  document.getElementById("new-playlist").addEventListener("click", () => {
    modal.style.display = "flex"; // Show modal
    console.log("Modal opened.");
  });

  // Button to create playlist
  createButton.addEventListener("click", handleCreatePlaylist);

  // Button to cancel and hide modal
  cancelButton.addEventListener("click", () => {
    closeModal();
    console.log("Modal closed via cancel.");
  });

  // Ensure modal is hidden on app load
  modal.style.display = "none";

  // Other controls
  document.getElementById("add-files").addEventListener("click", addFilesToPlaylist);
  renderPlaylists();
  console.log("UI listeners initialized.");
}

// Create a new playlist
function handleCreatePlaylist() {
  const nameInput = document.getElementById("playlist-name");
  const name = nameInput.value.trim();

  if (!name) {
    alert("Playlist name cannot be empty.");
    return;
  }

  if (!addPlaylist(name)) {
    alert("Playlist name already exists.");
    return;
  }

  renderPlaylists();
  closeModal();
  console.log(`Playlist "${name}" created.`);
}

// Close the modal and reset its state
function closeModal() {
  const modal = document.getElementById("modal");
  modal.style.display = "none"; // Explicitly hide the modal
  document.getElementById("playlist-name").value = ""; // Clear the input field
  console.log("Modal closed.");
}

// Render playlists
export function renderPlaylists() {
  const playlistPane = document.getElementById("playlists");
  playlistPane.innerHTML = "";

  Object.keys(playlists).forEach((name) => {
    const li = document.createElement("li");
    li.textContent = name;

    li.addEventListener("click", () => {
      currentPlaylist = name;
      renderPlaylistTracks();
    });

    const deleteBtn = document.createElement("button");
    deleteBtn.textContent = "X";
    deleteBtn.className = "delete-btn";
    deleteBtn.addEventListener("click", (e) => {
      e.stopPropagation();
      deletePlaylist(name);
      renderPlaylists();
      if (currentPlaylist === name) {
        currentPlaylist = null;
        renderPlaylistTracks();
      }
    });

    li.appendChild(deleteBtn);
    playlistPane.appendChild(li);
  });

  console.log("Playlists rendered.");
}

// Render tracks for the selected playlist
function renderPlaylistTracks() {
  const playlistDiv = document.getElementById("playlist");
  if (currentPlaylist) {
    const tracks = getPlaylist(currentPlaylist);
    playlistDiv.innerHTML = tracks.length
      ? tracks.map((track) => `<div>${track.name}</div>`).join("")
      : "No tracks in this playlist.";
  } else {
    playlistDiv.innerHTML = "No playlist selected.";
  }
}

// Add files to the selected playlist
async function addFilesToPlaylist() {
  if (!currentPlaylist) {
    alert("Please select a playlist first.");
    return;
  }

  const files = await selectFolderOrFiles();
  if (files) {
    playlists[currentPlaylist].push(...files);
    savePlaylists();
    renderPlaylistTracks();
    console.log(`Added files to playlist "${currentPlaylist}":`, files);
  }
}
