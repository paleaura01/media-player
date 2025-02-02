// src/player.h
#ifndef PLAYER_H
#define PLAYER_H

#include <fstream>
#include <string>
#include <vector>
#include <SDL.h>
#include <SDL_ttf.h>

extern "C" {
#include <libavformat/avformat.h>
#include <libavcodec/avcodec.h>
#include <libswresample/swresample.h>
#include <libavutil/opt.h>
#include <libavutil/channel_layout.h>
}

// A single class that orchestrates the UI, playlists, and audio decoding.
class Player {
public:
    Player();
    ~Player();

    bool init();       // Sets up SDL, TTF, etc.
    void update();     // Main loop body: handles events, draws UI, etc.
    void shutdown();   // Cleans up everything.

    bool isRunning() const;

private:
    bool running;

    // --- UI Components / Layout ---
    SDL_Rect timeBar;
    SDL_Rect volumeBar;
    SDL_Rect playlistPanel;
    SDL_Rect mainPanel;

    // Buttons
    SDL_Rect prevButton;
    SDL_Rect playButton;
    SDL_Rect nextButton;
    SDL_Rect stopButton;
    SDL_Rect shuffleButton;
    SDL_Rect muteButton;
    SDL_Rect newPlaylistButton;

    // Playlist management
    struct Playlist {
        std::string name;
        std::vector<std::string> songs;
    };
    std::vector<Playlist> playlists;
    int activePlaylist;
    void savePlaylistState();
    void loadPlaylistState();

    // Playback tracking
    double currentTime;
    double totalDuration;
    bool isMuted;
    bool isShuffled;

    // SDL objects
    SDL_Window* window;
    SDL_Renderer* renderer;
    TTF_Font* font;

    // FFmpeg members
    AVFormatContext* fmtCtx;
    AVCodecContext*  codecCtx;
    SwrContext*      swrCtx;
    int              audioStreamIndex;
    AVPacket*        packet;
    AVFrame*         frame;
    uint8_t*         audioBuffer;
    int              audioBufferSize;
    int              audioBufferIndex;
    SDL_AudioDeviceID audioDev;
    bool             playingAudio;
    std::string      loadedFile;

    // ========== UI Helpers ==========
    SDL_Texture* renderText(const std::string &text, SDL_Color color);
    void renderButtonText(SDL_Texture* texture, const SDL_Rect& button);

    // Draw methods
    void drawTimeBar();
    void drawControls();
    void drawPlaylistPanel();

    // Playlist
    void handleMouseClick(int x, int y);
    void handleFileDrop(const char* filePath);
    void handlePlaylistCreation();
    void calculateSongDuration();

    // ========== Audio Methods ==========
    bool loadAudioFile(const std::string &filename);
    void playAudio();
    void stopAudio();

    // Audio callback
    void audioCallback(Uint8* stream, int len);
    static void sdlAudioCallback(void* userdata, Uint8* stream, int len);
};

#endif // PLAYER_H
