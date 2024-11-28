// main.js
const { app, BrowserWindow, ipcMain, dialog } = require("electron");
const path = require("path");

let mainWindow;

app.on("ready", () => {
  mainWindow = new BrowserWindow({
    width: 800,
    height: 600,
    webPreferences: {
      preload: path.join(__dirname, "preload.js"),
      contextIsolation: true,   // Isolate context for security
      nodeIntegration: false,   // Do not enable Node.js integration in renderer
      sandbox: false,           // Disable sandbox to allow Node.js in preload
    },
  });

  mainWindow.loadFile("index.html");
  mainWindow.webContents.openDevTools(); // Open DevTools for debugging
});

app.on("window-all-closed", () => {
  if (process.platform !== "darwin") {
    app.quit();
  }
});

// Handle folder selection
ipcMain.handle("dialog:selectFolder", async () => {
  const result = await dialog.showOpenDialog({
    properties: ["openDirectory"],
  });
  return result.canceled ? null : result.filePaths[0];
});