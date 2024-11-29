import { loadTrack, playTrack } from "./player.js";
import { getPlaylist } from "./playlists.js";
import { getCurrentPlaylist } from "./playlistManager.js";

let currentTrackIndex = -1;

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
  playlistDiv.innerHTML = tracks
    .map(
      (track, index) =>
        `<div class="track" data-index="${index}" data-path="${track.path}">${track.name}</div>`
    )
    .join("");

  playlistDiv.querySelectorAll(".track").forEach((trackElement) => {
    trackElement.addEventListener("click", () => {
      const trackPath = trackElement.getAttribute("data-path");
      const trackIndex = parseInt(trackElement.getAttribute("data-index"), 10);

      if (trackPath) {
        window.electron.fileExists(trackPath).then((exists) => {
          if (exists) {
            currentTrackIndex = trackIndex;
            console.log(`Playing track: ${trackPath}`);
            loadTrack(trackPath);
            playTrack();
          } else {
            alertForMissingFile(trackPath);
            playNextTrack();
          }
        });
      }
    });
  });

  console.log(`Tracks rendered for playlist "${playlistName}":`, tracks);
}

function alertForMissingFile(filePath) {
  const alertDiv = document.createElement("div");
  alertDiv.classList.add("alert");
  alertDiv.textContent = `File does not exist: ${filePath}`;
  document.body.appendChild(alertDiv);

  setTimeout(() => {
    alertDiv.remove();
  }, 2000);
}

export function playNextTrack() {
  const playlistName = getCurrentPlaylist();
  if (!playlistName) return;

  const tracks = getPlaylist(playlistName);
  if (currentTrackIndex === -1 || currentTrackIndex >= tracks.length - 1) {
    console.log("No more tracks to play.");
    return;
  }

  currentTrackIndex += 1;
  const nextTrack = tracks[currentTrackIndex];
  if (nextTrack) {
    console.log(`Playing next track: ${nextTrack.path}`);
    loadTrack(nextTrack.path);
    playTrack();
  }
}
