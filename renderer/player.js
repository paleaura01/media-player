// renderer/player.js

import { Howl } from 'howler';
import { getPlaylist, updatePlaylist } from './playlists.js';
import { renderPlaylistTracks } from './trackManager.js';

let sound = null;
let currentPlaylistName = null;
let currentPlaylist = [];
let currentTrackIndex = 0;
let shuffleMode = false;
let repeatMode = false;
let progressInterval = null;

let currentTrackPath = null;

// Shuffle management variables
let currentMinPlayCount = 0;
let shuffledTracks = [];

// Store original playlist order
let originalPlaylistOrder = [];

export function setCurrentPlaylist(playlistName) {
  currentPlaylistName = playlistName;
  currentPlaylist = playlistName ? getPlaylist(playlistName) : [];
  currentTrackIndex = 0;

  // Initialize shuffle variables if shuffle mode is on
  if (shuffleMode) {
    // Save original playlist order
    originalPlaylistOrder = currentPlaylist.slice();
    shuffleCurrentPlaylist();
    // Reset currentMinPlayCount
    currentMinPlayCount = 0;
    // Update playlist display
    renderPlaylistTracks(currentPlaylistName);
  }
}

export function setCurrentTrackIndex(index) {
  currentTrackIndex = index;
}

function shuffleCurrentPlaylist() {
  currentPlaylist = shuffleArray(currentPlaylist);
}

function shuffleArray(array) {
  const shuffled = array.slice(); // Create a shallow copy
  for (let i = shuffled.length - 1; i > 0; i--) {
    const j = Math.floor(Math.random() * (i + 1));
    [shuffled[i], shuffled[j]] = [shuffled[j], shuffled[i]];
  }
  return shuffled;
}

export function loadTrack(filePath) {
  if (sound) {
    sound.unload();
    clearInterval(progressInterval); // Clear previous interval
  }

  const localUrl = `local://${filePath}`;
  console.log('Loading track:', localUrl);

  // Get the track from the current playlist
  let track = null;
  if (currentPlaylist) {
    track = currentPlaylist.find((t) => t.path === filePath);
    if (track && typeof track.lastPosition !== 'number') {
      track.lastPosition = 0; // Initialize if undefined
    }
  }

  sound = new Howl({
    src: [localUrl],
    html5: true,
    onload: () => {
      console.log('Track loaded:', localUrl);
      updateTrackTitle(filePath);
      updateTimeDisplay();
      currentTrackPath = filePath; // Update the current track path
      highlightCurrentTrack();

      // Seek to lastPosition if available
      if (track && track.lastPosition > 0) {
        sound.seek(track.lastPosition);
      }
      startProgressUpdater();
    },
    onplay: () => {
      console.log('Playing track...');
      startProgressUpdater();
    },
    onpause: () => {
      console.log('Track paused.');
      clearInterval(progressInterval);
      // Update lastPosition
      const seek = sound.seek();
      updateLastPosition(seek);
    },
    onend: () => {
      console.log('Track ended.');
      clearInterval(progressInterval);
      // Reset lastPosition to 0
      updateLastPosition(0);

      // Increment play count here, since the track has finished playing
      incrementPlayCount(filePath);

      if (repeatMode) {
        playTrack();
      } else {
        nextTrack();
        // If no next track, clear the current track path and highlight
        if (!sound || !sound.playing()) {
          currentTrackPath = null;
          highlightCurrentTrack();
        }
      }
    },
    onloaderror: (id, error) => console.error('Load error:', error),
    onplayerror: (id, error) => console.error('Play error:', error),
  });
}

function incrementPlayCount(filePath) {
  if (currentPlaylist && currentTrackPath) {
    const trackIndex = currentPlaylist.findIndex((t) => t.path === currentTrackPath);
    if (trackIndex !== -1) {
      const track = currentPlaylist[trackIndex];
      if (typeof track.playCount !== 'number') {
        track.playCount = 0; // Initialize if undefined
      }
      track.playCount += 1;
      currentPlaylist[trackIndex] = track;
      updatePlaylist(currentPlaylistName, currentPlaylist); // Save the updated playlist

      // Re-render the playlist to update play counts
      renderPlaylistTracks(currentPlaylistName);
    }
  }
}

