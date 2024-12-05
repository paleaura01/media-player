// renderer/player.js

import { Howl } from 'howler';
import { getPlaylist } from './playlists.js';

let sound = null;
let currentPlaylistName = null;
let currentPlaylist = [];
let currentTrackIndex = 0;
let shuffleMode = false;
let repeatMode = false;
let progressInterval = null;

export function setCurrentPlaylist(playlistName) {
  currentPlaylistName = playlistName;
  currentPlaylist = playlistName ? getPlaylist(playlistName) : [];
  currentTrackIndex = 0;
}

export function setCurrentTrackIndex(index) {
  currentTrackIndex = index;
}

export function loadTrack(filePath) {
  if (sound) {
    sound.unload();
    clearInterval(progressInterval); // Clear previous interval
  }

  const localUrl = `local://${filePath}`;

  console.log('Loading track:', localUrl);

  sound = new Howl({
    src: [localUrl],
    html5: true,
    onload: () => {
      console.log('Track loaded:', localUrl);
      updateTrackTitle(filePath);
      updateTimeDisplay();
      startProgressUpdater();
    },
    onplay: () => {
      console.log('Playing track...');
      startProgressUpdater();
    },
    onpause: () => {
      console.log('Track paused.');
      clearInterval(progressInterval);
    },
    onend: () => {
      console.log('Track ended.');
      clearInterval(progressInterval);
      if (repeatMode) {
        playTrack();
      } else {
        nextTrack();
      }
    },
    onloaderror: (id, error) => console.error('Load error:', error),
    onplayerror: (id, error) => console.error('Play error:', error),
  });
}

export function playTrack() {
  if (sound) {
    sound.play();
  } else {
    console.warn('No track loaded.');
  }
}

export function pauseTrack() {
  if (sound) {
    sound.pause();
  } else {
    console.warn('No track loaded to pause.');
  }
}

export function nextTrack() {
  if (!currentPlaylist || currentPlaylist.length === 0) {
    console.warn('No playlist loaded or playlist is empty.');
    return;
  }

  if (shuffleMode) {
    currentTrackIndex = Math.floor(Math.random() * currentPlaylist.length);
  } else {
    currentTrackIndex = (currentTrackIndex + 1) % currentPlaylist.length;
  }

  const nextTrack = currentPlaylist[currentTrackIndex];
  loadTrack(nextTrack.path);
  playTrack();
}

export function prevTrack() {
  if (!currentPlaylist || currentPlaylist.length === 0) {
    console.warn('No playlist loaded or playlist is empty.');
    return;
  }

  if (shuffleMode) {
    currentTrackIndex = Math.floor(Math.random() * currentPlaylist.length);
  } else {
    currentTrackIndex =
      (currentTrackIndex - 1 + currentPlaylist.length) % currentPlaylist.length;
  }

  const prevTrack = currentPlaylist[currentTrackIndex];
  loadTrack(prevTrack.path);
  playTrack();
}

export function toggleShuffle() {
  shuffleMode = !shuffleMode;
  return shuffleMode;
}

export function toggleRepeat() {
  repeatMode = !repeatMode;
  return repeatMode;
}

// New functions for updating the UI

function updateTrackTitle(filePath) {
  const trackTitleElement = document.getElementById('track-title');
  if (trackTitleElement) {
    const fileName = filePath.split('\\').pop().split('/').pop();
    trackTitleElement.textContent = fileName;
  }
}

function updateTimeDisplay() {
  const timeDisplay = document.getElementById('time-display');
  if (timeDisplay && sound) {
    const duration = formatTime(sound.duration());
    timeDisplay.textContent = `00:00 / ${duration}`;
  }
}

function startProgressUpdater() {
  const progressBar = document.getElementById('progress-bar');
  const timeDisplay = document.getElementById('time-display');

  if (progressBar && timeDisplay && sound) {
    clearInterval(progressInterval);
    progressInterval = setInterval(() => {
      const seek = sound.seek();
      const duration = sound.duration();
      const progressPercent = (seek / duration) * 100;
      progressBar.style.width = `${progressPercent}%`;

      const currentTime = formatTime(seek);
      const totalTime = formatTime(duration);
      timeDisplay.textContent = `${currentTime} / ${totalTime}`;
    }, 1000);
  }
}

function formatTime(seconds) {
  const minutes = Math.floor(seconds / 60) || 0;
  const secs = Math.floor(seconds % 60) || 0;
  return `${minutes < 10 ? '0' : ''}${minutes}:${secs < 10 ? '0' : ''}${secs}`;
}

// Make progress bar clickable
const progressBarContainer = document.getElementById('progress-bar-container');
if (progressBarContainer) {
  progressBarContainer.addEventListener('click', (e) => {
    if (sound) {
      const rect = progressBarContainer.getBoundingClientRect();
      const clickX = e.clientX - rect.left;
      const width = rect.width;
      const percentage = clickX / width;
      const newTime = sound.duration() * percentage;
      sound.seek(newTime);
      startProgressUpdater(); // Restart the interval to ensure proper updates
    }
  });
}
