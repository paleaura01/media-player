// main.js

const { app, BrowserWindow, ipcMain, dialog } = require('electron');
const path = require('path');

let mainWindow;

app.on('ready', () => {
  mainWindow = new BrowserWindow({
    width: 800,
    height: 600,
    webPreferences: {
      preload: path.join(__dirname, 'preload.js'),
      contextIsolation: true,
      enableRemoteModule: false,
    },
  });

  if (process.env.NODE_ENV === 'development') {
    // Load the Vite dev server during development
    mainWindow.loadURL('http://localhost:3000');
  } else {
    // Load the built renderer files in production
    const indexPath = path.join(__dirname, 'dist', 'index.html');
    mainWindow.loadFile(indexPath);
  }

  mainWindow.webContents.openDevTools(); // Optional: Remove in production
});

// Dialog and file system handlers
ipcMain.handle('dialog:selectFolderOrFiles', async () => {
  const result = await dialog.showOpenDialog({
    properties: ['openFile', 'openDirectory', 'multiSelections'],
    filters: [{ name: 'Audio Files', extensions: ['mp3', 'wav', 'ogg', 'opus'] }],
  });
  return result.canceled ? null : result.filePaths.map((filePath) => ({
    name: path.basename(filePath),
    path: filePath,
  }));
});

ipcMain.handle('readDirectory', async (event, folderPath) => {
  const fs = require('fs');
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
