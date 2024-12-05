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
  const sessionId = Date.now(); // Unique session ID
  activeSessionId = sessionId;

  if (isPlaying) {
    console.log("Stopping current playback...");
    await stopPlayback(activeSessionId);
  }

  const speakerConfig = {
    channels: 2,
    bitDepth: 16,
    sampleRate: 44100,
  };

  currentTrackPath = filePath;

  try {
    await window.audioPlayer.playTrack(filePath, speakerConfig, sessionId);

    if (activeSessionId !== sessionId) {
      console.warn("Session mismatch. Ignoring playback.");
      return;
    }

    isPlaying = true;
    console.log(`Playback started for: ${filePath}`);
    startProgressUpdater();
  } catch (err) {
    console.error("Error starting playback:", err.message);
  }
}


export async function stopPlayback(requestedSessionId = null) {
  console.log("Stopping playback...");
  isPlaying = false;

  await window.audioPlayer.stopPlayback(requestedSessionId || activeSessionId);
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
    try {
      const { currentTime, duration } = await window.audioPlayer.getCurrentTime();

      const progressBar = document.getElementById("progress-bar");
      const timeDisplay = document.getElementById("time-display");

      if (progressBar && timeDisplay && duration > 0) {
        progressBar.style.width = `${(currentTime / duration) * 100}%`;
        timeDisplay.textContent = `${formatTime(currentTime)} / ${formatTime(duration)}`;
        document.getElementById("track-title").textContent = `Playing: ${currentTrackPath}`;
      } else {
        console.log("[PROGRESS UPDATER] Timer and progress bar not initialized.");
      }
    } catch (error) {
      console.error("Error updating progress:", error.message);
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
  const playlistTracks = document.querySelectorAll(".track");
  playlistTracks.forEach((trackElement) => {
    if (trackElement.dataset.path === currentTrackPath) {
      trackElement.classList.add("selected");
      trackElement.scrollIntoView({ behavior: "smooth", block: "nearest" });
    } else {
      trackElement.classList.remove("selected");
    }
  });
}