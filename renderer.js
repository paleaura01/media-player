let playlist = [];
let currentTrackIndex = 0;
let sound = null;
let isPaused = false;
let selectedFolderPath = null;

let playbackPositions = {};

// New variables for Shuffle and Repeat
let shuffleMode = false;
let repeatMode = false;
let playCounts = {};
let unplayedTracks = [];

// Load the last selected folder and settings if available
window.addEventListener('DOMContentLoaded', () => {
  const savedFolderPath = localStorage.getItem('selectedFolderPath');
  const savedPlaybackPositions = JSON.parse(localStorage.getItem('playbackPositions'));

  if (savedPlaybackPositions) {
    playbackPositions = savedPlaybackPositions;
  }

  if (savedFolderPath) {
    loadFolder(savedFolderPath);
  }

  // Load Shuffle and Repeat modes
  const savedShuffleMode = localStorage.getItem('shuffleMode');
  const savedRepeatMode = localStorage.getItem('repeatMode');

  if (savedShuffleMode) {
    shuffleMode = savedShuffleMode === 'true';
    const shuffleButton = document.getElementById('shuffle');
    shuffleButton.textContent = shuffleMode ? 'Shuffle On' : 'Shuffle Off';
    shuffleButton.classList.toggle('active', shuffleMode);
  }

  if (savedRepeatMode) {
    repeatMode = savedRepeatMode === 'true';
    const repeatButton = document.getElementById('repeat');
    repeatButton.textContent = repeatMode ? 'Repeat On' : 'Repeat Off';
    repeatButton.classList.toggle('active', repeatMode);
  }

  // Disable pause button initially
  document.getElementById('pause').disabled = true;
});

// Select a folder and load files
const folderSelector = document.getElementById('folder-selector');
folderSelector.addEventListener('click', async () => {
  const folderPath = await window.electron.selectFolder();
  if (!folderPath) return;

  selectedFolderPath = folderPath;
  localStorage.setItem('selectedFolderPath', folderPath);
  loadFolder(folderPath);
});

// Load files from a folder into the playlist
function loadFolder(folderPath) {
  playlist = window.electron.readDirectory(folderPath);
  if (playlist.length === 0) {
    console.log('No supported audio files found in the selected folder.');
    return;
  }

  // Initialize play counts
  playCounts = {};
  playlist.forEach((track) => {
    playCounts[track.path] = 0;
  });

  // Initialize unplayed tracks
  initializeUnplayedTracks();

  loadTrack(0);
  renderPlaylist();
}

// Initialize unplayed tracks list
function initializeUnplayedTracks() {
  unplayedTracks = [...Array(playlist.length).keys()]; // [0, 1, 2, ..., n]
}

// Load a track
function loadTrack(index) {
  // Save current track position before switching
  saveCurrentTrackPosition();

  if (sound) sound.unload();
  const track = playlist[index];
  sound = new Howl({
    src: [track.path],
    html5: true,
    onend: () => {
      nextTrack();
    },
  });

  currentTrackIndex = index;
  isPaused = false;

  // Increment play count
  playCounts[track.path] = (playCounts[track.path] || 0) + 1;

  // Remove track from unplayedTracks if present
  const unplayedIndex = unplayedTracks.indexOf(index);
  if (unplayedIndex !== -1) {
    unplayedTracks.splice(unplayedIndex, 1);
  }

  updateCurrentTrackDisplay();
  console.log(`Loaded track: ${track.name}`);

  // Resume from saved position if available
  const trackKey = track.path;
  const savedPosition = playbackPositions[trackKey];
  if (savedPosition !== undefined) {
    sound.seek(savedPosition);
    console.log(`Resumed track at position: ${savedPosition}`);
  }

  // Update the playlist display
  renderPlaylist();

  // Update button states
  document.getElementById('play').disabled = false;
  document.getElementById('pause').disabled = true;
}

