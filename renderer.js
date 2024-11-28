let playlists = {}; // Object to store playlists and their tracks
let currentPlaylist = null; // The name of the current playlist
let playlistTracks = []; // Tracks in the current playlist
let currentTrackIndex = 0;
let sound = null;
let isPaused = false;

let playbackPositions = {};
let shuffleMode = false;
let repeatMode = false;

window.addEventListener('DOMContentLoaded', () => {
  // Load saved playlists and settings
  const savedPlaylists = JSON.parse(localStorage.getItem('playlists'));
  if (savedPlaylists) {
    playlists = savedPlaylists;
    renderPlaylists();
  }

  const savedCurrentPlaylist = localStorage.getItem('currentPlaylist');
  if (savedCurrentPlaylist && playlists[savedCurrentPlaylist]) {
    selectPlaylist(savedCurrentPlaylist);
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

  // References to modal elements
  const modalOverlay = document.getElementById('modal-overlay');
  const playlistNameInput = document.getElementById('playlist-name-input');
  const modalOkButton = document.getElementById('modal-ok');
  const modalCancelButton = document.getElementById('modal-cancel');

  // Show the modal dialog to create a new playlist
  document.getElementById('new-playlist').addEventListener('click', () => {
    playlistNameInput.value = ''; // Clear previous input
    showModal();
  });

  // Handle modal "OK" button click
  modalOkButton.addEventListener('click', () => {
    const playlistName = playlistNameInput.value.trim();
    if (playlistName && !playlists[playlistName]) {
      playlists[playlistName] = [];
      savePlaylists();
      renderPlaylists();
      selectPlaylist(playlistName);
      hideModal();
    } else if (playlists[playlistName]) {
      alert('Playlist already exists.');
    } else {
      alert('Please enter a valid playlist name.');
    }
  });

  // Handle modal "Cancel" button click
  modalCancelButton.addEventListener('click', () => {
    hideModal();
  });

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
  });

  // Toggle Repeat Mode
  document.getElementById('repeat').addEventListener('click', () => {
    repeatMode = !repeatMode;
    const repeatButton = document.getElementById('repeat');
    repeatButton.textContent = repeatMode ? 'Repeat On' : 'Repeat Off';
    repeatButton.classList.toggle('active', repeatMode);

    localStorage.setItem('repeatMode', repeatMode);
  });

  // Initialize Drag-and-Drop Handlers
  const playlistDiv = document.getElementById('playlist');
  playlistDiv.addEventListener('dragover', allowDrop);
  playlistDiv.addEventListener('drop', dropHandler);
});

// Functions to show and hide the modal
function showModal() {
  document.getElementById('modal-overlay').classList.remove('hidden');
  // Set focus to the input field
  document.getElementById('playlist-name-input').focus();
}

function hideModal() {
  document.getElementById('modal-overlay').classList.add('hidden');
}

function renderPlaylists() {
    const playlistsUl = document.getElementById('playlists');
    playlistsUl.innerHTML = '';
    for (const playlistName in playlists) {
      const li = document.createElement('li');
  
      const playlistNameSpan = document.createElement('span');
      playlistNameSpan.textContent = playlistName;
      playlistNameSpan.addEventListener('click', () => {
        selectPlaylist(playlistName);
      });
  
      const deleteButton = document.createElement('button');
      deleteButton.textContent = '×'; // Use a cross symbol or 'Delete'
      deleteButton.classList.add('delete-playlist');
      deleteButton.addEventListener('click', (event) => {
        event.stopPropagation(); // Prevent the playlist selection event
        deletePlaylist(playlistName);
      });
  
      if (playlistName === currentPlaylist) {
        li.classList.add('selected');
      }
  
      li.appendChild(playlistNameSpan);
      li.appendChild(deleteButton);
      playlistsUl.appendChild(li);
    }
  }
  
  function deletePlaylist(playlistName) {
    if (confirm(`Are you sure you want to delete the playlist "${playlistName}"?`)) {
      delete playlists[playlistName];
      savePlaylists();
      if (playlistName === currentPlaylist) {
        currentPlaylist = null;
        playlistTracks = [];
        document.getElementById('playlist-title').textContent = 'No Playlist Selected';
        renderPlaylist();
      }
      renderPlaylists();
  
      // If no playlists remain, prompt to create a new one
      if (Object.keys(playlists).length === 0) {
        showModal();
      }
    }
  }
  

// Select a playlist
function selectPlaylist(playlistName) {
  currentPlaylist = playlistName;
  localStorage.setItem('currentPlaylist', currentPlaylist);
  playlistTracks = playlists[currentPlaylist] || [];
  currentTrackIndex = 0;
  renderPlaylist();
  document.getElementById('playlist-title').textContent = currentPlaylist;
  updatePlaylistSelection();
}

// Update playlist selection UI
function updatePlaylistSelection() {
  const playlistItems = document.getElementById('playlists').children;
  for (const item of playlistItems) {
    if (item.textContent === currentPlaylist) {
      item.classList.add('selected');
    } else {
      item.classList.remove('selected');
    }
  }
}

