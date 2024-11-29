// renderer/ui.js

import { playlists, addPlaylist, deletePlaylist, getPlaylist, savePlaylists } from "./playlists.js";
import { renderLibraryTree } from "./libraryRenderer.js";
import { loadTrack, playTrack } from "./player.js";
import { setupDragAndDrop } from "./dragAndDrop.js";

let currentPlaylist = null;

export function setupUIListeners() {
  try {
    const modal = document.getElementById("modal");
    const createButton = document.getElementById("create-playlist");
    const cancelButton = document.getElementById("cancel-playlist");
    const addLibraryBtn = document.getElementById("add-library");
    const addToPlaylistBtn = document.getElementById("add-to-playlist");

    // Listener for "New Playlist"
    document.getElementById("new-playlist").addEventListener("click", openModal);
    createButton.addEventListener("click", handleCreatePlaylist);
    cancelButton.addEventListener("click", closeModal);

    // Listener for adding to library
    addLibraryBtn.addEventListener("click", async () => {
      console.log("'Add to Library' button clicked");
      try {
        await renderLibraryTree();
      } catch (error) {
        console.error("Error rendering library tree:", error);
      }
    });

    // Listener for adding tracks to playlist
    addToPlaylistBtn.addEventListener("click", async () => {
      if (!currentPlaylist) {
        alert("Please select a playlist first!");
        return;
      }

      const selectedFiles = await window.electron.selectFiles();
      if (selectedFiles && selectedFiles.length > 0) {
        const playlist = getPlaylist(currentPlaylist);
        selectedFiles.forEach((filePath) => {
          if (!playlist.some((track) => track.path === filePath)) {
            playlist.push({ name: filePath.split("\\").pop(), path: filePath });
          }
        });
        savePlaylists();
        renderPlaylistTracks();
      }
    });

    // Initialize UI with the most recent playlist if available
    const lastUsedPlaylist = localStorage.getItem("lastUsedPlaylist");
    if (lastUsedPlaylist && playlists[lastUsedPlaylist]) {
      currentPlaylist = lastUsedPlaylist;
      console.log(`Loaded most recent playlist: ${currentPlaylist}`);
      renderPlaylistTracks();
    }

    renderPlaylists();
    setupDragAndDrop(); // Initialize drag-and-drop
  } catch (error) {
    console.error("Error initializing UI listeners:", error);
  }
}

export function renderPlaylists() {
  const playlistPane = document.getElementById("playlists");
  if (!playlistPane) {
    console.error("Playlist pane not found!");
    return;
  }

  playlistPane.innerHTML = "";

  Object.keys(playlists).forEach((name) => {
    const li = document.createElement("li");
    li.textContent = name;

    li.addEventListener("click", () => {
      console.log(`Playlist "${name}" selected.`);
      currentPlaylist = name;
      localStorage.setItem("lastUsedPlaylist", name); // Save the selected playlist
      renderPlaylistTracks();
    });

    const deleteBtn = document.createElement("button");
    deleteBtn.textContent = "X";
    deleteBtn.className = "delete-btn";
    deleteBtn.addEventListener("click", (e) => {
      e.stopPropagation();
      deletePlaylist(name);
      console.log(`Playlist "${name}" deleted.`);
      renderPlaylists();
      if (currentPlaylist === name) {
        currentPlaylist = null;
        localStorage.removeItem("lastUsedPlaylist");
        renderPlaylistTracks();
      }
    });

    li.appendChild(deleteBtn);
    playlistPane.appendChild(li);
  });

  console.log("Playlists rendered:", Object.keys(playlists));
}

export function renderPlaylistTracks() {
  const playlistDiv = document.getElementById("playlist");
  if (!playlistDiv) {
    console.error("Playlist display area not found!");
    return;
  }

  if (!currentPlaylist) {
    playlistDiv.innerHTML = "No playlist selected.";
    return;
  }

  const tracks = getPlaylist(currentPlaylist);

  if (tracks.length > 0) {
    playlistDiv.innerHTML = tracks
      .map(
        (track, index) =>
          `<div class="track" data-index="${index}" data-path="${track.path}">${track.name}</div>`
      )
      .join("");

    playlistDiv.querySelectorAll(".track").forEach((trackElement) => {
      trackElement.addEventListener("click", () => {
        const trackPath = trackElement.getAttribute("data-path");
        console.log(`Playing track: ${trackPath}`);
        loadTrack(trackPath);
        playTrack();
      });
    });
  } else {
    playlistDiv.innerHTML = "No tracks in this playlist.";
  }
}

// Open the modal
function openModal() {
  const modal = document.getElementById("modal");
  if (modal) {
    modal.classList.remove("modal-hidden");
    modal.classList.add("modal-visible");
    console.log("Modal set to visible.");
  } else {
    console.error("Modal element not found!");
  }
}

// Close the modal
function closeModal() {
  const modal = document.getElementById("modal");
  if (modal) {
    modal.classList.remove("modal-visible");
    modal.classList.add("modal-hidden");
    const nameInput = document.getElementById("playlist-name");
    if (nameInput) {
      nameInput.value = ""; // Clear input field
    }
    console.log("Modal hidden and input field cleared.");
  } else {
    console.error("Modal element not found!");
  }
}

// Create a new playlist
function handleCreatePlaylist() {
  const nameInput = document.getElementById("playlist-name");
  if (!nameInput) {
    console.error("Playlist name input field not found!");
    return;
  }

  const name = nameInput.value.trim();
  if (!name) {
    alert("Playlist name cannot be empty.");
    return;
  }

  if (!addPlaylist(name)) {
    alert("Playlist name already exists.");
    return;
  }

  currentPlaylist = name;
  localStorage.setItem("lastUsedPlaylist", name); // Save the new playlist
  renderPlaylists();
  renderPlaylistTracks();
  closeModal();
}
