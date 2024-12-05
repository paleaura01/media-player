// preload.js

const { contextBridge, ipcRenderer } = require("electron");

contextBridge.exposeInMainWorld("audioPlayer", {
  playTrack: (filePath, config) => ipcRenderer.invoke("audio:playTrack", filePath, config),
  stopPlayback: () => ipcRenderer.invoke("audio:stopPlayback"),
  getCurrentTime: () => ipcRenderer.invoke("audio:getCurrentTime"),
  seekTo: (seekTime) => ipcRenderer.invoke("audio:seekTo", seekTime),
});


contextBridge.exposeInMainWorld("electron", {
  selectFiles: () => ipcRenderer.invoke("dialog:selectFiles"),
});
