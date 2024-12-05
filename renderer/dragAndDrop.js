// renderer/dragAndDrop.js

import { getCurrentPlaylist } from "./playlistManager.js";
import { getPlaylist, savePlaylists } from "./playlists.js";
import { renderPlaylistTracks } from "./trackManager.js";

// Utility: Check if a file is an audio file
function isAudioFile(fileName) {
  const audioExtensions = [".mp3", ".wav", ".ogg", ".opus", ".flac", ".aac", ".mp4"];
  return audioExtensions.includes(fileName.slice(fileName.lastIndexOf(".")).toLowerCase());
}

export function setupDragAndDrop() {
  const playlistDiv = document.getElementById("playlist");

  if (!playlistDiv) {
    console.error("Drag-and-drop setup failed: missing playlist element.");
    return;
  }

  // Enable dropping into the playlist
  playlistDiv.addEventListener("dragover", (event) => {
    event.preventDefault();
  });

  playlistDiv.addEventListener("drop", async (event) => {
    event.preventDefault();
    // Get the file paths of the dropped files
    const files = event.dataTransfer.files;
    console.log(`Dropped files:`, files);

    if (!files || files.length === 0) return;

    const currentPlaylist = getCurrentPlaylist();
    if (!currentPlaylist) {
      alert("Please select a playlist first.");
      return;
    }

    const playlist = getPlaylist(currentPlaylist);

    for (let i = 0; i < files.length; i++) {
      const file = files[i];
      // Only accept audio files
      if (isAudioFile(file.name)) {
        const filePath = file.path;
        if (!playlist.some((track) => track.path === filePath)) {
          playlist.push({ name: file.name, path: filePath });
          console.log(`Added "${filePath}" to playlist "${currentPlaylist}".`);
        }
      }
    }

    savePlaylists(); // Save playlists to ensure persistence
    renderPlaylistTracks(currentPlaylist); // Re-render playlist tracks
  });
}
