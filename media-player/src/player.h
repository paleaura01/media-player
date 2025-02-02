// player.h
#ifndef PLAYER_H
#define PLAYER_H

#include <string>
#include <SDL.h>
#include <SDL_ttf.h>
#include <SDL_mixer.h>

class Player {
public:
    Player();
    ~Player();

    // Initialize SDL subsystems, create window, renderer, load font, and prepare audio.
    bool init();

    // Process events, render GUI, and handle button clicks.
    void update();

    // Clean up resources.
    void shutdown();

    // Return whether the player should continue running.
    bool isRunning() const;

private:
    bool running;

    SDL_Window* window;
    SDL_Renderer* renderer;
    TTF_Font* font;

    // Audio playback
    Mix_Music* music;
    std::string loadedFile;

    // GUI button rectangles
    SDL_Rect playButton;
    SDL_Rect stopButton;

    // Helper to render text; returns an SDL_Texture* that must be destroyed after use.
    SDL_Texture* renderText(const std::string &text, SDL_Color color);
};

#endif // PLAYER_H
