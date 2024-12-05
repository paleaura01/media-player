// main.js

const { app, BrowserWindow, ipcMain, dialog } = require("electron");
const { spawn } = require("child_process");
const Speaker = require("speaker");
const path = require("path");
const { v4: uuidv4 } = require("uuid");

let mainWindow;
let speaker = null;
let ffmpegChildProcess = null;
let playbackSessionId = null;
let playbackStartTime = 0;
let trackDuration = 0;
let isStoppingPlayback = false; // Prevent overlapping stops

function stopPlayback(sessionId) {
  if (sessionId && sessionId !== playbackSessionId) {
    console.warn("Ignoring stop request for outdated session:", sessionId);
    return; // Only stop if the session matches the current session
  }

  if (isStoppingPlayback) return;
  isStoppingPlayback = true;

  if (speaker) {
    try {
      console.log("Unpiping and ending speaker...");
      speaker.end();
    } catch (err) {
      console.error("Error ending speaker:", err.message);
    }
    speaker = null;
  }

  if (ffmpegChildProcess) {
    try {
      console.log("Stopping FFmpeg process...");
      if (ffmpegChildProcess.stdout) {
        ffmpegChildProcess.stdout.unpipe(); // Unpipe before killing
      }
      if (typeof ffmpegChildProcess.kill === "function") {
        ffmpegChildProcess.kill("SIGKILL");
        console.log("FFmpeg process killed successfully.");
      }
    } catch (err) {
      console.error("Error killing FFmpeg process:", err.message);
    }
    ffmpegChildProcess = null;
  }

  playbackSessionId = null;
  playbackStartTime = 0;
  trackDuration = 0;
  isStoppingPlayback = false;
}

function startPlayback(filePath, config) {
  const sessionId = uuidv4(); // Unique session ID for this playback
  playbackSessionId = sessionId;

  stopPlayback(); // Stop any previous playback before starting a new one

  try {
    speaker = new Speaker(config);

    // Spawn FFmpeg process
    ffmpegChildProcess = spawn("ffmpeg", [
      "-i", filePath, // Input file
      "-f", "s16le",  // Output format
      "-acodec", "pcm_s16le", // PCM codec
      "-ar", config.sampleRate, // Sample rate
      "-ac", config.channels,   // Channels
      "pipe:1", // Output to pipe
    ]);

    ffmpegChildProcess.stdout.pipe(speaker);

    ffmpegChildProcess.stderr.on("data", (data) => {
      console.error(`FFmpeg stderr: ${data}`);
    });

    ffmpegChildProcess.on("close", (code) => {
      console.log(`FFmpeg process closed with code ${code}`);
      if (playbackSessionId === sessionId) {
        stopPlayback(); // Ensure resources are cleaned up only for the current session
      }
    });

    playbackStartTime = Date.now();
    console.log(`Playback started successfully for: ${filePath}`);
  } catch (err) {
    console.error("Error starting playback:", err.message);
    stopPlayback(sessionId); // Ensure resources are cleaned up if there is an error
  }
}

app.on("ready", () => {
  mainWindow = new BrowserWindow({
    width: 800,
    height: 800,
    webPreferences: {
      preload: path.join(__dirname, "preload.js"),
      contextIsolation: true,
      nodeIntegration: false,
    },
  });

  const startUrl =
    process.env.NODE_ENV === "development"
      ? "http://localhost:3000"
      : `file://${path.join(__dirname, "dist", "index.html")}`;
  mainWindow.loadURL(startUrl);

  if (process.env.NODE_ENV === "development") {
    mainWindow.webContents.openDevTools();
  }

  ipcMain.handle("audio:playTrack", async (event, filePath, config) => {
    const newSessionId = uuidv4();
    playbackSessionId = newSessionId;

    startPlayback(filePath, config);
  });

  ipcMain.handle("audio:stopPlayback", (event, sessionId) => {
    stopPlayback(sessionId);
  });

  ipcMain.handle("audio:getCurrentTime", async () => {
    if (!playbackStartTime || !trackDuration) {
      return { currentTime: 0, duration: trackDuration };
    }

    const currentTime = (Date.now() - playbackStartTime) / 1000;
    return { currentTime: Math.min(currentTime, trackDuration), duration: trackDuration };
  });

  ipcMain.handle("dialog:selectFiles", async () => {
    try {
      const result = await dialog.showOpenDialog(mainWindow, {
        properties: ["openFile", "multiSelections"],
        filters: [
          { name: "Audio Files", extensions: ["mp3", "wav", "ogg", "opus", "flac", "aac", "mp4"] },
        ],
      });
      return result.filePaths || [];
    } catch (error) {
      console.error("Error in dialog:selectFiles handler:", error.message);
      throw error;
    }
  });
});
