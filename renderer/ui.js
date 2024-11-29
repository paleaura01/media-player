// renderer/ui.js

import { playlists, addPlaylist, deletePlaylist, getPlaylist } from "./playlists.js";
import { renderLibraryTree } from "./libraryRenderer.js";

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
      console.log("'Add to Playlist' button clicked");
      if (!currentPlaylist) {
        alert("Please select a playlist first!");
        return;
      }

      try {
        // Open file selection dialog
        const selectedFiles = await window.electron.selectFiles(); // Use a dedicated file selection dialog
        if (selectedFiles && selectedFiles.length > 0) {
          const playlist = getPlaylist(currentPlaylist);
          selectedFiles.forEach((filePath) => {
            if (!playlist.some((track) => track.path === filePath)) {
              playlist.push({ name: filePath.split("\\").pop(), path: filePath });
            }
          });
          console.log(`Files added to playlist "${currentPlaylist}":`, selectedFiles);
          renderPlaylistTracks();
        } else {
          console.log("No files selected.");
        }
      } catch (error) {
        console.error("Error adding files to playlist:", error);
      }
    });

    renderPlaylists();
  } catch (error) {
    console.error("Error initializing UI listeners:", error);
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
    console.error("Failed to create playlist: name is empty.");
    return;
  }

  if (!addPlaylist(name)) {
    alert("Playlist name already exists.");
    console.error("Failed to create playlist: name already exists.");
    return;
  }

  renderPlaylists();
  closeModal(); // Close modal after creating playlist
  console.log(`Playlist "${name}" created successfully.`);
}

// Render playlists
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
        renderPlaylistTracks();
      }
    });

    li.appendChild(deleteBtn);
    playlistPane.appendChild(li);
  });

  console.log("Playlists rendered:", Object.keys(playlists));
}

// Render tracks for the selected playlist
function renderPlaylistTracks() {
  const playlistDiv = document.getElementById("playlist");
  if (!playlistDiv) {
    console.error("Playlist display area not found!");
    return;
  }

  if (!currentPlaylist) {
    playlistDiv.innerHTML = "No playlist selected.";
    console.warn("No playlist selected.");
    return;
  }

  const tracks = getPlaylist(currentPlaylist);

  if (tracks.length > 0) {
    playlistDiv.innerHTML = tracks
      .map((track) => `<div class="track">${track.name}</div>`)
      .join("");
    console.log(`Tracks rendered for playlist "${currentPlaylist}":`, tracks);
  } else {
    playlistDiv.innerHTML = "No tracks in this playlist.";
    console.warn(`No tracks found in playlist "${currentPlaylist}".`);
  }
}

