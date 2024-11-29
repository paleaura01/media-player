// renderer/dragAndDrop.js

import { getPlaylist, savePlaylists } from "./playlists.js";
import { renderPlaylistTracks } from "./ui.js";

export function setupDragAndDrop(getCurrentPlaylist) {
  const libraryTree = document.getElementById("library-tree");
  const playlistDiv = document.getElementById("playlist");

  if (!libraryTree || !playlistDiv) {
    console.error("Library tree or playlist div not found!");
    return;
  }

  libraryTree.addEventListener("dragstart", (event) => {
    const target = event.target;
    if (target.classList.contains("file-node")) {
      event.dataTransfer.setData(
        "text/plain",
        JSON.stringify({
          name: target.textContent,
          path: target.getAttribute("data-path"),
        })
      );
    }
  });

  playlistDiv.addEventListener("dragover", (event) => {
    event.preventDefault();
    playlistDiv.classList.add("drag-over");
  });

  playlistDiv.addEventListener("dragleave", () => {
    playlistDiv.classList.remove("drag-over");
  });

  playlistDiv.addEventListener("drop", (event) => {
    event.preventDefault();
    playlistDiv.classList.remove("drag-over");

    const data = event.dataTransfer.getData("text/plain");
    if (!data) return;

    const file = JSON.parse(data);
    const currentPlaylist = getCurrentPlaylist();
    if (!currentPlaylist) {
      alert("Please select a playlist first!");
      return;
    }

    const playlist = getPlaylist(currentPlaylist);
    if (!playlist.some((track) => track.path === file.path)) {
      playlist.push(file);
      savePlaylists();
      renderPlaylistTracks(currentPlaylist);
    }
  });
}
