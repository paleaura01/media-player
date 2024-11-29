// preload.js

const { contextBridge, ipcRenderer } = require('electron');

contextBridge.exposeInMainWorld('electron', {
  selectFolderOrFiles: async () => ipcRenderer.invoke('dialog:selectFolderOrFiles'),
  readDirectory: async (folderPath) => ipcRenderer.invoke('readDirectory', folderPath),
});
