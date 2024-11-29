// renderer/dragAndDrop.js

import { getCurrentPlaylist } from "./playlistManager.js";
import { getPlaylist, savePlaylists } from "./playlists.js";
import { renderPlaylistTracks } from "./trackManager.js";

export function setupDragAndDrop() {
  const libraryTree = document.getElementById("library-tree");
  const playlistDiv = document.getElementById("playlist");

  if (!libraryTree || !playlistDiv) {
    console.error("Drag-and-drop setup failed: missing library or playlist elements.");
    return;
  }

  // Enable dragging from the library
  libraryTree.addEventListener("dragstart", (event) => {
    const target = event.target;
    if (target && target.classList.contains("file-node")) {
      event.dataTransfer.setData("text/plain", target.getAttribute("data-path"));
      console.log(`Dragging file: ${target.getAttribute("data-path")}`);
    }
  });

  // Enable dropping into the playlist
  playlistDiv.addEventListener("dragover", (event) => {
    event.preventDefault();
  });

  playlistDiv.addEventListener("drop", (event) => {
    event.preventDefault();
    const filePath = event.dataTransfer.getData("text/plain");
    console.log(`Dropped file: ${filePath}`);

    if (!filePath) return;

    const currentPlaylist = getCurrentPlaylist();
    if (!currentPlaylist) {
      alert("Please select a playlist first.");
      return;
    }

    const playlist = getPlaylist(currentPlaylist);
    if (!playlist.some((track) => track.path === filePath)) {
      playlist.push({ name: filePath.split("\\").pop(), path: filePath });
      console.log(`Added "${filePath}" to playlist "${currentPlaylist}".`);
      savePlaylists(); // Save playlists to ensure persistence
      renderPlaylistTracks(currentPlaylist); // Re-render playlist tracks
    }
  });
}

