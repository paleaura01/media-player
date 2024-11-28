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

ipcMain.handle("readDirectory", (event, folderPath) => {
  try {
    const files = fs.readdirSync(folderPath).filter((file) => {
      const ext = path.extname(file).toLowerCase();
      return [".mp3", ".wav", ".ogg", ".opus"].includes(ext);
    });
    return files.map((file) => ({
      name: file,
      path: path.join(folderPath, file),
    }));
  } catch (error) {
    console.error("Error reading directory:", error);
    return [];
  }
});
