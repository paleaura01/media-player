//player.cpp
// player.cpp (the orchestrator)
#include "player.h"
#include <iostream>

// Constructor: sets up basic defaults, UI rectangles, etc.
Player::Player()
    : running(true), window(nullptr), renderer(nullptr), font(nullptr),
      fmtCtx(nullptr), codecCtx(nullptr), swrCtx(nullptr),
      audioStreamIndex(-1), packet(nullptr), frame(nullptr),
      audioBuffer(nullptr), audioBufferSize(0), audioBufferIndex(0),
      audioDev(0), playingAudio(false), loadedFile(""),
      currentTime(0), totalDuration(0), isMuted(false), isShuffled(false),
      activePlaylist(-1)
{
    // Main window sections
    playlistPanel = { 0, 100, 200, 500 };
    mainPanel     = { 200, 100, 600, 500 };
    
    // Time bar
    timeBar = { 210, 20, 540, 20 };
    
    // Control buttons
    prevButton    = { 210, 50, 40, 40 };
    playButton    = { 260, 50, 40, 40 };
    nextButton    = { 310, 50, 40, 40 };
    stopButton    = { 360, 50, 40, 40 };
    shuffleButton = { 410, 50, 40, 40 };
    muteButton    = { 460, 50, 40, 40 };
    volumeBar     = { 510, 50, 80, 40 };
    
    newPlaylistButton = { 10, 60, 180, 30 };
}

Player::~Player() {
    // Save playlists on destruction to ensure persistence
    savePlaylistState();
    shutdown();
}

bool Player::init() {
    // Load any saved playlists from disk first
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
            // On the X (close), we can also call savePlaylistState() explicitly if you want
            // but we already do it in the destructor. 
            running = false;
        }
        else if (event.type == SDL_MOUSEBUTTONDOWN) {
            // ... handle UI events (like new playlist button, play, stop, etc.)
            // but we keep that logic in player_ui.cpp to reduce clutter.
            // We'll just forward the event or call a helper.  
            int x = event.button.x;
            int y = event.button.y;
            // We'll define a helper in player_ui.cpp that handles these clicks:
            handleMouseClick(x, y); // We'll define 'handleMouseClick()' in player_ui.cpp
        }
        else if (event.type == SDL_DROPFILE) {
            char* filePath = event.drop.file;
            // Also forward to a helper if you'd like
            handleFileDrop(filePath); // We'll define in player_playlists.cpp
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
    
    // Draw UI (calls out to player_ui.cpp)
    drawPlaylistPanel();  
    drawControls();       
    drawTimeBar();        

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
