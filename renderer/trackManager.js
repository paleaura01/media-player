// renderer/trackManager.js

import { getPlaylist, updatePlaylist } from "./playlists.js";
import { loadTrack, playTrack, setCurrentPlaylist, setCurrentTrackIndex } from "./player.js";

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
  playlistDiv.innerHTML = ""; // Clear the playlist container

  // Update the header to include the playlist name and number of tracks
  const numTracks = tracks.length;
  playlistHeader.innerHTML = `<h2>${playlistName} (${numTracks} tracks)</h2>`;

  tracks.forEach((track, index) => {
    const trackElement = document.createElement("div");
    trackElement.className = "track";
    trackElement.dataset.path = track.path; // Store the track's path

    // Title Container
    const titleContainer = document.createElement("div");
    titleContainer.className = "track-title";

    // Track Name
    const trackName = document.createElement("span");
    trackName.textContent = track.name;
    titleContainer.appendChild(trackName);

    // Play Count
    const playCount = document.createElement("span");
    playCount.className = "play-count";
    playCount.textContent = ` (Played ${track.playCount || 0} times)`;
    titleContainer.appendChild(playCount);

    // Add click listener to play the track
    trackElement.addEventListener("click", () => {
      console.log(`Playing track: ${track.path}`);
      setCurrentPlaylist(playlistName);
      setCurrentTrackIndex(index);
      loadTrack(track.path);
      playTrack();
    });

    // Delete Button Container
    const deleteContainer = document.createElement("div");
    deleteContainer.className = "delete-container";

    const deleteButton = document.createElement("button");
    deleteButton.className = "delete-track";
    deleteButton.textContent = "X";

    // Event listener for deleting the track
    deleteButton.addEventListener("click", (e) => {
      e.stopPropagation(); // Prevent triggering track play
      tracks.splice(index, 1); // Remove the track from the playlist
      updatePlaylist(playlistName, tracks); // Save the playlist changes
      renderPlaylistTracks(playlistName); // Re-render the playlist
    });

    deleteContainer.appendChild(deleteButton);
    trackElement.appendChild(titleContainer);
    trackElement.appendChild(deleteContainer);
    playlistDiv.appendChild(trackElement);
  });

  console.log(`Tracks rendered for playlist "${playlistName}":`, tracks);
}
