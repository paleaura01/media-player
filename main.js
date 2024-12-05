// main.js

const { app, BrowserWindow, ipcMain } = require("electron");
const { spawn } = require("child_process");
const Speaker = require("speaker");
const path = require("path");

let mainWindow;
let speaker = null;
let ffmpegChildProcess = null;
let playbackStartTime = 0;
let trackDuration = 0;
let isPlaying = false;

// Track playback progress
let currentPlaybackTime = 0;

// Stop playback function
async function stopPlayback() {
  if (speaker) {
    try {
      speaker.end();
      speaker = null;
      console.log("[STOP] Speaker stopped.");
    } catch (err) {
      console.error("[STOP] Error stopping speaker:", err.message);
    }
  }

  if (ffmpegChildProcess) {
    try {
      if (ffmpegChildProcess.stdout) ffmpegChildProcess.stdout.unpipe();
      ffmpegChildProcess.kill("SIGKILL");
      ffmpegChildProcess = null;
      console.log("[STOP] FFmpeg process killed.");
    } catch (err) {
      console.error("[STOP] Error killing FFmpeg process:", err.message);
    }
  }

  playbackStartTime = 0;
  trackDuration = 0;
  currentPlaybackTime = 0;
  isPlaying = false;
}

// Start playback function
function startPlayback(filePath, config) {
  console.log(`[PLAYBACK] Starting playback for file: ${filePath}`);

  stopPlayback().then(() => {
    try {
      speaker = new Speaker(config);

      ffmpegChildProcess = spawn("ffmpeg", [
        "-i",
        filePath,
        "-f",
        "s16le",
        "-acodec",
        "pcm_s16le",
        "-ar",
        config.sampleRate,
        "-ac",
        config.channels,
        "pipe:1",
      ]);

      ffmpegChildProcess.stdout.pipe(speaker);

      // Listen to FFmpeg logs for playback progress
      ffmpegChildProcess.stderr.on("data", (data) => {
        const output = data.toString();

        // Extract duration and playback time
        const timeMatch = output.match(/time=(\d+):(\d+):(\d+\.\d+)/);
        if (timeMatch) {
          const [, hours, minutes, seconds] = timeMatch;
          currentPlaybackTime =
            parseInt(hours) * 3600 + parseInt(minutes) * 60 + parseFloat(seconds);

          if (!isPlaying) {
            playbackStartTime = Date.now();
            isPlaying = true;
            console.log("[PLAYBACK] Audio stream started.");
          }

          // Send current playback time to renderer
          mainWindow.webContents.send("audio:updateProgress", {
            currentTime: currentPlaybackTime,
            duration: trackDuration,
          });
        }

        // Extract track duration
        const durationMatch = output.match(/Duration: (\d+):(\d+):(\d+\.\d+)/);
        if (durationMatch) {
          const [, hours, minutes, seconds] = durationMatch;
          trackDuration =
            parseInt(hours) * 3600 + parseInt(minutes) * 60 + parseFloat(seconds);
          console.log(`[DURATION] Track duration: ${trackDuration} seconds`);
        }
      });

      ffmpegChildProcess.on("close", (code) => {
        console.log(`[FFmpeg] Process closed with code ${code}`);
        stopPlayback();
      });

      console.log("[PLAYBACK] Playback initialization completed.");
    } catch (err) {
      console.error(`[PLAYBACK] Error starting playback: ${err.message}`);
      stopPlayback();
    }
  });
}

// Handle playTrack request
ipcMain.handle("audio:playTrack", async (event, filePath, config) => {
  startPlayback(filePath, config);
});

// Handle stopPlayback request
ipcMain.handle("audio:stopPlayback", async () => {
  console.log("[REQUEST] Stop playback request received.");
  await stopPlayback();
});

// Handle getCurrentTime request
ipcMain.handle("audio:getCurrentTime", async () => {
  return {
    currentTime: currentPlaybackTime,
    duration: trackDuration,
  };
});

// Electron app setup
app.on("ready", () => {
  mainWindow = new BrowserWindow({
    width: 800,
    height: 800,
    webPreferences: {
      preload: path.join(__dirname, "preload.js"), // Ensure the correct preload script path
      contextIsolation: true, // Keep isolation enabled
      nodeIntegration: false, // Ensure Node.js is not directly accessible in the renderer
    },
  });
  

  const startUrl = `file://${path.join(__dirname, "renderer/index.html")}`;
  mainWindow.loadURL(startUrl);

  mainWindow.webContents.openDevTools();
});
