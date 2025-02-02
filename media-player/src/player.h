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
    SDL_Rect libraryPanel; // right-side panel for songs
    SDL_Rect mainPanel;

    // Transport Buttons
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

    // Rects for each playlist row + "X" button
    std::vector<SDL_Rect> playlistRects;
    std::vector<SDL_Rect> playlistDeleteRects;

    // Rects for each song in the active playlist
    std::vector<SDL_Rect> songRects;

    // Index of currently active playlist
    int activePlaylist;

    // For saving/loading
    void savePlaylistState();
    void loadPlaylistState();

    // Double-click detection
    int    lastPlaylistClickIndex = -1;
    Uint32 lastPlaylistClickTime  = 0;

    // Inline rename fields
    bool        isRenaming    = false;  // Are we editing a playlist name right now?
    int         renameIndex   = -1;     // Which playlist index is being edited?
    std::string renameBuffer;           // Current typed text for rename

    // Playback tracking
    double currentTime;
    double totalDuration;
    bool isMuted;
    bool isShuffled;

    // ======================================
    // Deletion Confirmation Modal Variables
    // ======================================
    bool isConfirmingDeletion = false;  // if true, show "Are you sure?" dialog
    int  deleteCandidateIndex = -1;     // which playlist is pending deletion

    // The dialog box / yes/no button rectangles
    SDL_Rect confirmDialogRect;
    SDL_Rect confirmYesButton;
    SDL_Rect confirmNoButton;

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
    void drawSongPanel();
    void drawConfirmDialog(); // NEW: draws the "Are you sure?" modal

    // Playlist logic
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
