// preload.js

const { contextBridge, ipcRenderer } = require("electron");

contextBridge.exposeInMainWorld("audioPlayer", {
  playTrack: (filePath, config, sessionId) => ipcRenderer.invoke("audio:playTrack", filePath, config, sessionId),
  stopPlayback: () => ipcRenderer.invoke("audio:stopPlayback"),
  getCurrentTime: () => ipcRenderer.invoke("audio:getCurrentTime"),
  on: (channel, listener) => {
    const validChannels = ["audio:updateProgress"];
    if (validChannels.includes(channel)) {
      ipcRenderer.on(channel, listener);
    }
  },
});


contextBridge.exposeInMainWorld("electron", {
  selectFiles: () => ipcRenderer.invoke("dialog:selectFiles"),
});
