// renderer/playlists.js

export const playlists = {};

export function loadPlaylists() {
  const savedPlaylists = JSON.parse(localStorage.getItem("playlists"));
  if (savedPlaylists) {
    Object.assign(playlists, savedPlaylists);
    console.log("Playlists loaded:", playlists);
  } else {
    console.log("No saved playlists found.");
  }
}

export function savePlaylists() {
  localStorage.setItem("playlists", JSON.stringify(playlists));
  console.log("Playlists saved:", playlists);
}

export function addPlaylist(name) {
  if (playlists[name]) {
    console.warn(`Playlist '${name}' already exists.`);
    return false;
  }
  playlists[name] = [];
  savePlaylists();
  return true;
}

export function deletePlaylist(name) {
  if (!playlists[name]) {
    console.warn(`Playlist '${name}' does not exist.`);
    return;
  }
  delete playlists[name];
  savePlaylists();
  console.log(`Playlist '${name}' deleted.`);
}

export function getPlaylist(name) {
  return playlists[name] || [];
}

export function updatePlaylist(name, newTracks) {
  if (!playlists[name]) {
    console.warn(`Playlist '${name}' does not exist.`);
    return;
  }
  playlists[name] = newTracks;
  savePlaylists(); // Save updated playlists to localStorage
  console.log(`Playlist '${name}' updated.`);
}
