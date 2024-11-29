// renderer/ui.js

import { handleCreatePlaylist } from "./playlistManager.js";

export function setupUIListeners() {
  try {
    const modal = document.getElementById("modal");
    const createButton = document.getElementById("create-playlist");
    const cancelButton = document.getElementById("cancel-playlist");

    // Open the modal when clicking the New Playlist button
    document.getElementById("new-playlist").addEventListener("click", openModal);

    // Close the modal when clicking the Cancel button
    cancelButton.addEventListener("click", closeModal);

    // Create a playlist when clicking the Create button
    createButton.addEventListener("click", () => {
      const nameInput = document.getElementById("playlist-name");
      if (nameInput) {
        handleCreatePlaylist(nameInput.value.trim());
        closeModal();
      }
    });
  } catch (error) {
    console.error("Error initializing UI listeners:", error);
  }
}

function openModal() {
  const modal = document.getElementById("modal");
  if (modal) {
    modal.classList.remove("modal-hidden");
    modal.classList.add("modal-visible");
  } else {
    console.error("Modal element not found!");
  }
}

function closeModal() {
  const modal = document.getElementById("modal");
  if (modal) {
    modal.classList.remove("modal-visible");
    modal.classList.add("modal-hidden");
    const nameInput = document.getElementById("playlist-name");
    if (nameInput) nameInput.value = ""; // Clear the input
  } else {
    console.error("Modal element not found!");
  }
}

