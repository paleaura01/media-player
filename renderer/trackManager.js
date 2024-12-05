// renderer/trackManager.js

import { getPlaylist, updatePlaylist } from "./playlists.js";
import {   setCurrentPlaylist, 
  setCurrentTrackIndex, 
  loadTrack, 
  playTrack  } from "./player.js"; // Ensure these are imported for playback

export function renderPlaylistTracks(playlistName) {
  const playlistDiv = document.getElementById("playlist");
  if (!playlistDiv) {
    console.error("Playlist display area not found!");
    return;
  }

  if (!playlistName) {
    playlistDiv.innerHTML = "No playlist selected.";
    return;
  }

  const tracks = getPlaylist(playlistName);
  playlistDiv.innerHTML = ""; // Clear the playlist container

  tracks.forEach((track, index) => {
    
    const trackElement = document.createElement("div");
    trackElement.className = "track";

    // Title Container
    const titleContainer = document.createElement("div");
    titleContainer.className = "track-title";
    titleContainer.textContent = track.name;

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
