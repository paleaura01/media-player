// preload.js

const { contextBridge, ipcRenderer } = require('electron');

contextBridge.exposeInMainWorld('electron', {
  selectFiles: async () => ipcRenderer.invoke('dialog:selectFiles'),
  selectFolderOrFiles: async () => ipcRenderer.invoke('dialog:selectFolderOrFiles'),
  readDirectory: async (folderPath) => ipcRenderer.invoke('readDirectory', folderPath),
  fileExists: async (filePath) => ipcRenderer.invoke('fileExists', filePath), // Expose fileExists
});
