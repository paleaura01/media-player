// player.h
#ifndef PLAYER_H
#define PLAYER_H

#include <fstream>
#include <string>
#include <vector>
#include <atomic>
#include <mutex>
#include <SDL.h>
#include <SDL_ttf.h>
#include <SDL_image.h>  // For icon loading

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

    // Increments the play count for the finished track.
    void incrementFinishedTrack();

private:
    // Runtime state
    bool running;
    
    // UI layout rectangles
    SDL_Rect timeBar;
    SDL_Rect volumeBar;
    SDL_Rect playlistPanel;
    SDL_Rect libraryPanel;
    SDL_Rect mainPanel;
    
    // Transport Buttons
    SDL_Rect prevButton;
    SDL_Rect playButton;
    SDL_Rect nextButton;
    SDL_Rect shuffleButton;
    SDL_Rect muteButton;
    SDL_Rect newPlaylistButton;
    SDL_Rect rewindButton;
    SDL_Rect forwardButton;
    
    // Playlist structure
    struct Playlist {
        std::string name;
        std::vector<std::string> songs;
        std::vector<int> playCounts;
        // NEW: Last played timestamp (in seconds) for each song.
        std::vector<double> lastPositions;
    };
    std::vector<Playlist> playlists;
    std::vector<SDL_Rect> playlistRects;
    std::vector<SDL_Rect> playlistDeleteRects;
    std::vector<SDL_Rect> songRects;
    
    void savePlaylistState();
    void loadPlaylistState();

    // For double-click renaming
    int    lastPlaylistClickIndex = -1;
    Uint32 lastPlaylistClickTime  = 0;
    bool        isRenaming    = false;
    int         renameIndex   = -1;
    std::string renameBuffer;
    
    // Playback state
    double currentTime;
    double totalDuration;
    bool isMuted;
    bool isShuffled;
    int activePlaylist;
    std::atomic<bool> reachedEOF{false};
    std::vector<int> shuffleOrder;
    int shuffleIndex = 0;
    std::vector<bool> playedInCycle;
    double lastCountCheckTime = 0.0;

    // "Are you sure?" deletion confirmation for playlists
    bool isConfirmingDeletion = false;
    int  deleteCandidateIndex = -1;
    SDL_Rect confirmDialogRect;
    SDL_Rect confirmYesButton;
    SDL_Rect confirmNoButton;
    
    // SDL objects
    SDL_Window* window;
    SDL_Renderer* renderer;
    TTF_Font* font;
    
    // FFmpeg objects
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
    
    // Last decoded PTS (in seconds) from the audio callback.
    std::atomic<double> lastPTS{0.0};

    // Volume (0 to 100)
    float volume = 100.0f;
    
    // For thread safety
    std::mutex audioMutex;
    std::mutex playlistMutex;
    
    // For song deletion via "X" button on a song row.
    int hoveredSongIndex = -1;
    int songListScrollOffset = 0;  // NEW: Scroll offset for the song list

    // UI / mouse logic
    void handleMouseClick(int x, int y);
    void handleFileDrop(const char* filePath);
    void handlePlaylistCreation();
    void calculateSongDuration();
    
    // Audio methods
    bool loadAudioFile(const std::string &filename);
    void playAudio();
    void stopAudio();
    void playNextTrack();
    bool seekTo(double seconds);   // Changed to just one declaration with void return type
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

    // NEW: Variables to store resume information.
    double resumePosition = 0.0;
    double lastSaveTime = 0.0;
    bool resumed = false;
};

#endif // PLAYER_H
