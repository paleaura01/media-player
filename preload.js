// preload.js

const { contextBridge, ipcRenderer } = require('electron');

contextBridge.exposeInMainWorld('electron', {
  selectFiles: async () => ipcRenderer.invoke('dialog:selectFiles'), // File selection only
  selectFolderOrFiles: async () => ipcRenderer.invoke('dialog:selectFolderOrFiles'),
  readDirectory: async (folderPath) => ipcRenderer.invoke('readDirectory', folderPath),
});