// Save the current track's playback position
function saveCurrentTrackPosition() {
  if (sound) {
    const track = playlist[currentTrackIndex];
    const trackKey = track.path;
    playbackPositions[trackKey] = sound.seek();
    // Persist positions to localStorage
    localStorage.setItem('playbackPositions', JSON.stringify(playbackPositions));
    console.log(`Saved position ${playbackPositions[trackKey]} for track: ${track.name}`);
  }
}

// Play or resume the current track
function playTrack() {
  if (sound) {
    if (!sound.playing()) {
      sound.play();
      isPaused = false;
      console.log('Playing track.');
      // Disable the play button
      document.getElementById('play').disabled = true;
      document.getElementById('pause').disabled = false;
    } else {
      console.log('Track is already playing.');
    }
  } else {
    console.log('No track loaded.');
  }
}

// Pause the current track
function pauseTrack() {
  if (sound && sound.playing()) {
    sound.pause();
    isPaused = true;
    saveCurrentTrackPosition();
    console.log('Paused track.');
    // Enable the play button
    document.getElementById('play').disabled = false;
    document.getElementById('pause').disabled = true;
  }
}

// Play the next track
function nextTrack() {
  saveCurrentTrackPosition();

  if (repeatMode) {
    // Repeat the current track
    loadTrack(currentTrackIndex);
    playTrack();
    return;
  }

  if (shuffleMode) {
    if (unplayedTracks.length === 0) {
      // All tracks have been played, reset unplayedTracks
      initializeUnplayedTracks();
    }

    // Randomly select a track from unplayedTracks
    const randomIndex = Math.floor(Math.random() * unplayedTracks.length);
    const nextIndex = unplayedTracks.splice(randomIndex, 1)[0];
    loadTrack(nextIndex);
  } else {
    // Normal sequential playback
    const nextIndex = (currentTrackIndex + 1) % playlist.length;
    loadTrack(nextIndex);
  }

  playTrack();
}

// Play the previous track
function prevTrack() {
  saveCurrentTrackPosition();

  // For previous track, we'll navigate sequentially
  const prevIndex = (currentTrackIndex - 1 + playlist.length) % playlist.length;
  loadTrack(prevIndex);
  playTrack();
}

// Render the playlist
function renderPlaylist() {
  const playlistDiv = document.getElementById('playlist');
  playlistDiv.innerHTML = playlist
    .map((track, index) => {
      return `<div class="track ${
        index === currentTrackIndex ? 'current' : ''
      }">${track.name} <span class="play-count">(Played ${playCounts[track.path]} times)</span></div>`;
    })
    .join('');
}

// Update the current track display
function updateCurrentTrackDisplay() {
  const currentTrackDiv = document.getElementById('current-track');
  currentTrackDiv.textContent = `Now playing: ${
    playlist[currentTrackIndex]?.name || 'No track playing'
  }`;
}

// Attach event listeners to controls
document.getElementById('play').addEventListener('click', playTrack);
document.getElementById('pause').addEventListener('click', pauseTrack);
document.getElementById('next').addEventListener('click', nextTrack);
document.getElementById('prev').addEventListener('click', prevTrack);

// Toggle Shuffle Mode
document.getElementById('shuffle').addEventListener('click', () => {
  shuffleMode = !shuffleMode;
  const shuffleButton = document.getElementById('shuffle');
  shuffleButton.textContent = shuffleMode ? 'Shuffle On' : 'Shuffle Off';
  shuffleButton.classList.toggle('active', shuffleMode);

  localStorage.setItem('shuffleMode', shuffleMode);

  if (shuffleMode) {
    initializeUnplayedTracks();
  }
});

// Toggle Repeat Mode
document.getElementById('repeat').addEventListener('click', () => {
  repeatMode = !repeatMode;
  const repeatButton = document.getElementById('repeat');
  repeatButton.textContent = repeatMode ? 'Repeat On' : 'Repeat Off';
  repeatButton.classList.toggle('active', repeatMode);

  localStorage.setItem('repeatMode', repeatMode);
});

// Save playback position before closing the window
window.addEventListener('beforeunload', () => {
  saveCurrentTrackPosition();
});