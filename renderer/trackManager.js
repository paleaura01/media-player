// renderer/trackManager.js

import { getPlaylist, updatePlaylist } from "./playlists.js";
import { getCurrentTrackPath, getIsPlaying, loadTrack, setCurrentPlaylist, setCurrentTrackIndex } from "./player.js";

export function renderPlaylistTracks(playlistName) {
  const playlistDiv = document.getElementById("playlist");
  if (!playlistDiv) {
    console.error("Playlist display area not found!");
    return;
  }

  const playlistHeader = document.getElementById("playlist-header");
  if (!playlistHeader) {
    console.error("Playlist header not found!");
    return;
  }

  if (!playlistName) {
    playlistDiv.innerHTML = "No playlist selected.";
    playlistHeader.innerHTML = "<h2>Playlist</h2>";
    return;
  }

  const tracks = getPlaylist(playlistName);
  playlistDiv.innerHTML = "";

  const numTracks = tracks.length;
  playlistHeader.innerHTML = `<h2>${playlistName} (${numTracks} tracks)</h2>`;

  tracks.forEach((track, index) => {
    const trackElement = document.createElement("div");
    trackElement.className = "track";
    trackElement.dataset.path = track.path;

    trackElement.addEventListener("click", () => {
      const currentPath = getCurrentTrackPath();
      const isPlaying = getIsPlaying();

      if (currentPath === track.path && isPlaying) {
        console.log(`Track already playing: ${track.path}`);
        return;
      }

      console.log(`Playing track: ${track.path}`);
      setCurrentPlaylist(playlistName);
      setCurrentTrackIndex(index);
      loadTrack(track.path);
    });

    const titleContainer = document.createElement("div");
    titleContainer.className = "track-title";

    const trackName = document.createElement("span");
    trackName.textContent = track.name;
    titleContainer.appendChild(trackName);

    const playCount = document.createElement("span");
    playCount.className = "play-count";
    playCount.textContent = ` (Played ${track.playCount || 0} times)`;
    titleContainer.appendChild(playCount);

    const deleteContainer = document.createElement("div");
    deleteContainer.className = "delete-container";

    const deleteButton = document.createElement("button");
    deleteButton.className = "delete-track";
    deleteButton.textContent = "X";

    deleteButton.addEventListener("click", (e) => {
      e.stopPropagation();
      tracks.splice(index, 1);
      updatePlaylist(playlistName, tracks);
      renderPlaylistTracks(playlistName);
    });

    deleteContainer.appendChild(deleteButton);
    trackElement.appendChild(titleContainer);
    trackElement.appendChild(deleteContainer);
    playlistDiv.appendChild(trackElement);
  });

  console.log(`Tracks rendered for playlist "${playlistName}":`, tracks);
}
