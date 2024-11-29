// renderer/player.js

import { Howl } from 'howler';

let sound = null;

export function loadTrack(filePath) {
  if (sound) sound.unload();

  const localUrl = `local://${filePath}`; // Convert file path to local protocol URL

  console.log('Loading track:', localUrl);

  sound = new Howl({
    src: [localUrl],
    html5: true, // Ensures compatibility for large files and Electron
    onload: () => console.log('Track loaded:', localUrl),
    onplay: () => console.log('Playing track...'),
    onend: () => console.log('Track ended.'),
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
