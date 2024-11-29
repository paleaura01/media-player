// renderer/player.js

import { Howl } from "howler";

let sound = null;

export function loadTrack(filePath) {
  if (sound) sound.unload(); // Unload the previous sound
  console.log(`Loading track: ${filePath}`);

  // Convert the file path to use the `file://` protocol
  const audioUrl = `file://${filePath.replace(/\\/g, "/")}`;

  sound = new Howl({
    src: [audioUrl],
    html5: true, // Use HTML5 Audio for larger files
    onplay: () => console.log(`Playing: ${audioUrl}`),
    onend: () => console.log("Track ended."),
    onloaderror: (id, error) => console.error("Error loading audio:", error),
    onplayerror: (id, error) => console.error("Error playing audio:", error),
  });

  sound.load();
}

export function playTrack() {
  if (sound) {
    sound.play();
    console.log("Playback started.");
  } else {
    console.warn("No track loaded to play.");
  }
}



export function pauseTrack() {
  if (sound) {
    sound.pause();
    console.log("Playback paused.");
  } else {
    console.warn("No track loaded to pause.");
  }
}



export function nextTrack(playlistName) {
  const playlist = playlists[playlistName];
  currentTrackIndex = shuffleMode
    ? Math.floor(Math.random() * playlist.length)
    : (currentTrackIndex + 1) % playlist.length;
  loadTrack(playlistName, currentTrackIndex);
  playTrack();
}

export function prevTrack(playlistName) {
  const playlist = playlists[playlistName];
  currentTrackIndex =
    (currentTrackIndex - 1 + playlist.length) % playlist.length;
  loadTrack(playlistName, currentTrackIndex);
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