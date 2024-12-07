// renderer/player.js

import { getPlaylist } from "./playlists.js";
import { generateUUID } from "./uuid.js";

let currentPlaylistName = null;
let currentPlaylist = [];
let currentTrackIndex = 0;
let currentTrackPath = null;
let repeatMode = false;
let shuffleMode = false;
let isPlaying = false;
let progressInterval = null;
let activeSessionId = null;
let isLoading = false; // Prevent overlapping track loads

const { audioPlayer } = window;

export function toggleRepeat() {
  repeatMode = !repeatMode;
  logWithTimestamp(`[TOGGLE REPEAT] Repeat mode is now ${repeatMode ? "ON" : "OFF"}.`);
  return repeatMode;
}

export function toggleShuffle() {
  shuffleMode = !shuffleMode;
  logWithTimestamp(`[TOGGLE SHUFFLE] Shuffle mode is now ${shuffleMode ? "ON" : "OFF"}.`);
  return shuffleMode;
}

export function setCurrentPlaylist(playlistName) {
  currentPlaylistName = playlistName;
  currentPlaylist = playlistName ? getPlaylist(playlistName) : [];
  currentTrackIndex = 0;
  logWithTimestamp(`[SET PLAYLIST] Playlist set to "${playlistName}" with ${currentPlaylist.length} tracks.`);
}

export function setCurrentTrackIndex(index) {
  currentTrackIndex = index;
  logWithTimestamp(`[SET TRACK INDEX] Track index set to ${index}.`);
}

export async function loadTrack(filePath) {
  if (isLoading) {
    logWithTimestamp(`[LOAD TRACK] Another load operation is already in progress.`);
    return;
  }

  const sessionId = generateUUID();
  logWithTimestamp(`[LOAD TRACK] Attempting to load track: ${filePath} with session ID: ${sessionId}`);
  isLoading = true;

  try {
    if (isPlaying) {
      logWithTimestamp(`[LOAD TRACK] Stopping current playback (Active Session: ${activeSessionId}).`);
      await stopPlayback(activeSessionId);
    }

    activeSessionId = sessionId;
    currentTrackPath = filePath;

    const speakerConfig = {
      channels: 2,
      bitDepth: 16,
      sampleRate: 44100,
    };

    const { success, trackDuration } = await audioPlayer.playTrack(filePath, speakerConfig, sessionId);

    if (!success) {
      logWithTimestamp(`[LOAD TRACK] Playback failed for: ${filePath}`);
      return;
    }

    if (activeSessionId !== sessionId) {
      logWithTimestamp(`[LOAD TRACK] Session mismatch. Stopping playback for session ID: ${sessionId}`);
      await stopPlayback(sessionId);
      return;
    }

    isPlaying = true;
    logWithTimestamp(`[LOAD TRACK] Playback started successfully for: ${filePath}`);
    updateTrackScroller(filePath);

    if (trackDuration > 0) {
      logWithTimestamp(`[LOAD TRACK] Track duration set to ${trackDuration}s.`);
    } else {
      logWithTimestamp(`[LOAD TRACK] Track duration is unavailable.`);
    }

    await startProgressUpdater(trackDuration); // Pass track duration to progress updater
  } catch (error) {
    logWithTimestamp(`[LOAD TRACK] Error: ${error.message}`);
  } finally {
    isLoading = false;
  }
}


export async function stopPlayback(requestedSessionId = null) {
  logWithTimestamp(`[STOP PLAYBACK] Stop requested for session: ${requestedSessionId}`);

  if (requestedSessionId && requestedSessionId !== activeSessionId) {
    logWithTimestamp(`[STOP PLAYBACK] Ignoring stop request for non-active session: ${requestedSessionId}`);
    return;
  }

  logWithTimestamp(`[STOP PLAYBACK] Stopping playback for active session: ${activeSessionId}`);
  isPlaying = false;

  try {
    await audioPlayer.stopPlayback();
    logWithTimestamp(`[STOP PLAYBACK] Playback stopped successfully for session: ${activeSessionId}`);
  } catch (err) {
    logWithTimestamp(`[STOP PLAYBACK] Error stopping playback: ${err.message}`);
  } finally {
    if (requestedSessionId === activeSessionId || requestedSessionId === null) {
      logWithTimestamp(`[STOP PLAYBACK] Clearing active session: ${activeSessionId}`);
      activeSessionId = null;
    }
  }
}

function logWithTimestamp(message) {
  const timestamp = new Date().toISOString();
  console.log(`[${timestamp}] ${message}`);
}

