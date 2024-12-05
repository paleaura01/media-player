// renderer/player.js

import { getPlaylist, updatePlaylist } from "./playlists.js";

let currentPlaylistName = null;
let currentPlaylist = [];
let currentTrackIndex = 0;
let currentTrackPath = null;
let repeatMode = false;
let shuffleMode = false;
let isPlaying = false;
let activeSessionId = null;
let progressInterval = null;

export function toggleRepeat() {
  repeatMode = !repeatMode;
  return repeatMode;
}

export function toggleShuffle() {
  shuffleMode = !shuffleMode;
  return shuffleMode;
}

export function setCurrentPlaylist(playlistName) {
  currentPlaylistName = playlistName;
  currentPlaylist = playlistName ? getPlaylist(playlistName) : [];
  currentTrackIndex = 0;
}

export function setCurrentTrackIndex(index) {
  currentTrackIndex = index;
}

export async function loadTrack(filePath) {
  const sessionId = Date.now(); // Unique session ID for this playback
  activeSessionId = sessionId;

  if (isPlaying) {
    console.warn("Stopping current playback...");
    await stopPlayback(); // Wait for the previous playback to stop completely
  }

  const speakerConfig = {
    channels: 2,
    bitDepth: 16,
    sampleRate: 44100,
  };

  currentTrackPath = filePath;

  const track = currentPlaylist.find((t) => t.path === filePath);
  const trackTitleElement = document.getElementById("track-title");

  // Update the UI immediately to reflect the track being loaded
  if (track && trackTitleElement) {
    trackTitleElement.textContent = `Loading: ${track.name}`;
  }

  try {
    await window.audioPlayer.playTrack(filePath, speakerConfig);

    // Ensure the session is still valid after playback starts
    if (activeSessionId !== sessionId) {
      console.warn("Session mismatch detected. Ignoring outdated session playback.");
      return;
    }

    isPlaying = true;
    console.log(`Playback started for: ${filePath}`);

    if (track && trackTitleElement) {
      trackTitleElement.textContent = track.name; // Update to show the actual track name
    }

    highlightCurrentTrack();
    startProgressUpdater();
  } catch (err) {
    console.error("Error starting playback:", err.message);

    // Revert to "No track playing" if playback fails
    if (trackTitleElement) {
      trackTitleElement.textContent = "No track playing";
    }
  }
}


export async function stopPlayback() {
  console.log("Stopping playback...");
  isPlaying = false;

  await window.audioPlayer.stopPlayback(activeSessionId); // Ensure the audio is flushed
  console.log("Playback stopped.");
}


export function restoreLastTrack() {
  const currentTrackInfo = JSON.parse(localStorage.getItem("currentTrackInfo"));

  if (currentTrackInfo) {
    console.log("Restoring last playing track:", currentTrackInfo);
    setCurrentPlaylist(currentTrackInfo.playlistName);

    const trackIndex = currentPlaylist.findIndex((t) => t.path === currentTrackInfo.trackPath);
    if (trackIndex !== -1) {
      setCurrentTrackIndex(trackIndex);
      loadTrack(currentTrackInfo.trackPath);
    } else {
      console.warn("Track not found in the playlist.");
    }
  }
}

export function playTrack() {
  if (currentTrackPath) {
    loadTrack(currentTrackPath);
  } else if (currentPlaylist.length > 0) {
    loadTrack(currentPlaylist[currentTrackIndex].path);
  } else {
    console.warn("No track to play.");
  }
}

export function nextTrack() {
  if (!currentPlaylist.length) return;

  currentTrackIndex = (currentTrackIndex + 1) % currentPlaylist.length;
  loadTrack(currentPlaylist[currentTrackIndex].path);
}

export function prevTrack() {
  if (!currentPlaylist.length) return;

  currentTrackIndex =
    (currentTrackIndex - 1 + currentPlaylist.length) % currentPlaylist.length;
  loadTrack(currentPlaylist[currentTrackIndex].path);
}

function startProgressUpdater() {
  clearProgressUpdater();
  progressInterval = setInterval(async () => {
    const { currentTime, duration } = await window.audioPlayer.getCurrentTime();
    const progressBar = document.getElementById("progress-bar");
    const timeDisplay = document.getElementById("time-display");

    if (progressBar && timeDisplay) {
      const formattedCurrentTime = formatTime(currentTime);
      const formattedDuration = formatTime(duration);

      progressBar.style.width = `${(currentTime / duration) * 100}%`;
      timeDisplay.textContent = `${formattedCurrentTime} / ${formattedDuration}`;
    }
  }, 500);
}

function clearProgressUpdater() {
  clearInterval(progressInterval);
}

function formatTime(seconds) {
  const mins = Math.floor(seconds / 60);
  const secs = Math.floor(seconds % 60);
  return `${mins}:${secs < 10 ? "0" : ""}${secs}`;
}

function highlightCurrentTrack() {
  const playlistTracks = document.querySelectorAll("#playlist .track");
  playlistTracks.forEach((trackElement) => {
    if (trackElement.dataset.path === currentTrackPath) {
      trackElement.classList.add("selected");
      trackElement.scrollIntoView({ behavior: "smooth", block: "nearest" });
    } else {
      trackElement.classList.remove("selected");
    }
  });
}
