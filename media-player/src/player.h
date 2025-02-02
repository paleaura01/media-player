// src/player.h
#ifndef PLAYER_H
#define PLAYER_H

#include <string>
#include <SDL.h>
#include <SDL_ttf.h>

// FFmpeg headers (wrapped in extern "C")
extern "C" {
#include <libavformat/avformat.h>
#include <libavcodec/avcodec.h>
#include <libswresample/swresample.h>
#include <libavutil/opt.h>
#include <libavutil/channel_layout.h>
}

class Player {
public:
    Player();
    ~Player();

    // Initialize SDL (video and audio), FFmpeg, window, renderer, font, and the audio device.
    bool init();

    // Process events, update the GUI, and handle audio decoding/playback.
    void update();

    // Clean up all resources.
    void shutdown();

    // Returns whether the player is still running.
    bool isRunning() const;

private:
    bool running;

    // SDL objects.
    SDL_Window* window;
    SDL_Renderer* renderer;
    TTF_Font* font;

    // GUI button rectangles.
    SDL_Rect playButton;
    SDL_Rect stopButton;

    // FFmpeg decoding members.
    AVFormatContext* fmtCtx;
    AVCodecContext* codecCtx;
    SwrContext* swrCtx;
    int audioStreamIndex;
    AVPacket* packet;
    AVFrame* frame;

    // Audio output buffer (decoded/resampled to S16 stereo at 44100 Hz).
    uint8_t* audioBuffer;
    int audioBufferSize;
    int audioBufferIndex;

    // SDL audio device.
    SDL_AudioDeviceID audioDev;

    // Playback state.
    bool playingAudio;

    // Currently loaded file path.
    std::string loadedFile;

    // Helper: render text to an SDL_Texture.
    SDL_Texture* renderText(const std::string &text, SDL_Color color);

    // Load an audio file using FFmpeg.
    bool loadAudioFile(const std::string &filename);

    // Start and stop audio playback (using SDL_PauseAudioDevice).
    void playAudio();
    void stopAudio();

    // Audio callback: decodes audio frames and fills the output stream.
    void audioCallback(Uint8* stream, int len);
    static void sdlAudioCallback(void* userdata, Uint8* stream, int len);
};

#endif // PLAYER_H
