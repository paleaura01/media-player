const { contextBridge, ipcRenderer } = require("electron");
const fs = require("fs");
const path = require("path");

contextBridge.exposeInMainWorld("electron", {
  selectFolder: () => ipcRenderer.invoke("dialog:selectFolder"),
  readDirectory: (folderPath) => {
    try {
      return fs
        .readdirSync(folderPath)
        .filter((file) =>
          [".mp3", ".wav", ".ogg", ".opus"].includes(path.extname(file).toLowerCase())
        )
        .map((file) => ({
          name: file,
          path: path.join(folderPath, file),
        }));
    } catch (err) {
      console.error("Error reading directory:", err);
      return [];
    }
  },
});
