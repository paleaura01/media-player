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

      ffmpegChildProcess.stdout.on("data", () => {
        if (!isPlaying) {
          playbackStartTime = Date.now();
          isPlaying = true;
          console.log(`[PLAYBACK] Audio stream started for: ${filePath}`);
        }
      });

      ffmpegChildProcess.stderr.on("data", (data) => {
        console.error(`[FFmpeg STDERR] ${data}`);
      });

      ffmpegChildProcess.on("close", (code) => {
        console.log(`[FFmpeg] Process closed with code ${code}`);
        stopPlayback();
      });

      console.log(`[PLAYBACK] Playback initialization completed.`);
    } catch (err) {
      console.error(`[PLAYBACK] Error starting playback: ${err.message}`);
      stopPlayback();
    }
  });
}

// Handle playTrack request
ipcMain.handle("audio:playTrack", async (event, filePath, config) => {
  const ffmpegProcess = spawn("ffmpeg", ["-i", filePath, "-f", "null", "-"]);
  ffmpegProcess.stderr.on("data", (data) => {
    const output = data.toString();
    const durationMatch = output.match(/Duration: (\d+):(\d+):(\d+\.\d+)/);
    if (durationMatch) {
      const [, hours, minutes, seconds] = durationMatch;
      trackDuration =
        parseInt(hours) * 3600 + parseInt(minutes) * 60 + parseFloat(seconds);
      console.log(`[DURATION] Track duration: ${trackDuration} seconds`);
    }
  });

  ffmpegProcess.on("close", (code) => {
    console.log("[DURATION] FFmpeg duration extraction complete.");
    startPlayback(filePath, config);
  });
});

// Handle stopPlayback request
ipcMain.handle("audio:stopPlayback", async () => {
  console.log("[REQUEST] Stop playback request received.");
  await stopPlayback();
});

// Handle getCurrentTime request
ipcMain.handle("audio:getCurrentTime", async () => {
  if (!isPlaying || !playbackStartTime || !trackDuration) {
    console.warn("[GET TIME] Playback not started or track duration unavailable.");
    return { currentTime: 0, duration: trackDuration || 0 };
  }

  const elapsedTime = (Date.now() - playbackStartTime) / 1000;
  console.log(`[GET TIME] Elapsed time: ${elapsedTime.toFixed(2)} seconds, Duration: ${trackDuration} seconds.`);
  return {
    currentTime: Math.min(elapsedTime, trackDuration),
    duration: trackDuration,
  };
});

// Electron app setup
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

  const startUrl = `file://${path.join(__dirname, "renderer/index.html")}`;
  mainWindow.loadURL(startUrl);

  mainWindow.webContents.openDevTools();
});

