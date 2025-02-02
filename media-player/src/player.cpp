//player.cpp
#include "player.h"
#include <iostream>

// Constructor: Initialize member variables and define button positions.
Player::Player() 
    : running(true), window(nullptr), renderer(nullptr), font(nullptr), music(nullptr) {
    // Set Play and Stop button positions and sizes.
    playButton = { 50, 500, 150, 50 };
    stopButton = { 250, 500, 150, 50 };
}

// Destructor calls shutdown to clean up.
Player::~Player() {
    shutdown();
}

// Initialize SDL2 (video and audio), SDL_ttf, SDL_mixer, create window/renderer, and load a font.
bool Player::init() {
    if (SDL_Init(SDL_INIT_VIDEO | SDL_INIT_AUDIO) != 0) {
        std::cerr << "SDL_Init Error: " << SDL_GetError() << std::endl;
        return false;
    }
    if (TTF_Init() != 0) {
        std::cerr << "TTF_Init Error: " << TTF_GetError() << std::endl;
        return false;
    }
    if (Mix_OpenAudio(44100, MIX_DEFAULT_FORMAT, 2, 2048) < 0) {
        std::cerr << "Mix_OpenAudio Error: " << Mix_GetError() << std::endl;
        return false;
    }
    window = SDL_CreateWindow("Barebones Media Player", SDL_WINDOWPOS_CENTERED, SDL_WINDOWPOS_CENTERED, 800, 600, SDL_WINDOW_SHOWN);
    if (!window) {
        std::cerr << "SDL_CreateWindow Error: " << SDL_GetError() << std::endl;
        return false;
    }
    renderer = SDL_CreateRenderer(window, -1, SDL_RENDERER_ACCELERATED);
    if (!renderer) {
        std::cerr << "SDL_CreateRenderer Error: " << SDL_GetError() << std::endl;
        return false;
    }
    // Load font (ensure Arial.ttf is in the executable directory or change filename accordingly)
    font = TTF_OpenFont("Arial.ttf", 24);
    if (!font) {
        std::cerr << "TTF_OpenFont Error: " << TTF_GetError() << std::endl;
        return false;
    }
    // Enable file drop events for loading audio files.
    SDL_EventState(SDL_DROPFILE, SDL_ENABLE);

    std::cout << "Initialization successful." << std::endl;
    return true;
}

// Helper function to render text to a texture.
SDL_Texture* Player::renderText(const std::string &text, SDL_Color color) {
    SDL_Surface* surface = TTF_RenderText_Solid(font, text.c_str(), color);
    if (!surface) {
        std::cerr << "TTF_RenderText_Solid Error: " << TTF_GetError() << std::endl;
        return nullptr;
    }
    SDL_Texture* texture = SDL_CreateTextureFromSurface(renderer, surface);
    SDL_FreeSurface(surface);
    return texture;
}

// Process events, handle file drops, and button clicks; render the GUI.
void Player::update() {
    SDL_Event event;
    while (SDL_PollEvent(&event)) {
        if (event.type == SDL_QUIT) {
            running = false;
        }
        else if (event.type == SDL_MOUSEBUTTONDOWN) {
            int x = event.button.x;
            int y = event.button.y;
            // If play button is clicked
            if (x >= playButton.x && x <= playButton.x + playButton.w &&
                y >= playButton.y && y <= playButton.y + playButton.h) {
                std::cout << "Play button pressed." << std::endl;
                if (music) {
                    Mix_PlayMusic(music, 1); // play once
                } else {
                    std::cout << "No audio loaded!" << std::endl;
                }
            }
            // If stop button is clicked
            if (x >= stopButton.x && x <= stopButton.x + stopButton.w &&
                y >= stopButton.y && y <= stopButton.y + stopButton.h) {
                std::cout << "Stop button pressed." << std::endl;
                Mix_HaltMusic();
            }
        }
        else if (event.type == SDL_DROPFILE) {
            char* filePath = event.drop.file;
            std::cout << "File dropped: " << filePath << std::endl;
            // Free any previously loaded music.
            if (music) {
                Mix_FreeMusic(music);
                music = nullptr;
            }
            // Attempt to load the dropped file as music.
            music = Mix_LoadMUS(filePath);
            if (!music) {
                std::cerr << "Failed to load music: " << Mix_GetError() << std::endl;
            } else {
                loadedFile = filePath;
                std::cout << "Loaded audio file: " << loadedFile << std::endl;
            }
            SDL_free(filePath);
        }
    }

    // Render background.
    SDL_SetRenderDrawColor(renderer, 50, 50, 50, 255);
    SDL_RenderClear(renderer);

    // Draw the play button (green).
    SDL_SetRenderDrawColor(renderer, 0, 200, 0, 255);
    SDL_RenderFillRect(renderer, &playButton);

    // Draw the stop button (red).
    SDL_SetRenderDrawColor(renderer, 200, 0, 0, 255);
    SDL_RenderFillRect(renderer, &stopButton);

    // Render button labels.
    SDL_Color white = {255, 255, 255, 255};
    SDL_Texture* playLabel = renderText("Play", white);
    SDL_Texture* stopLabel = renderText("Stop", white);
    if (playLabel) {
        int w, h;
        SDL_QueryTexture(playLabel, nullptr, nullptr, &w, &h);
        SDL_Rect dest = { playButton.x + (playButton.w - w) / 2, playButton.y + (playButton.h - h) / 2, w, h };
        SDL_RenderCopy(renderer, playLabel, nullptr, &dest);
        SDL_DestroyTexture(playLabel);
    }
    if (stopLabel) {
        int w, h;
        SDL_QueryTexture(stopLabel, nullptr, nullptr, &w, &h);
        SDL_Rect dest = { stopButton.x + (stopButton.w - w) / 2, stopButton.y + (stopButton.h - h) / 2, w, h };
        SDL_RenderCopy(renderer, stopLabel, nullptr, &dest);
        SDL_DestroyTexture(stopLabel);
    }

    // Render status text: show loaded file or prompt.
    std::string status = loadedFile.empty() ? "Drop an audio file to load." : "Loaded: " + loadedFile;
    SDL_Texture* statusLabel = renderText(status, white);
    if (statusLabel) {
        int w, h;
        SDL_QueryTexture(statusLabel, nullptr, nullptr, &w, &h);
        SDL_Rect dest = { 50, 50, w, h };
        SDL_RenderCopy(renderer, statusLabel, nullptr, &dest);
        SDL_DestroyTexture(statusLabel);
    }

    SDL_RenderPresent(renderer);
    SDL_Delay(16);  // ~60 FPS
}

// Shutdown: free resources and quit subsystems.
void Player::shutdown() {
    if (music) {
        Mix_FreeMusic(music);
        music = nullptr;
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
    Mix_CloseAudio();
    TTF_Quit();
    SDL_Quit();
    std::cout << "Player shutdown." << std::endl;
    running = false;
}

bool Player::isRunning() const {
    return running;
}
