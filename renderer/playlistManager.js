// renderer/playlistManager.js

import { playlists, addPlaylist, deletePlaylist, getPlaylist, savePlaylists } from "./playlists.js";
import { renderPlaylistTracks } from "./trackManager.js";

let currentPlaylist = null;

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
      renderPlaylistTracks(currentPlaylist);
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
        renderPlaylistTracks(null);
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
