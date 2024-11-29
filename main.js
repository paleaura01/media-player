// main.js

const { app, BrowserWindow, ipcMain, dialog } = require("electron");
const path = require("path");
const fs = require("fs");

let mainWindow;
let libraryWindow; // Declare a variable for the library window


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

ipcMain.handle("openLibraryWindow", () => {
    if (libraryWindow) {
      libraryWindow.focus();
      return;
    }
  
    libraryWindow = new BrowserWindow({
      width: 600,
      height: 400,
      webPreferences: {
        preload: path.join(__dirname, "preload.js"),
        contextIsolation: true,
      },
    });
  
    libraryWindow.loadFile("library.html");
    libraryWindow.on("closed", () => (libraryWindow = null));
    console.log("Library window opened.");
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
      console.log(`Reading directory: ${folderPath}`);
      const items = fs.readdirSync(folderPath).map((fileName) => {
        const filePath = path.join(folderPath, fileName);
        const stats = fs.statSync(filePath);
  
        return {
          name: fileName,
          path: filePath,
          type: stats.isDirectory() ? "directory" : "file",
        };
      });
      console.log(`Items found in directory "${folderPath}":`, items);
      return items;
    } catch (error) {
      console.error(`Error reading directory: ${folderPath}`, error);
      return [];
    }
  });
  