// Render the tracks in the current playlist
function renderPlaylist() {
  const playlistDiv = document.getElementById('playlist');
  playlistDiv.innerHTML = '';
  if (playlistTracks.length === 0) {
    playlistDiv.innerHTML = '<p>Drag and drop files or folders here to add to the playlist.</p>';
    return;
  }
  playlistTracks.forEach((track, index) => {
    const trackDiv = document.createElement('div');
    trackDiv.classList.add('track');
    if (index === currentTrackIndex) {
      trackDiv.classList.add('current');
    }
    trackDiv.textContent = track.name;
    trackDiv.addEventListener('click', () => {
      loadTrack(index);
      playTrack();
    });
    playlistDiv.appendChild(trackDiv);
  });
}

// Save playlists to localStorage
function savePlaylists() {
  localStorage.setItem('playlists', JSON.stringify(playlists));
}

// Drag-and-Drop Handlers
function allowDrop(event) {
  event.preventDefault();
}

async function dropHandler(event) {
  event.preventDefault();
  const files = event.dataTransfer.files;
  for (const file of files) {
    if (file.type === '') {
      // It's a directory
      const folderPath = file.path;
      const tracks = window.electron.readDirectory(folderPath);
      addTracksToPlaylist(tracks);
    } else {
      // It's a file
      const track = {
        name: file.name,
        path: file.path,
      };
      addTracksToPlaylist([track]);
    }
  }
}

// Add tracks to the current playlist
function addTracksToPlaylist(tracks) {
  if (!currentPlaylist) {
    alert('Please create or select a playlist first.');
    return;
  }
  playlists[currentPlaylist] = playlists[currentPlaylist].concat(tracks);
  savePlaylists();
  selectPlaylist(currentPlaylist);
}

// Handle folder selection
const folderSelector = document.getElementById("folder-selector");
folderSelector.addEventListener("click", async () => {
  const folderPath = await window.electron.selectFolder();
  if (!folderPath) return;

  const tracks = window.electron.readDirectory(folderPath);
  addTracksToPlaylist(tracks);
});

  // Add files or folders
  const addFilesButton = document.getElementById("add-files");
  addFilesButton.addEventListener("click", async () => {
    const selectedItems = await window.electron.selectFolderOrFiles();
    if (!selectedItems) return;
    addTracksToPlaylist(selectedItems);
  });


// Playback Controls

function loadTrack(index) {
  saveCurrentTrackPosition();

  if (sound) sound.unload();
  const track = playlistTracks[index];
  sound = new Howl({
    src: [track.path],
    html5: true,
    onend: () => {
      nextTrack();
    },
  });

  currentTrackIndex = index;
  isPaused = false;

  updateCurrentTrackDisplay();

  const trackKey = track.path;
  const savedPosition = playbackPositions[trackKey];
  if (savedPosition !== undefined) {
    sound.seek(savedPosition);
  }

  renderPlaylist();

  document.getElementById('play').disabled = false;
  document.getElementById('pause').disabled = true;
}

function playTrack() {
  if (sound) {
    if (!sound.playing()) {
      sound.play();
      isPaused = false;
      document.getElementById('play').disabled = true;
      document.getElementById('pause').disabled = false;
    }
  }
}

function pauseTrack() {
  if (sound && sound.playing()) {
    sound.pause();
    isPaused = true;
    saveCurrentTrackPosition();
    document.getElementById('play').disabled = false;
    document.getElementById('pause').disabled = true;
  }
}

function nextTrack() {
  saveCurrentTrackPosition();

  if (repeatMode) {
    loadTrack(currentTrackIndex);
    playTrack();
    return;
  }

  if (shuffleMode) {
    const indices = [...Array(playlistTracks.length).keys()];
    indices.splice(currentTrackIndex, 1);
    const randomIndex = indices[Math.floor(Math.random() * indices.length)];
    loadTrack(randomIndex);
  } else {
    const nextIndex = (currentTrackIndex + 1) % playlistTracks.length;
    loadTrack(nextIndex);
  }

  playTrack();
}

function prevTrack() {
  saveCurrentTrackPosition();

  const prevIndex = (currentTrackIndex - 1 + playlistTracks.length) % playlistTracks.length;
  loadTrack(prevIndex);
  playTrack();
}

function saveCurrentTrackPosition() {
  if (sound) {
    const track = playlistTracks[currentTrackIndex];
    const trackKey = track.path;
    playbackPositions[trackKey] = sound.seek();
    localStorage.setItem('playbackPositions', JSON.stringify(playbackPositions));
  }
}

function updateCurrentTrackDisplay() {
  const currentTrackDiv = document.getElementById('current-track');
  const track = playlistTracks[currentTrackIndex];
  currentTrackDiv.textContent = track ? `Now playing: ${track.name}` : 'No track playing';
}
