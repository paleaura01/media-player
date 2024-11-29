// renderer/playlistManager.js

import { renderPlaylistTracks as renderTracks } from "./trackManager.js";
import { playlists, addPlaylist, deletePlaylist, getPlaylist, savePlaylists } from "./playlists.js";

let currentPlaylist = null;



export function loadLastUsedPlaylist() {
  const lastUsedPlaylist = localStorage.getItem("lastUsedPlaylist");
  if (lastUsedPlaylist && playlists[lastUsedPlaylist]) {
    currentPlaylist = lastUsedPlaylist;
    console.log(`Loaded last used playlist: "${currentPlaylist}"`);
    renderTracks(currentPlaylist); // Render using the proper function
  } else {
    console.log("No last used playlist found or it does not exist.");
    currentPlaylist = null;
  }
}

export function saveCurrentPlaylist(name) {
  localStorage.setItem("lastUsedPlaylist", name);
}

export function handleCreatePlaylist(name) {
  if (!name.trim()) {
    alert("Playlist name cannot be empty.");
    return false;
  }

  if (!addPlaylist(name)) {
    alert("Playlist name already exists.");
    return false;
  }

  savePlaylists();
  currentPlaylist = name;
  saveCurrentPlaylist(name);
  renderPlaylists();
  return true;
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
      saveCurrentPlaylist(name);
      renderTracks(currentPlaylist); // Render using the proper function
    });

    const deleteBtn = document.createElement("button");
    deleteBtn.textContent = "X";
    deleteBtn.className = "delete-btn";
    deleteBtn.addEventListener("click", (e) => {
      e.stopPropagation();
      deletePlaylist(name);
      savePlaylists();
      renderPlaylists();
      if (currentPlaylist === name) {
        currentPlaylist = null;
        localStorage.removeItem("lastUsedPlaylist");
        renderTracks(null);
      }
    });

    li.appendChild(deleteBtn);
    playlistPane.appendChild(li);
  });

  console.log("Playlists rendered:", Object.keys(playlists));
}

export function getCurrentPlaylist() {
  return currentPlaylist;
}
