import { playlists, addPlaylist, deletePlaylist, getPlaylist, savePlaylists } from "./playlists.js";
import { renderLibraryTree } from "./libraryRenderer.js";

let currentPlaylist = null;

// Initialize UI Listeners
export function setupUIListeners() {
  const modal = document.getElementById("modal");
  const createButton = document.getElementById("create-playlist");
  const cancelButton = document.getElementById("cancel-playlist");

  document.getElementById("new-playlist").addEventListener("click", () => {
    openModal();
    console.log("Modal opened.");
  });

  createButton.addEventListener("click", handleCreatePlaylist);

  cancelButton.addEventListener("click", () => {
    closeModal();
    console.log("Modal closed via cancel.");
  });

  modal.classList.add("modal-hidden"); // Ensure modal starts hidden

  // Updated: Attach library tree rendering to the "+" button
  document.getElementById("add-files").addEventListener("click", async () => {
    console.log("Add files button clicked.");
    try {
      await renderLibraryTree();
    } catch (error) {
      console.error("Error rendering library tree:", error);
    }
  });

  renderPlaylists();
  console.log("UI listeners initialized.");
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
      console.log(`Playlist selected: ${name}`);
      currentPlaylist = name;
      renderPlaylistTracks();
    });

    const deleteBtn = document.createElement("button");
    deleteBtn.textContent = "X";
    deleteBtn.className = "delete-btn";
    deleteBtn.addEventListener("click", (e) => {
      e.stopPropagation();
      deletePlaylist(name);
      console.log(`Playlist deleted: ${name}`);
      renderPlaylists();
      if (currentPlaylist === name) {
        currentPlaylist = null;
        renderPlaylistTracks();
      }
    });

    li.appendChild(deleteBtn);
    playlistPane.appendChild(li);
  });

  console.log("Playlists rendered:", playlists);
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
    console.log(`Tracks rendered for playlist: ${currentPlaylist}`);
  } else {
    playlistDiv.innerHTML = "No tracks in this playlist.";
    console.warn(`No tracks found in playlist: ${currentPlaylist}`);
  }
}
