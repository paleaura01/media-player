// preload.js

const { contextBridge, ipcRenderer } = require("electron");

contextBridge.exposeInMainWorld("audioPlayer", {
  playTrack: (filePath, config, seekTime = 0) =>
    ipcRenderer.invoke("audio:playTrack", filePath, config, seekTime),
  stopPlayback: (sessionId) =>
    ipcRenderer.invoke("audio:stopPlayback", sessionId), // Accept sessionId
  getCurrentTime: () => ipcRenderer.invoke("audio:getCurrentTime"),
  seekTo: (seekTime) => ipcRenderer.invoke("audio:seekTo", seekTime),
});


contextBridge.exposeInMainWorld("electron", {
  selectFiles: () => ipcRenderer.invoke("dialog:selectFiles"),
});
