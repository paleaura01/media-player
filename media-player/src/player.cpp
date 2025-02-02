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
        else if (event.type == SDL_TEXTINPUT) {
            if (isRenaming) {
                renameBuffer += event.text.text;
            }
        }
        else if (event.type == SDL_KEYDOWN) {
            if (isRenaming) {
                if (event.key.keysym.sym == SDLK_BACKSPACE) {
                    if (!renameBuffer.empty()) {
                        renameBuffer.pop_back();
                    }
                }
                else if (event.key.keysym.sym == SDLK_RETURN) {
                    if (renameIndex >= 0 && renameIndex < (int)playlists.size()) {
                        playlists[renameIndex].name = renameBuffer;
                    }
                    isRenaming = false;
                    renameIndex = -1;
                    renameBuffer.clear();
                    SDL_StopTextInput();
                }
                else if (event.key.keysym.sym == SDLK_ESCAPE) {
                    isRenaming = false;
                    renameIndex = -1;
                    renameBuffer.clear();
                    SDL_StopTextInput();
                }
            }
        }
        else if (event.type == SDL_DROPFILE) {
            char* filePath = event.drop.file;
            handleFileDrop(filePath);
            SDL_free(filePath);
        }
        int mouseX, mouseY;
SDL_GetMouseState(&mouseX, &mouseY);
hoveredSongIndex = -1;
for (size_t i = 0; i < songRects.size(); i++) {
    if (mouseX >= songRects[i].x && mouseX <= songRects[i].x + songRects[i].w &&
        mouseY >= songRects[i].y && mouseY <= songRects[i].y + songRects[i].h) {
        hoveredSongIndex = i;
        break;
    }
}
    }

    // Update currentTime from lastPTS
    if (playingAudio) {
        currentTime = lastPTS.load(std::memory_order_relaxed);
    }

    // Clear
    SDL_SetRenderDrawColor(renderer, 30, 30, 30, 255);
    SDL_RenderClear(renderer);

    // Draw
    drawPlaylistPanel();
    drawSongPanel();
    drawControls();
    drawTimeBar();
    drawConfirmDialog();

    SDL_RenderPresent(renderer);
    SDL_Delay(16);
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
