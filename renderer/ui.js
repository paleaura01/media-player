// renderer/ui.js

import {
  handleCreatePlaylist,
  renderPlaylists,
  loadLastUsedPlaylist,
  getCurrentPlaylist,
} from "./playlistManager.js";
import { renderLibraryTree } from "./libraryRenderer.js";
import { setupDragAndDrop } from "./dragAndDrop.js";
import { renderPlaylistTracks } from "./trackManager.js";
import { savePlaylists, getPlaylist } from "./playlists.js";

// Import playback functions
import {
  playTrack,
  pauseTrack,
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

    document.getElementById("pause").addEventListener("click", () => {
      pauseTrack();
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

    // Reference DOM elements
    const modal = document.getElementById("modal");
    const createButton = document.getElementById("create-playlist");
    const cancelButton = document.getElementById("cancel-playlist");
    const addLibraryBtn = document.getElementById("add-library");
    const addToPlaylistBtn = document.getElementById("add-to-playlist");
    const closeLibraryBtn = document.getElementById("close-library");

    // Open modal when clicking the "New Playlist" button
    document.getElementById("new-playlist").addEventListener("click", () => {
      console.log("'New Playlist' button clicked.");
      openModal();
    });

    // Close modal when clicking the Cancel button
    cancelButton.addEventListener("click", () => {
      console.log("'Cancel' button clicked.");
      closeModal();
    });

    // Create a new playlist when clicking the Create button
    createButton.addEventListener("click", () => {
      console.log("'Create Playlist' button clicked.");
      const nameInput = document.getElementById("playlist-name");
      if (nameInput) {
        const playlistName = nameInput.value.trim();
        if (playlistName) {
          handleCreatePlaylist(playlistName);
          closeModal();
        } else {
          alert("Playlist name cannot be empty!");
        }
      }
    });

    // Handle Library Button functionality
    addLibraryBtn.addEventListener("click", async () => {
      console.log("'Add to Library' button clicked.");
      try {
        await renderLibraryTree();
        console.log("Library tree rendered successfully.");

        // Show the library tree
        const libraryTree = document.getElementById("library-tree");
        libraryTree.classList.remove("hidden");

        // Hide the '+' button when the library tree is visible
        addLibraryBtn.style.display = "none";

        // Adjust the library container
        const libraryContainer = document.getElementById("library-container");
        libraryContainer.style.flex = "1";
      } catch (error) {
        console.error("Error rendering library tree:", error);
      }
    });

    // Handle Close Library Button functionality
    closeLibraryBtn.addEventListener("click", () => {
      console.log("'Close Library' button clicked.");
      const libraryTree = document.getElementById("library-tree");
      libraryTree.classList.add("hidden");

      // Show the '+' button when the library tree is hidden
      addLibraryBtn.style.display = "block";

      // Adjust the library container height
      const libraryContainer = document.getElementById("library-container");
      libraryContainer.style.flex = "0 0 auto";
    });

    // Handle Add to Playlist Button functionality
    addToPlaylistBtn.addEventListener("click", async () => {
      console.log("'Add to Playlist' button clicked.");
      try {
        const selectedFiles = await window.electron.selectFiles();
        if (selectedFiles && selectedFiles.length > 0) {
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
          console.log("Tracks added to playlist successfully.");
        }
      } catch (error) {
        console.error("Error adding files to playlist:", error);
      }
    });

    // Initialize last used playlist and drag-and-drop functionality
    loadLastUsedPlaylist();
    renderPlaylists();
    setupDragAndDrop();

    // Initialize splitters
    setupSplitters();
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
    console.log("Modal opened.");
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
    if (nameInput) nameInput.value = ""; // Clear input
    console.log("Modal closed.");
  } else {
    console.error("Modal element not found!");
  }
}

// Function to initialize splitters
function setupSplitters() {
  // Vertical Splitter between Playlist Pane and Content Area
  const verticalSplitter = document.getElementById("vertical-splitter");
  const playlistPane = document.getElementById("playlist-pane");
  const contentArea = document.getElementById("content-area");

  let isResizingVertical = false;

  verticalSplitter.addEventListener("mousedown", () => {
    isResizingVertical = true;
  });

  document.addEventListener("mousemove", (e) => {
    if (isResizingVertical) {
      const newWidth = e.clientX - playlistPane.offsetLeft;
      playlistPane.style.width = `${newWidth}px`;
    }
  });

  document.addEventListener("mouseup", () => {
    isResizingVertical = false;
  });

  // Horizontal Splitter between Library and Playlist Containers
  const horizontalSplitter = document.getElementById("horizontal-splitter");
  const libraryContainer = document.getElementById("library-container");
  const playlistContainer = document.getElementById("playlist-container");

  let isResizingHorizontal = false;

  horizontalSplitter.addEventListener("mousedown", () => {
    isResizingHorizontal = true;
  });

  document.addEventListener("mousemove", (e) => {
    if (isResizingHorizontal) {
      const containerOffsetTop = libraryContainer.parentElement.offsetTop;
      const newHeight = e.clientY - containerOffsetTop;
      libraryContainer.style.height = `${newHeight}px`;
    }
  });

  document.addEventListener("mouseup", () => {
    isResizingHorizontal = false;
  });
}
