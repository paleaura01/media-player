// preload.js

const { contextBridge, ipcRenderer } = require("electron");

contextBridge.exposeInMainWorld("electron", {
  selectFolderOrFiles: () => ipcRenderer.invoke("dialog:selectFolderOrFiles"),
  readDirectory: (folderPath) => ipcRenderer.invoke("readDirectory", folderPath),
});
