// src/player.h
#ifndef PLAYER_H
#define PLAYER_H

#include <fstream>
#include <string>
#include <vector>
#include <atomic>
#include <SDL.h>
#include <SDL_ttf.h>

extern "C" {
#include <libavformat/avformat.h>
#include <libavcodec/avcodec.h>
#include <libswresample/swresample.h>
#include <libavutil/opt.h>
#include <libavutil/channel_layout.h>
}

// A single class that orchestrates the UI, playlists, and audio decoding
class Player {
public:
    Player();
    ~Player();

    bool init();
    void update();
    void shutdown();
    bool isRunning() const;

private:
    bool running;

    // UI Layout
    SDL_Rect timeBar;
    SDL_Rect volumeBar;
    SDL_Rect playlistPanel;
    SDL_Rect libraryPanel;
    SDL_Rect mainPanel;

    // Transport Buttons
    SDL_Rect prevButton;
    SDL_Rect playButton;
    SDL_Rect nextButton;
    SDL_Rect stopButton;
    SDL_Rect shuffleButton;
    SDL_Rect muteButton;
    SDL_Rect newPlaylistButton;
    SDL_Rect rewindButton;
    SDL_Rect forwardButton;

    // Playlist
    struct Playlist {
        std::string name;
        std::vector<std::string> songs;
    };
    std::vector<Playlist> playlists;
    std::vector<SDL_Rect> playlistRects;
    std::vector<SDL_Rect> playlistDeleteRects;
    std::vector<SDL_Rect> songRects;
    int activePlaylist;

    void savePlaylistState();
    void loadPlaylistState();

    // Double-click
    int    lastPlaylistClickIndex = -1;
    Uint32 lastPlaylistClickTime  = 0;

    // Inline rename
    bool        isRenaming    = false;
    int         renameIndex   = -1;
    std::string renameBuffer;

    // Playback
    double currentTime;
    double totalDuration;
    bool   isMuted;
    bool   isShuffled;

    // "Are you sure?" confirm deletion
    bool isConfirmingDeletion = false;
    int  deleteCandidateIndex = -1;
    SDL_Rect confirmDialogRect;
    SDL_Rect confirmYesButton;
    SDL_Rect confirmNoButton;

    // SDL
    SDL_Window* window;
    SDL_Renderer* renderer;
    TTF_Font* font;

    // FFmpeg
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

    // The last decoded PTS (in seconds), updated in audioCallback
    std::atomic<double> lastPTS{0.0};

    // UI / Mouse logic
    void handleMouseClick(int x, int y);
    void handleFileDrop(const char* filePath);
    void handlePlaylistCreation();
    void calculateSongDuration();

    // Audio
    bool loadAudioFile(const std::string &filename);
    void playAudio();
    void stopAudio();

    // If you want seeking in time bar
    void seekTo(double seconds);

    // Audio callback
    void audioCallback(Uint8* stream, int len);
    static void sdlAudioCallback(void* userdata, Uint8* stream, int len);

    // Draw methods
    SDL_Texture* renderText(const std::string &text, SDL_Color color);
    void renderButtonText(SDL_Texture* texture, const SDL_Rect& button);
    void drawTimeBar();
    void drawControls();
    void drawPlaylistPanel();
    void drawSongPanel();
    void drawConfirmDialog();
};

#endif // PLAYER_H
