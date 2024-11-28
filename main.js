// main.js

const { app, BrowserWindow, ipcMain, dialog } = require("electron");
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
  console.log("Main window loaded with DevTools open.");
});

ipcMain.handle("dialog:selectFolderOrFiles", async () => {
  const result = await dialog.showOpenDialog({
    properties: ["openFile", "openDirectory", "multiSelections"],
    filters: [{ name: "Audio Files", extensions: ["mp3", "wav", "ogg", "opus"] }],
  });
  return result.canceled ? null : result.filePaths.map((filePath) => ({
    name: path.basename(filePath),
    path: filePath,
  }));
});

ipcMain.handle("readDirectory", async (event, folderPath) => {
    try {
      const items = fs.readdirSync(folderPath).map((fileName) => {
        const filePath = path.join(folderPath, fileName);
        const stats = fs.statSync(filePath);
  
        return {
          name: fileName,
          path: filePath,
          type: stats.isDirectory() ? "directory" : "file",
        };
      });
      return items;
    } catch (error) {
      console.error(`Error reading directory: ${folderPath}`, error);
      return [];
    }
  });
  
  
