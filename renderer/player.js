// renderer/player.js

import { Howl } from 'howler';
import { getPlaylist } from './playlists.js';

let sound = null;
let currentPlaylistName = null;
let currentPlaylist = [];
let currentTrackIndex = 0;
let shuffleMode = false;
let repeatMode = false;

export function setCurrentPlaylist(playlistName) {
  currentPlaylistName = playlistName;
  currentPlaylist = getPlaylist(playlistName);
  currentTrackIndex = 0;
}

export function setCurrentTrackIndex(index) {
  currentTrackIndex = index;
}

export function loadTrack(filePath) {
  if (sound) sound.unload();

  const localUrl = `local://${filePath}`;

  console.log('Loading track:', localUrl);

  sound = new Howl({
    src: [localUrl],
    html5: true,
    onload: () => console.log('Track loaded:', localUrl),
    onplay: () => console.log('Playing track...'),
    onend: () => {
      console.log('Track ended.');
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
    console.warn('No playlist loaded.');
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
    console.warn('No playlist loaded.');
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