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

const { audioPlayer } = window;

export function toggleRepeat() {
  repeatMode = !repeatMode;
  console.log(`[TOGGLE REPEAT] Repeat mode is now ${repeatMode ? "ON" : "OFF"}.`);
  return repeatMode;
}

export function toggleShuffle() {
  shuffleMode = !shuffleMode;
  console.log(`[TOGGLE SHUFFLE] Shuffle mode is now ${shuffleMode ? "ON" : "OFF"}.`);
  return shuffleMode;
}

export function setCurrentPlaylist(playlistName) {
  currentPlaylistName = playlistName;
  currentPlaylist = playlistName ? getPlaylist(playlistName) : [];
  currentTrackIndex = 0;
  console.log(`[SET PLAYLIST] Playlist set to "${playlistName}" with ${currentPlaylist.length} tracks.`);
}

export function setCurrentTrackIndex(index) {
  currentTrackIndex = index;
  console.log(`[SET TRACK INDEX] Track index set to ${index}.`);
}

export async function loadTrack(filePath) {
  const sessionId = Date.now(); // Unique session ID
  activeSessionId = sessionId;

  console.log(`[LOAD TRACK] Loading track: ${filePath}`);

  if (isPlaying) {
    console.log(`[LOAD TRACK] Stopping current playback before loading new track.`);
    await stopPlayback(activeSessionId);
  }

  const speakerConfig = {
    channels: 2,
    bitDepth: 16,
    sampleRate: 44100,
  };

  currentTrackPath = filePath;

  try {
    console.log(`[LOAD TRACK] Requesting playback for: ${filePath}`);
    await audioPlayer.playTrack(filePath, speakerConfig, sessionId);

    if (activeSessionId !== sessionId) {
      console.warn(`[LOAD TRACK] Session mismatch detected. Ignoring playback request.`);
      return;
    }

    isPlaying = true;
    console.log(`[LOAD TRACK] Playback started for: ${filePath}`);
    await startProgressUpdater();
    highlightCurrentTrack();
    updateTrackScroller(filePath);
  } catch (err) {
    console.error(`[LOAD TRACK] Error starting playback: ${err.message}`);
  }
}

export async function stopPlayback(requestedSessionId = null) {
  console.log(`[STOP PLAYBACK] Stopping playback for session: ${requestedSessionId || activeSessionId}`);
  isPlaying = false;

  clearProgressUpdater();

  try {
    await audioPlayer.stopPlayback(requestedSessionId || activeSessionId);
    console.log(`[STOP PLAYBACK] Playback successfully stopped.`);
  } catch (err) {
    console.error(`[STOP PLAYBACK] Error stopping playback: ${err.message}`);
  }
}

export function restoreLastTrack() {
  const currentTrackInfo = JSON.parse(localStorage.getItem("currentTrackInfo"));

  if (currentTrackInfo) {
    console.log(`[RESTORE TRACK] Restoring track:`, currentTrackInfo);
    setCurrentPlaylist(currentTrackInfo.playlistName);

    const trackIndex = currentPlaylist.findIndex((t) => t.path === currentTrackInfo.trackPath);
    if (trackIndex !== -1) {
      setCurrentTrackIndex(trackIndex);
      loadTrack(currentTrackInfo.trackPath);
    } else {
      console.warn(`[RESTORE TRACK] Track not found in playlist.`);
    }
  }
}

export function playTrack(filePath) {
  console.log(`[PLAY TRACK] Playing track: ${filePath}`);
  loadTrack(filePath);
}

export function stopTrack() {
  console.log(`[STOP TRACK] Stopping track.`);
  stopPlayback();
}

export function nextTrack() {
  if (!currentPlaylist.length) {
    console.warn(`[NEXT TRACK] No playlist loaded.`);
    return;
  }

  currentTrackIndex = (currentTrackIndex + 1) % currentPlaylist.length;
  console.log(`[NEXT TRACK] Switching to track index ${currentTrackIndex}.`);
  loadTrack(currentPlaylist[currentTrackIndex].path);
}

export function prevTrack() {
  if (!currentPlaylist.length) {
    console.warn(`[PREV TRACK] No playlist loaded.`);
    return;
  }

  currentTrackIndex = (currentTrackIndex - 1 + currentPlaylist.length) % currentPlaylist.length;
  console.log(`[PREV TRACK] Switching to track index ${currentTrackIndex}.`);
  loadTrack(currentPlaylist[currentTrackIndex].path);
}

function formatTime(seconds) {
  const mins = Math.floor(seconds / 60);
  const secs = Math.floor(seconds % 60);
  return `${mins}:${secs < 10 ? "0" : ""}${secs}`;
}

async function startProgressUpdater() {
  clearProgressUpdater();
  console.log(`[PROGRESS UPDATER] Starting progress updater for: ${currentTrackPath}`);

  for (let retries = 0; retries < 10; retries++) {
    const progressBar = document.getElementById("progress-bar");
    const timeDisplay = document.getElementById("time-display");

    if (progressBar && timeDisplay) {
      console.log("[PROGRESS UPDATER] DOM elements ready.");
      break;
    }

    console.warn(`[PROGRESS UPDATER] DOM elements not ready. Retrying... (${retries + 1}/10)`);
    await new Promise((resolve) => setTimeout(resolve, 100));
  }

  const progressBar = document.getElementById("progress-bar");
  const timeDisplay = document.getElementById("time-display");

  if (!progressBar || !timeDisplay) {
    console.error("[PROGRESS UPDATER] Failed to initialize: Required DOM elements not found.");
    return;
  }

  progressInterval = setInterval(async () => {
    try {
      const { currentTime, duration } = await audioPlayer.getCurrentTime();

      if (progressBar && timeDisplay && duration > 0) {
        progressBar.style.width = `${(currentTime / duration) * 100}%`;
        timeDisplay.textContent = `${formatTime(currentTime)} / ${formatTime(duration)}`;
        console.log(`[PROGRESS UPDATER] Updated progress: ${currentTime} / ${duration}`);
      } else {
        console.warn("[PROGRESS UPDATER] Timer and progress bar not initialized.");
      }
    } catch (error) {
      console.error(`[PROGRESS UPDATER] Error updating progress: ${error.message}`);
    }
  }, 500);
}

function clearProgressUpdater() {
  if (progressInterval) {
    console.log(`[PROGRESS UPDATER] Clearing existing progress updater.`);
    clearInterval(progressInterval);
    progressInterval = null;
  }
}

function highlightCurrentTrack() {
  console.log(`[HIGHLIGHT TRACK] Highlighting track: ${currentTrackPath}`);
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

function updateTrackScroller(filePath) {
  const scroller = document.getElementById("track-scroller");
  if (scroller) {
    scroller.textContent = `Playing: ${filePath.split("\\").pop()}`;
    console.log(`[TRACK SCROLLER] Updated scroller with: ${filePath}`);
  } else {
    console.warn("[TRACK SCROLLER] Scroller element not found.");
  }
}
