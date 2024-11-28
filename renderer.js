// renderer.js
let playlist = [];
let currentTrackIndex = 0;
let sound = null;

// Select a folder and load files
const folderSelector = document.getElementById("folder-selector");
folderSelector.addEventListener("click", async () => {
  const folderPath = await window.electron.selectFolder();
  if (!folderPath) return;

  loadFolder(folderPath);
});

// Load files from a folder into the playlist
function loadFolder(folderPath) {
  playlist = window.electron.readDirectory(folderPath);
  if (playlist.length === 0) {
    console.log("No supported audio files found in the selected folder.");
    return;
  }

  loadTrack(0);
  renderPlaylist();
}


// Load a track
function loadTrack(index) {
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
  updateCurrentTrackDisplay();
  console.log(`Loaded track: ${track.name}`);
}

// Play the current track
function playTrack() {
  if (sound) {
    sound.play();
    console.log("Playing track.");
  }
}

// Pause the current track
function pauseTrack() {
  if (sound) {
    sound.pause();
    console.log("Paused track.");
  }
}

// Stop playback
function stopTrack() {
  if (sound) {
    sound.stop();
    console.log("Stopped playback.");
  }
}

// Play the next track
function nextTrack() {
  const nextIndex = (currentTrackIndex + 1) % playlist.length;
  loadTrack(nextIndex);
  playTrack();
}

// Play the previous track
function prevTrack() {
  const prevIndex = (currentTrackIndex - 1 + playlist.length) % playlist.length;
  loadTrack(prevIndex);
  playTrack();
}

// Render the playlist
function renderPlaylist() {
  const playlistDiv = document.getElementById("playlist");
  playlistDiv.innerHTML = playlist
    .map((track, index) => {
      return `<div class="track ${
        index === currentTrackIndex ? "current" : ""
      }">${track.name}</div>`;
    })
    .join("");
}

// Update the current track display
function updateCurrentTrackDisplay() {
  const currentTrackDiv = document.getElementById("current-track");
  currentTrackDiv.textContent = `Now playing: ${
    playlist[currentTrackIndex]?.name || "No track playing"
  }`;
}

// Attach event listeners to controls
document.getElementById("play").addEventListener("click", playTrack);
document.getElementById("pause").addEventListener("click", pauseTrack);
document.getElementById("stop").addEventListener("click", stopTrack);
document.getElementById("next").addEventListener("click", nextTrack);
document.getElementById("prev").addEventListener("click", prevTrack);
