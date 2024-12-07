// main.js

const { app, BrowserWindow, ipcMain } = require("electron");
const { spawn } = require("child_process");
const Speaker = require("speaker");
const path = require("path");
const ffmpeg = require("fluent-ffmpeg");
const playbackSessions = new Map();

let mainWindow;
let speaker = null;
let ffmpegChildProcess = null;
let playbackStartTime = 0;
let trackDuration = 0;
let isPlaying = false;
let activeSessionId = null;

// Stop playback function
async function stopPlayback(requestedSessionId = null) {
  if (requestedSessionId && requestedSessionId !== activeSessionId) {
    console.log(`[STOP] Ignoring stop request for session ID: ${requestedSessionId}`);
    return;
  }

  if (activeSessionId && playbackSessions.has(activeSessionId)) {
    const { speaker, ffmpegChildProcess } = playbackSessions.get(activeSessionId);

    if (speaker) {
      try {
        speaker.end();
        console.log(`[STOP] Speaker stopped for session: ${activeSessionId}`);
      } catch (err) {
        console.error(`[STOP] Error stopping speaker for session ${activeSessionId}:`, err.message);
      }
    }

    if (ffmpegChildProcess) {
      try {
        if (ffmpegChildProcess.stdout) ffmpegChildProcess.stdout.unpipe();
        ffmpegChildProcess.kill("SIGKILL");
        console.log(`[STOP] FFmpeg process killed for session: ${activeSessionId}`);
      } catch (err) {
        console.error(`[STOP] Error killing FFmpeg process for session ${activeSessionId}:`, err.message);
      }
    }

    playbackSessions.delete(activeSessionId); // Clean up session resources
  }

  activeSessionId = null;
  playbackStartTime = 0;
  trackDuration = 0;
  isPlaying = false;

  console.log("[STOP] Playback stopped successfully.");
}

// Fetch metadata for track duration using FFmpeg
async function getTrackDuration(filePath) {
  return new Promise((resolve, reject) => {
    ffmpeg.ffprobe(filePath, (err, metadata) => {
      if (err) {
        console.error(`[FFmpeg Metadata] Error: ${err.message}`);
        return reject(err);
      }
      const duration = metadata.format.duration || 0;
      console.log(`[FFmpeg Metadata] Duration for "${filePath}": ${duration}s`);
      resolve(duration);
    });
  });
}

// Handle playTrack request
ipcMain.handle("audio:playTrack", async (event, filePath, config, sessionId) => {
  if (activeSessionId && activeSessionId !== sessionId) {
    console.warn(`[PLAYBACK] Ignoring playback request for session ID: ${sessionId} (Active: ${activeSessionId})`);
    return { success: false, trackDuration: 0 };
  }

  try {
    // Ensure the previous playback is fully stopped before starting a new one
    await stopPlayback(activeSessionId);
    activeSessionId = sessionId;

    console.log(`[PLAYBACK] Starting playback for: ${filePath} (Session ID: ${sessionId})`);
    const speaker = new Speaker(config);

    const ffmpegChildProcess = spawn("ffmpeg", [
      "-i", filePath,
      "-f", "s16le",
      "-acodec", "pcm_s16le",
      "-ar", config.sampleRate,
      "-ac", config.channels,
      "pipe:1",
    ]);

    ffmpegChildProcess.stdout.pipe(speaker);

    playbackSessions.set(sessionId, { speaker, ffmpegChildProcess });

    playbackStartTime = Date.now();
    trackDuration = await getTrackDuration(filePath); // Set track duration
    isPlaying = true;

    ffmpegChildProcess.on("close", (code) => {
      console.log(`[FFmpeg] Process closed for session: ${sessionId} with code ${code}`);
      if (sessionId === activeSessionId) {
        stopPlayback(sessionId);
      }
    });

    return { success: true, trackDuration }; // Return the duration to the renderer
  } catch (error) {
    console.error(`[PLAYBACK] Error starting playback for session ${sessionId}: ${error.message}`);
    return { success: false, trackDuration: 0 };
  }
});

// Handle stopPlayback request
ipcMain.handle("audio:stopPlayback", async (event, sessionId) => {
  console.log(`[REQUEST] Stop playback request received for session: ${sessionId}`);
  await stopPlayback(sessionId);
});

// Handle getCurrentTime request
ipcMain.handle("audio:getCurrentTime", async () => {
  try {
    if (!isPlaying || playbackStartTime === 0) {
      logWithTimestampMain("[PROGRESS] No active playback session.");
      return { currentTime: 0, duration: trackDuration };
    }
    const currentTime = (Date.now() - playbackStartTime) / 1000; // Calculate elapsed time
    logWithTimestampMain(
      `[PROGRESS] Current playback time: ${currentTime.toFixed(2)}s / ${trackDuration.toFixed(2)}s`
    );
    return { currentTime, duration: trackDuration };
  } catch (error) {
    logWithTimestampMain(`[ERROR] Error in getCurrentTime: ${error.message}`);
    return { currentTime: 0, duration: 0 };
  }
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

// Inline timestamped logging for the main process
function logWithTimestampMain(message) {
  const timestamp = new Date().toISOString();
  console.log(`[MAIN] [${timestamp}] ${message}`);
}
