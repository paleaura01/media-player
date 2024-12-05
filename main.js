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

function isSpeakerSilent(speaker) {
  return new Promise((resolve) => {
    const bufferCheckInterval = 100; // Interval to check for silence (ms)
    const silenceThreshold = 0.001; // Threshold to consider silence
    let silent = true;

    const silenceChecker = setInterval(() => {
      if (!speaker || !speaker._writableState || !speaker._writableState.bufferedRequest) {
        clearInterval(silenceChecker);
        resolve(true); // No active audio stream means silent
        return;
      }

      // Check if the speaker buffer is under the threshold
      if (speaker._writableState.bufferedRequestSize > silenceThreshold) {
        silent = false;
      }

      if (silent) {
        clearInterval(silenceChecker);
        resolve(true);
      }
    }, bufferCheckInterval);
  });
}

async function stopPlayback() {
  if (speaker) {
    try {
      console.log("Stopping speaker...");
      speaker.end();
      speaker = null;
    } catch (err) {
      console.error("Error stopping speaker:", err.message);
    }
  }

  if (ffmpegChildProcess) {
    try {
      if (ffmpegChildProcess.stdout) {
        ffmpegChildProcess.stdout.unpipe();
      }
      ffmpegChildProcess.kill("SIGKILL");
      ffmpegChildProcess = null;
      console.log("FFmpeg process killed successfully.");
    } catch (err) {
      console.error("Error killing FFmpeg process:", err.message);
    }
  }
}

function startPlayback(filePath, config) {
  const sessionId = uuidv4();
  playbackSessionId = sessionId;

  stopPlayback().then(async () => {
    // Wait for silence before starting the next track
    const silent = await isSpeakerSilent(speaker);
    if (!silent) {
      console.log("Speaker is not yet silent, delaying playback...");
      return;
    }

    try {
      speaker = new Speaker(config);

      ffmpegChildProcess = spawn("ffmpeg", [
        "-i", filePath,
        "-f", "s16le",
        "-acodec", "pcm_s16le",
        "-ar", config.sampleRate,
        "-ac", config.channels,
        "pipe:1",
      ]);

      ffmpegChildProcess.stdout.pipe(speaker);

      ffmpegChildProcess.stderr.on("data", (data) => {
        console.error(`FFmpeg stderr: ${data}`);
      });

      ffmpegChildProcess.on("close", (code) => {
        console.log(`FFmpeg process closed with code ${code}`);
        if (playbackSessionId === sessionId) {
          stopPlayback();
        }
      });

      playbackStartTime = Date.now();
      console.log(`Playback started successfully for: ${filePath}`);
    } catch (err) {
      console.error("Error starting playback:", err.message);
      stopPlayback();
    }
  });
}

ipcMain.handle("audio:playTrack", async (event, filePath, config) => {
  startPlayback(filePath, config);
});

ipcMain.handle("audio:stopPlayback", async () => {
  await stopPlayback();
});

ipcMain.handle("audio:getCurrentTime", async () => {
  if (!playbackStartTime || !trackDuration) {
    return { currentTime: 0, duration: trackDuration };
  }

  const currentTime = (Date.now() - playbackStartTime) / 2000;
  return { currentTime: Math.min(currentTime, trackDuration), duration: trackDuration };
});

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
