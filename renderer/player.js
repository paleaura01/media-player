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
    console.warn("Already playing, stopping first.");
    await stopPlayback();
  }

  const speakerConfig = {
    channels: 2,
    bitDepth: 16,
    sampleRate: 44100,
  };

  currentTrackPath = filePath;

  window.audioPlayer
    .playTrack(filePath, speakerConfig)
    .then(() => {
      if (activeSessionId !== sessionId) {
        console.warn("Playback session outdated, skipping playback.");
        return;
      }

      isPlaying = true;
      console.log("Playback started for:", filePath);

      const track = currentPlaylist.find((t) => t.path === filePath);
      const trackTitle = document.getElementById("track-title");
      if (trackTitle && track) {
        trackTitle.textContent = track.name;
      }

      highlightCurrentTrack();
      startProgressUpdater();
    })
    .catch((err) => {
      console.error("Error during playback:", err.message);
    });
}

export async function stopPlayback() {
  isPlaying = false;

  return window.audioPlayer
    .stopPlayback()
    .then(() => {
      console.log("Playback stopped.");
      clearProgressUpdater();

      const trackTitle = document.getElementById("track-title");
      if (trackTitle) {
        trackTitle.textContent = "No track playing";
      }
    })
    .catch((err) => {
      console.error("Error stopping playback:", err.message);
    });
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
