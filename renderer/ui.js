// renderer/ui.js

// renderer/ui.js

import { playlists, addPlaylist, deletePlaylist, getPlaylist, savePlaylists } from "./playlists.js";
import { renderLibraryTree } from "./libraryRenderer.js";
import { loadTrack, playTrack } from "./player.js";

let currentPlaylist = null;
let currentTrackIndex = 0;

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
      const selectedFiles = await window.electron.selectFiles(); // Use file selection dialog
      if (selectedFiles && selectedFiles.length > 0) {
        const playlist = getPlaylist(currentPlaylist);
        selectedFiles.forEach((filePath) => {
          if (!playlist.some((track) => track.path === filePath)) {
            playlist.push({ name: filePath.split("\\").pop(), path: filePath });
          }
        });
        savePlaylists(); // Save the updated playlists
        renderPlaylistTracks();
        console.log(`Files added to playlist "${currentPlaylist}":`, selectedFiles);
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
    if (nameInput) nameInput.value = ""; // Clear input field
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

  renderPlaylists();
  closeModal();
}

// Render playlists
export function renderPlaylists() {
  const playlistPane = document.getElementById("playlists");
  if (!playlistPane) return;

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
      trackElement.addEventListener("click", (e) => {
        currentTrackIndex = parseInt(trackElement.getAttribute("data-index"), 10);
        const trackPath = trackElement.getAttribute("data-path");

        window.electron.fileExists(trackPath).then((exists) => {
          if (!exists) {
            alertUser("File does not exist. Skipping...");
            skipToNextTrack();
          } else {
            console.log(`Playing track: ${trackPath}`);
            loadTrack(trackPath);
            playTrack();
          }
        });
      });
    });
  } else {
    playlistDiv.innerHTML = "No tracks in this playlist.";
  }
}

function skipToNextTrack() {
  const tracks = getPlaylist(currentPlaylist);
  if (!tracks.length) return;

  currentTrackIndex = (currentTrackIndex + 1) % tracks.length;
  const nextTrack = tracks[currentTrackIndex];

  window.electron.fileExists(nextTrack.path).then((exists) => {
    if (!exists) {
      skipToNextTrack(); // Recursively skip missing tracks
    } else {
      console.log(`Playing next track: ${nextTrack.path}`);
      loadTrack(nextTrack.path);
      playTrack();
    }
  });
}

function alertUser(message) {
  const alertDiv = document.createElement("div");
  alertDiv.className = "alert";
  alertDiv.textContent = message;
  document.body.appendChild(alertDiv);
  setTimeout(() => alertDiv.remove(), 2000);
}