export function restoreLastTrack() {
  const currentTrackInfo = JSON.parse(localStorage.getItem("currentTrackInfo"));

  if (currentTrackInfo) {
    logWithTimestamp(`[RESTORE TRACK] Restoring track: ${JSON.stringify(currentTrackInfo)}`);
    setCurrentPlaylist(currentTrackInfo.playlistName);

    const trackIndex = currentPlaylist.findIndex((t) => t.path === currentTrackInfo.trackPath);
    if (trackIndex !== -1) {
      setCurrentTrackIndex(trackIndex);
      loadTrack(currentTrackInfo.trackPath).catch((err) => {
        logWithTimestamp(`[RESTORE TRACK] Error restoring track: ${err.message}`);
      });
    } else {
      logWithTimestamp(`[RESTORE TRACK] Track not found in playlist.`);
    }
  }
}

export function playTrack() {
  if (currentTrackPath) {
    logWithTimestamp(`[PLAY TRACK] Resuming track: ${currentTrackPath}`);
    loadTrack(currentTrackPath);
  } else {
    logWithTimestamp("[PLAY TRACK] No track is currently selected.");
  }
}

export function stopTrack() {
  logWithTimestamp(`[STOP TRACK] Stopping track.`);
  stopPlayback();
}

export function nextTrack() {
  if (!currentPlaylist.length) {
    logWithTimestamp(`[NEXT TRACK] No playlist loaded.`);
    return;
  }

  currentTrackIndex = (currentTrackIndex + 1) % currentPlaylist.length;
  logWithTimestamp(`[NEXT TRACK] Switching to track index ${currentTrackIndex}.`);
  loadTrack(currentPlaylist[currentTrackIndex].path);
}

export function prevTrack() {
  if (!currentPlaylist.length) {
    logWithTimestamp(`[PREV TRACK] No playlist loaded.`);
    return;
  }

  currentTrackIndex = (currentTrackIndex - 1 + currentPlaylist.length) % currentPlaylist.length;
  logWithTimestamp(`[PREV TRACK] Switching to track index ${currentTrackIndex}.`);
  loadTrack(currentPlaylist[currentTrackIndex].path);
}

function formatTime(seconds) {
  const mins = Math.floor(seconds / 60);
  const secs = Math.floor(seconds % 60);
  return `${mins}:${secs < 10 ? "0" : ""}${secs}`;
}

async function startProgressUpdater(trackDuration) {
  clearProgressUpdater();

  const progressBar = document.getElementById("progress-bar");
  const timeDisplay = document.getElementById("time-display");

  if (!progressBar || !timeDisplay) {
    logWithTimestamp("[PROGRESS UPDATER] Required DOM elements not found.");
    return;
  }

  progressInterval = setInterval(async () => {
    try {
      const { currentTime, duration } = await audioPlayer.getCurrentTime();

      // Use passed trackDuration as a fallback if duration is unavailable
      const effectiveDuration = duration > 0 ? duration : trackDuration;

      if (effectiveDuration > 0) {
        const progressPercent = (currentTime / effectiveDuration) * 100;
        progressBar.style.width = `${progressPercent}%`;
        timeDisplay.textContent = `${formatTime(currentTime)} / ${formatTime(effectiveDuration)}`;
        logWithTimestamp(
          `[PROGRESS UPDATER] Progress: ${currentTime.toFixed(2)}s / ${effectiveDuration.toFixed(2)}s`
        );
      } else {
        timeDisplay.textContent = "00:00 / 00:00";
        progressBar.style.width = "0%";
        logWithTimestamp("[PROGRESS UPDATER] Duration unavailable, showing default.");
      }
    } catch (error) {
      logWithTimestamp(`[PROGRESS UPDATER] Error updating progress: ${error.message}`);
    }
  }, 500);
}




function clearProgressUpdater() {
  if (progressInterval) {
    clearInterval(progressInterval);
    progressInterval = null;
  }
}

function highlightCurrentTrack() {
  logWithTimestamp(`[HIGHLIGHT TRACK] Highlighting track: ${currentTrackPath}`);
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
  const trackTitle = document.getElementById("track-title");

  if (scroller) {
    const trackName = filePath.split("\\").pop();
    scroller.textContent = `Playing: ${trackName}`;
    logWithTimestamp(`[TRACK SCROLLER] Updated scroller with: ${trackName}`);
  } else {
    logWithTimestamp("[TRACK SCROLLER] Scroller element not found.");
  }

  if (trackTitle) {
    trackTitle.textContent = filePath ? `Now Playing: ${filePath.split("\\").pop()}` : "No track playing";
    logWithTimestamp(`[TRACK TITLE] Updated track title to: ${trackTitle.textContent}`);
  }
}

export function getCurrentTrackPath() {
  return currentTrackPath;
}

export function getIsPlaying() {
  return isPlaying;
}
