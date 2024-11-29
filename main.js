// main.js

const { app, BrowserWindow, ipcMain, dialog, protocol } = require('electron');
const path = require('path');
const fs = require('fs');

const isDev = process.env.NODE_ENV === 'development';

let mainWindow;

app.on('ready', () => {
  protocol.registerFileProtocol('local', (request, callback) => {
    const url = request.url.replace(/^local:\/\//, '');
    callback({ path: decodeURIComponent(url) });
  });

  mainWindow = new BrowserWindow({
    width: 800,
    height: 800,
    webPreferences: {
      preload: path.join(__dirname, 'preload.js'),
      contextIsolation: true,
      nodeIntegration: false,
    },
  });

  if (isDev) {
    mainWindow.loadURL('http://localhost:3000'); // Use Vite dev server
  } else {
    mainWindow.loadFile(path.join(__dirname, 'dist/index.html'));
  }

  if (isDev) {
    mainWindow.webContents.openDevTools(); // Enable DevTools in development
  }
});

// Dialog for selecting files
ipcMain.handle("dialog:selectFiles", async () => {
  const result = await dialog.showOpenDialog({
    properties: ["openFile", "multiSelections"], // File selection only
    filters: [{ name: "Audio Files", extensions: ["mp3", "wav", "ogg", "opus"] }],
  });
  return result.canceled ? null : result.filePaths;
});

// Dialog for selecting folders or files
ipcMain.handle("dialog:selectFolderOrFiles", async () => {
  const result = await dialog.showOpenDialog({
    properties: ["openFile", "openDirectory", "multiSelections"],
  });
  return result.canceled ? null : result.filePaths;
});

// Check if file exists
ipcMain.handle("fileExists", async (event, filePath) => {
  try {
    return fs.existsSync(filePath); // Returns true if the file exists, false otherwise
  } catch (error) {
    console.error(`Error checking if file exists: ${filePath}`, error);
    return false;
  }
});

// Read directory contents
ipcMain.handle('readDirectory', async (event, folderPath) => {
  try {
    return fs.readdirSync(folderPath).map((fileName) => {
      const filePath = path.join(folderPath, fileName);
      const stats = fs.statSync(filePath);
      return {
        name: fileName,
        path: filePath,
        type: stats.isDirectory() ? 'directory' : 'file',
      };
    });
  } catch (error) {
    console.error(`Error reading directory: ${folderPath}`, error);
    return [];
  }
});
