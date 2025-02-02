// player.cpp
#include "player.h"
#include <iostream>
#include <cmath> // llround
#include <cstdlib> // for srand, rand

Player::Player()
    : running(true), window(nullptr), renderer(nullptr), font(nullptr),
      fmtCtx(nullptr), codecCtx(nullptr), swrCtx(nullptr),
      audioStreamIndex(-1), packet(nullptr), frame(nullptr),
      audioBuffer(nullptr), audioBufferSize(0), audioBufferIndex(0),
      audioDev(0), playingAudio(false), loadedFile(""),
      currentTime(0), totalDuration(0), isMuted(false), isShuffled(false),
      activePlaylist(-1),
      isConfirmingDeletion(false), deleteCandidateIndex(-1)
{
    timeBar = { 10, 10, 780, 20 };
    // Transport buttons
    prevButton    = { 10,  40, 40, 40 };
    playButton    = { 55,  40, 80, 40 };
    nextButton    = { 140, 40, 40, 40 };
    shuffleButton = { 235, 40, 80, 40 };
    muteButton    = { 320, 40, 80, 40 };
    rewindButton  = { 405, 40, 40, 40 };
    forwardButton = { 450, 40, 40, 40 };
    volumeBar     = { 545, 40, 240, 40 };

    // Panels
    playlistPanel = { 0,  90, 200, 510 };
    libraryPanel  = { 200, 90, 600, 510 };
    newPlaylistButton = { 
        playlistPanel.x + 10,
        playlistPanel.y + 10,
        playlistPanel.w - 20,
        30
    };
    mainPanel = {0,0,0,0};

    // Confirm Deletion
    confirmDialogRect = { 250,200,300,150 };
    confirmYesButton  = { 280,300,100,30 };
    confirmNoButton   = { 420,300,100,30 };
}

Player::~Player() {
    savePlaylistState();
    shutdown();
}

bool Player::init() {
    loadPlaylistState();

    if (SDL_Init(SDL_INIT_VIDEO | SDL_INIT_AUDIO) != 0) {
        std::cerr << "SDL_Init Error: " << SDL_GetError() << std::endl;
        return false;
    }
    if (TTF_Init() != 0) {
        std::cerr << "TTF_Init Error: " << TTF_GetError() << std::endl;
        return false;
    }

    window = SDL_CreateWindow("Barebones Audio Player",
                              SDL_WINDOWPOS_CENTERED, SDL_WINDOWPOS_CENTERED,
                              800, 600, SDL_WINDOW_SHOWN);
    if (!window) {
        std::cerr << "SDL_CreateWindow Error: " << SDL_GetError() << std::endl;
        return false;
    }
    renderer = SDL_CreateRenderer(window, -1, SDL_RENDERER_ACCELERATED);
    if (!renderer) {
        std::cerr << "SDL_CreateRenderer Error: " << SDL_GetError() << std::endl;
        return false;
    }
    font = TTF_OpenFont("Arial.ttf", 24);
    if (!font) {
        std::cerr << "TTF_OpenFont Error: " << TTF_GetError() << std::endl;
        return false;
    }

    packet = av_packet_alloc();
    frame  = av_frame_alloc();
    if (!packet || !frame) {
        std::cerr << "Failed to allocate FFmpeg packet/frame." << std::endl;
        return false;
    }

    SDL_EventState(SDL_DROPFILE, SDL_ENABLE);
    std::cout << "Initialization successful.\n";

     // Optionally seed for random generator once
    srand(static_cast<unsigned>(SDL_GetTicks())); 
    return true;

}

void Player::update() {
    SDL_Event event;
    while (SDL_PollEvent(&event)) {
        if (event.type == SDL_QUIT) {
            running = false;
        }
        else if (event.type == SDL_MOUSEBUTTONDOWN) {
            handleMouseClick(event.button.x, event.button.y);
        }
    }

    // Update current time
    if (playingAudio) {
        currentTime = lastPTS.load(std::memory_order_relaxed);

        // === Detect end of song ===
if ((currentTime >= totalDuration && totalDuration > 0) || reachedEOF.load(std::memory_order_relaxed)) {
    std::cout << "[Debug] Track finished. Moving to next song...\n";
    reachedEOF.store(false, std::memory_order_relaxed);  // Reset the flag
    playNextTrack();
}
    }

    // UI Rendering
    SDL_SetRenderDrawColor(renderer, 30, 30, 30, 255);
    SDL_RenderClear(renderer);

    drawPlaylistPanel();
    drawSongPanel();
    drawControls();
    drawTimeBar();
    drawConfirmDialog();

    SDL_RenderPresent(renderer);
    SDL_Delay(16);
}

void Player::playNextTrack() {
    if (activePlaylist < 0 || playlists[activePlaylist].songs.empty()) return;

    const auto& songs = playlists[activePlaylist].songs;

    if (isShuffled) {
        // Shuffle mode: pick a random song (not the same one)
        if (songs.size() == 1) return; // Only one track, just replay it

        int currentIndex = -1;
        for (size_t i = 0; i < songs.size(); i++) {
            if (songs[i] == loadedFile) {
                currentIndex = (int)i;
                break;
            }
        }

        int randomIndex;
        do {
            randomIndex = rand() % songs.size();
        } while (randomIndex == currentIndex);

        if (loadAudioFile(songs[randomIndex])) {
            playAudio();
        }
    }
    else {
        // Normal mode: Go to the next track or loop back
        for (size_t i = 0; i < songs.size(); i++) {
            if (songs[i] == loadedFile) {
                if (i < songs.size() - 1) {
                    if (loadAudioFile(songs[i + 1])) {
                        playAudio();
                    }
                } else {
                    // Loop back to the first song if at the end
                    if (loadAudioFile(songs[0])) {
                        playAudio();
                    }
                }
                return;
            }
        }
    }
}



void Player::seekTo(double seconds) {
    if (!fmtCtx || audioStreamIndex < 0) return;

    int64_t target = (int64_t)llround(seconds * AV_TIME_BASE);
    if (av_seek_frame(fmtCtx, -1, target, AVSEEK_FLAG_BACKWARD) >= 0) {
        avcodec_flush_buffers(codecCtx);
        currentTime = seconds;
        lastPTS.store(seconds, std::memory_order_relaxed);
        std::cout << "[Debug] Seeked to " << seconds << " sec.\n";
    } else {
        std::cerr << "[Warn] av_seek_frame failed.\n";
    }
}

void Player::shutdown() {
    if (audioDev != 0) {
        SDL_CloseAudioDevice(audioDev);
        audioDev = 0;
    }
    if (audioBuffer) {
        av_free(audioBuffer);
        audioBuffer = nullptr;
    }
    if (swrCtx) {
        swr_free(&swrCtx);
        swrCtx = nullptr;
    }
    if (codecCtx) {
        avcodec_free_context(&codecCtx);
        codecCtx = nullptr;
    }
    if (fmtCtx) {
        avformat_close_input(&fmtCtx);
        fmtCtx = nullptr;
    }
    if (packet) {
        av_packet_free(&packet);
        packet = nullptr;
    }
    if (frame) {
        av_frame_free(&frame);
        frame = nullptr;
    }
    if (font) {
        TTF_CloseFont(font);
        font = nullptr;
    }
    if (renderer) {
        SDL_DestroyRenderer(renderer);
        renderer = nullptr;
    }
    if (window) {
        SDL_DestroyWindow(window);
        window = nullptr;
    }
    SDL_Quit();
    std::cout << "Player shutdown.\n";
    running = false;
}

bool Player::isRunning() const {
    return running;
}
