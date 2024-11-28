// renderer/player.js

import { playlists } from "./playlists.js";

let sound = null;
let currentTrackIndex = 0;
let shuffleMode = false;
let repeatMode = false;

export function loadTrack(playlistName, index) {
  if (sound) sound.unload();
  const track = playlists[playlistName][index];
  sound = new Howl({
    src: [track.path],
    html5: true,
    onend: () => {
      if (repeatMode) {
        loadTrack(playlistName, index);
        playTrack();
      } else {
        nextTrack(playlistName);
      }
    },
  });
  currentTrackIndex = index;
}

export function playTrack() {
  if (sound) sound.play();
}

export function pauseTrack() {
  if (sound) sound.pause();
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