function updateLastPosition(position) {
  if (currentPlaylist && currentTrackPath) {
    const trackIndex = currentPlaylist.findIndex((t) => t.path === currentTrackPath);
    if (trackIndex !== -1) {
      currentPlaylist[trackIndex].lastPosition = position;
      // Update the playlist
      updatePlaylist(currentPlaylistName, currentPlaylist);
    }
  }
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
    currentTrackIndex = (currentTrackIndex + 1) % currentPlaylist.length;
    const nextTrack = currentPlaylist[currentTrackIndex];
    loadTrack(nextTrack.path);
    playTrack();

    // Check if we've completed a full iteration
    if (currentTrackIndex === 0) {
      // Increment currentMinPlayCount
      currentMinPlayCount++;

      console.log(`All tracks have been played ${currentMinPlayCount} times. Reshuffling playlist.`);

      // Reshuffle currentPlaylist
      shuffleCurrentPlaylist();

      // Update playlist display
      renderPlaylistTracks(currentPlaylistName);
    }
  } else {
    currentTrackIndex = (currentTrackIndex + 1) % currentPlaylist.length;
    const nextTrack = currentPlaylist[currentTrackIndex];
    loadTrack(nextTrack.path);
    playTrack();
  }
}

export function prevTrack() {
  if (!currentPlaylist || currentPlaylist.length === 0) {
    console.warn('No playlist loaded or playlist is empty.');
    return;
  }

  if (shuffleMode) {
    // Previous track functionality is not implemented in shuffle mode
    console.warn('Previous track not available in shuffle mode.');
  } else {
    currentTrackIndex =
      (currentTrackIndex - 1 + currentPlaylist.length) % currentPlaylist.length;
    const prevTrack = currentPlaylist[currentTrackIndex];
    loadTrack(prevTrack.path);
    playTrack();
  }
}

export function toggleShuffle() {
  shuffleMode = !shuffleMode;
  if (shuffleMode) {
    // Save original playlist order
    originalPlaylistOrder = currentPlaylist.slice();
    // Shuffle currentPlaylist
    shuffleCurrentPlaylist();
    // Reset shuffle variables
    currentMinPlayCount = 0;
    currentTrackIndex = 0;
    // Update playlist display
    renderPlaylistTracks(currentPlaylistName);
  } else {
    // Restore original playlist order
    currentPlaylist = originalPlaylistOrder.slice();
    // Reset shuffle variables
    currentMinPlayCount = 0;
    currentTrackIndex = 0;
    // Update playlist display
    renderPlaylistTracks(currentPlaylistName);
  }
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

      // Update lastPosition for the current track
      updateLastPosition(seek);
    }, 1000);
  }
}

function formatTime(seconds) {
  const minutes = Math.floor(seconds / 60) || 0;
  const secs = Math.floor(seconds % 60) || 0;
  return `${minutes < 10 ? '0' : ''}${minutes}:${secs < 10 ? '0' : ''}${secs}`;
}

// Function to highlight the current track
export function highlightCurrentTrack() {
  // Highlight in playlist
  const playlistTracks = document.querySelectorAll('#playlist .track');
  playlistTracks.forEach((trackElement) => {
    if (trackElement.dataset.path === currentTrackPath) {
      trackElement.classList.add('selected');
    } else {
      trackElement.classList.remove('selected');
    }
  });

  // Highlight in library
  const libraryTracks = document.querySelectorAll('#library-tree-container .file-node');
  libraryTracks.forEach((trackElement) => {
    if (trackElement.dataset.path === currentTrackPath) {
      trackElement.classList.add('selected');
    } else {
      trackElement.classList.remove('selected');
    }
  });
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
