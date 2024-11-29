// renderer/ui.js

import { playlists, addPlaylist, deletePlaylist, getPlaylist } from "./playlists.js";
import { renderLibraryTree } from "./libraryRenderer.js";

let currentPlaylist = null;

// Initialize UI Listeners
export function setupUIListeners() {
  try {
    const modal = document.getElementById("modal");
    const createButton = document.getElementById("create-playlist");
    const cancelButton = document.getElementById("cancel-playlist");

    // Attach listener for "New Playlist"
    const newPlaylistBtn = document.getElementById("new-playlist");
    if (newPlaylistBtn) {
      newPlaylistBtn.addEventListener("click", () => {
        console.log("'New Playlist' button clicked");
        openModal();
      });
      console.log("'New Playlist' button listener attached");
    } else {
      console.error("Could not find 'New Playlist' button");
    }

    // Attach listener for "Create Playlist"
    if (createButton) {
      createButton.addEventListener("click", handleCreatePlaylist);
      console.log("'Create Playlist' button listener attached");
    } else {
      console.error("Could not find 'Create Playlist' button");
    }

    // Attach listener for "Cancel Playlist"
    if (cancelButton) {
      cancelButton.addEventListener("click", () => {
        console.log("'Cancel Playlist' button clicked");
        closeModal();
      });
      console.log("'Cancel Playlist' button listener attached");
    } else {
      console.error("Could not find 'Cancel Playlist' button");
    }

    // Attach listener for "Add Files"
    const addFilesBtn = document.getElementById("add-files");
    if (addFilesBtn) {
      addFilesBtn.addEventListener("click", async () => {
        console.log("'Add Files' button clicked");
        try {
          await renderLibraryTree();
        } catch (error) {
          console.error("Error rendering library tree:", error);
        }
      });
      console.log("'Add Files' button listener attached");
    } else {
      console.error("Could not find 'Add Files' button");
    }

    renderPlaylists();
    console.log("UI listeners initialized successfully");
  } catch (error) {
    console.error("Error initializing UI listeners:", error);
  }
}


// Open the modal using class-based approach
function openModal() {
  const modal = document.getElementById("modal");
  modal.classList.remove("modal-hidden");
  modal.classList.add("modal-visible");
  console.log("Modal set to visible.");
}

// Close the modal and reset its state
function closeModal() {
  const modal = document.getElementById("modal");
  modal.classList.remove("modal-visible");
  modal.classList.add("modal-hidden");
  document.getElementById("playlist-name").value = ""; // Clear input field
  console.log("Modal hidden and input field cleared.");
}

// Create a new playlist
function handleCreatePlaylist() {
  const nameInput = document.getElementById("playlist-name");
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
  closeModal();
  console.log(`Playlist "${name}" created successfully.`);
}

// Render playlists
export function renderPlaylists() {
  const playlistPane = document.getElementById("playlists");
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
