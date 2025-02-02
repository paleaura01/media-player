// player.cpp (the orchestrator)
#include "player.h"
#include <iostream>

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
    // Place the time bar near the top
    timeBar = { 10, 10, 780, 20 };

    // Place playback controls just below the time bar
    prevButton    = { 10,  40, 40, 40 };
    playButton    = { 60,  40, 40, 40 };
    nextButton    = { 110, 40, 40, 40 };
    stopButton    = { 160, 40, 40, 40 };
    shuffleButton = { 210, 40, 40, 40 };
    muteButton    = { 260, 40, 40, 40 };
    volumeBar     = { 310, 40, 80, 40 };

    // Left panel for playlists
    playlistPanel = { 0, 90, 200, 510 };
    // Right panel for songs
    libraryPanel  = { 200, 90, 600, 510 };

    // "New Playlist" button
    newPlaylistButton = { 
        playlistPanel.x + 10, 
        playlistPanel.y + 10, 
        playlistPanel.w - 20, 
        30 
    };
    mainPanel = { 0, 0, 0, 0 }; // Not used now

    // ====== Set up the "Are you sure?" confirmation dialog ======
    confirmDialogRect = {
        250, // x
        200, // y
        300, // width
        150  // height
    };
    confirmYesButton = {
        confirmDialogRect.x + 30,
        confirmDialogRect.y + confirmDialogRect.h - 50,
        100,
        30
    };
    confirmNoButton = {
        confirmDialogRect.x + confirmDialogRect.w - 130,
        confirmDialogRect.y + confirmDialogRect.h - 50,
        100,
        30
    };
}

Player::~Player() {
    // Save playlists on destruction
    savePlaylistState();
    shutdown();
}

bool Player::init() {
    // Load any saved playlists
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
    
    // Allocate FFmpeg packet/frame
    packet = av_packet_alloc();
    frame  = av_frame_alloc();
    if (!packet || !frame) {
        std::cerr << "Failed to allocate FFmpeg packet or frame." << std::endl;
        return false;
    }
    
    SDL_EventState(SDL_DROPFILE, SDL_ENABLE);
    std::cout << "Initialization successful." << std::endl;
    return true;
}

void Player::update() {
    SDL_Event event;
    while (SDL_PollEvent(&event)) {
        if (event.type == SDL_QUIT) {
            running = false;
        }
        else if (event.type == SDL_MOUSEBUTTONDOWN) {
            int x = event.button.x;
            int y = event.button.y;
            handleMouseClick(x, y); 
        }
        // ========== TEXT INPUT FOR RENAME MODE ==========
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
                    // Commit
                    if (renameIndex >= 0 && renameIndex < (int)playlists.size()) {
                        playlists[renameIndex].name = renameBuffer;
                    }
                    isRenaming = false;
                    renameIndex = -1;
                    renameBuffer.clear();
                    SDL_StopTextInput();
                }
                else if (event.key.keysym.sym == SDLK_ESCAPE) {
                    // Cancel
                    isRenaming = false;
                    renameIndex = -1;
                    renameBuffer.clear();
                    SDL_StopTextInput();
                }
            }
        }
        // ========== FILE DROPS / DRAG AND DROP ==========
        else if (event.type == SDL_DROPFILE) {
            char* filePath = event.drop.file;
            handleFileDrop(filePath);
            SDL_free(filePath);
        }
    }

    // Update currentTime if playing
    if (playingAudio && fmtCtx && fmtCtx->bit_rate > 0) {
        currentTime = static_cast<double>(fmtCtx->pb->pos) / (fmtCtx->bit_rate / 8);
    }
    
    // Clear screen
    SDL_SetRenderDrawColor(renderer, 30, 30, 30, 255);
    SDL_RenderClear(renderer);

    // Draw UI
    drawPlaylistPanel();  // left pane
    drawSongPanel();      // right pane
    drawControls();       // buttons
    drawTimeBar();        // track time bar

    // Finally, draw the confirmation dialog if needed (on top)
    drawConfirmDialog();

    SDL_RenderPresent(renderer);
    SDL_Delay(16);
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
    std::cout << "Player shutdown." << std::endl;
    running = false;
}

bool Player::isRunning() const {
    return running;
}
