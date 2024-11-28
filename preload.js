const { app, BrowserWindow, ipcMain, dialog,  contextBridge, ipcRenderer } = require("electron");
const path = require("path");
const fs = require("fs");

let mainWindow;

app.on("ready", () => {
  mainWindow = new BrowserWindow({
    width: 800,
    height: 600,
    webPreferences: {
      preload: path.join(__dirname, "preload.js"),
      contextIsolation: true,
    },
  });

  mainWindow.loadFile("index.html");
  mainWindow.webContents.openDevTools();
});

// Handle folder selection
ipcMain.handle("dialog:selectFolder", async () => {
  const result = await dialog.showOpenDialog({
    properties: ["openDirectory"],
  });
  return result.canceled ? null : result.filePaths[0];
});

// Handle file or folder selection
ipcMain.handle("dialog:selectFolderOrFiles", async () => {
  const result = await dialog.showOpenDialog({
    properties: ["openFile", "openDirectory", "multiSelections"],
    filters: [
      { name: "Audio Files", extensions: ["mp3", "wav", "ogg", "opus"] },
    ],
  });

  if (result.canceled) return null;

  return result.filePaths.map((filePath) => ({
    name: path.basename(filePath),
    path: filePath,
  }));
});

// Read directory contents
ipcMain.handle("readDirectory", (event, folderPath) => {
  const files = fs.readdirSync(folderPath);
  return files
    .filter((file) =>
      [".mp3", ".wav", ".ogg", ".opus"].some((ext) => file.endsWith(ext))
    )
    .map((file) => ({
      name: file,
      path: path.join(folderPath, file),
    }));
});

contextBridge.exposeInMainWorld("electron", {
    selectFolder: () => ipcRenderer.invoke("dialog:selectFolder"),
    selectFolderOrFiles: () => ipcRenderer.invoke("dialog:selectFolderOrFiles"),
    readDirectory: (folderPath) => ipcRenderer.invoke("readDirectory", folderPath),
  });