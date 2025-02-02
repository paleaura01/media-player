// src/player.cpp
#include "player.h"
#include <iostream>
#include <cstring>
#include <libavutil/mathematics.h>    // For av_popcount64 if needed
#include <libavutil/channel_layout.h>  // For AVChannelLayout and av_channel_layout_default()

Player::Player() 
    : running(true), window(nullptr), renderer(nullptr), font(nullptr),
      fmtCtx(nullptr), codecCtx(nullptr), swrCtx(nullptr),
      audioStreamIndex(-1), packet(nullptr), frame(nullptr),
      audioBuffer(nullptr), audioBufferSize(0), audioBufferIndex(0),
      audioDev(0), playingAudio(false), loadedFile("")
{
    // Set button positions and sizes.
    playButton = { 50, 500, 150, 50 };
    stopButton = { 250, 500, 150, 50 };
}

Player::~Player() {
    shutdown();
}

bool Player::init() {
    // Initialize FFmpeg networking (optional).
    avformat_network_init();

    if (SDL_Init(SDL_INIT_VIDEO | SDL_INIT_AUDIO) != 0) {
        std::cerr << "SDL_Init Error: " << SDL_GetError() << std::endl;
        return false;
    }
    if (TTF_Init() != 0) {
        std::cerr << "TTF_Init Error: " << TTF_GetError() << std::endl;
        return false;
    }
    
    window = SDL_CreateWindow("Barebones Audio Player", SDL_WINDOWPOS_CENTERED, SDL_WINDOWPOS_CENTERED,
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
    
    // Allocate FFmpeg packet and frame.
    packet = av_packet_alloc();
    frame = av_frame_alloc();
    if (!packet || !frame) {
        std::cerr << "Failed to allocate FFmpeg packet or frame." << std::endl;
        return false;
    }
    
    // Initialize FFmpeg pointers.
    fmtCtx = nullptr;
    codecCtx = nullptr;
    swrCtx = nullptr;
    audioStreamIndex = -1;
    audioBuffer = nullptr;
    audioBufferSize = 0;
    audioBufferIndex = 0;
    audioDev = 0;
    playingAudio = false;
    loadedFile = "";
    
    // Enable file drop events.
    SDL_EventState(SDL_DROPFILE, SDL_ENABLE);
    
    std::cout << "Initialization successful." << std::endl;
    return true;
}

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

void Player::sdlAudioCallback(void* userdata, Uint8* stream, int len) {
    Player* player = static_cast<Player*>(userdata);
    player->audioCallback(stream, len);
}

void Player::audioCallback(Uint8* stream, int len) {
    int remaining = len;
    uint8_t* out = stream;
    
    while (remaining > 0) {
        if (audioBufferIndex >= audioBufferSize) {
            audioBufferSize = 0;
            audioBufferIndex = 0;
            // Decode frames until we have decoded audio data.
            while (audioBufferSize == 0) {
                if (av_read_frame(fmtCtx, packet) < 0) {
                    memset(out, 0, remaining);
                    return;
                }
                if (packet->stream_index == audioStreamIndex) {
                    if (avcodec_send_packet(codecCtx, packet) >= 0) {
                        int ret = avcodec_receive_frame(codecCtx, frame);
                        if (ret >= 0) {
#if defined(avcodec_parameters_get_channels)
                            int nb_channels = avcodec_parameters_get_channels(fmtCtx->streams[audioStreamIndex]->codecpar);
#else
                            int nb_channels = 2;
#endif
                            if (nb_channels <= 0)
                                nb_channels = 2;  // Fallback to stereo.
                            
                            // Fill an AVChannelLayout with the default layout for nb_channels.
                            AVChannelLayout layout;
                            av_channel_layout_default(&layout, nb_channels);
                            int64_t in_ch_layout = layout.u.mask;
                            
                            if (!swrCtx) {
                                swrCtx = swr_alloc();
                                av_opt_set_int(swrCtx, "in_channel_layout", in_ch_layout, 0);
                                av_opt_set_int(swrCtx, "in_sample_rate", codecCtx->sample_rate, 0);
                                av_opt_set_sample_fmt(swrCtx, "in_sample_fmt", codecCtx->sample_fmt, 0);
                                av_opt_set_int(swrCtx, "out_channel_layout", AV_CH_LAYOUT_STEREO, 0);
                                av_opt_set_int(swrCtx, "out_sample_rate", 44100, 0);
                                av_opt_set_sample_fmt(swrCtx, "out_sample_fmt", AV_SAMPLE_FMT_S16, 0);
                                swr_init(swrCtx);
                            }
                            int dst_nb_samples = av_rescale_rnd(swr_get_delay(swrCtx, codecCtx->sample_rate) + frame->nb_samples,
                                                                 44100, codecCtx->sample_rate, AV_ROUND_UP);
                            int buffer_size = av_samples_get_buffer_size(nullptr, 2, dst_nb_samples, AV_SAMPLE_FMT_S16, 1);
                            if (audioBuffer) {
                                av_free(audioBuffer);
                            }
                            audioBuffer = (uint8_t*)av_malloc(buffer_size);
                            swr_convert(swrCtx, &audioBuffer, dst_nb_samples,
                                        (const uint8_t**)frame->data, frame->nb_samples);
                            audioBufferSize = buffer_size;
                            audioBufferIndex = 0;
                        }
                    }
                }
                av_packet_unref(packet);
            }
        }
        int bytesToCopy = audioBufferSize - audioBufferIndex;
        if (bytesToCopy > remaining)
            bytesToCopy = remaining;
        memcpy(out, audioBuffer + audioBufferIndex, bytesToCopy);
        audioBufferIndex += bytesToCopy;
        remaining -= bytesToCopy;
        out += bytesToCopy;
    }
}

bool Player::loadAudioFile(const std::string &filename) {
    // Clean up previous FFmpeg contexts.
    if (fmtCtx) {
        avformat_close_input(&fmtCtx);
        fmtCtx = nullptr;
    }
    if (codecCtx) {
        avcodec_free_context(&codecCtx);
        codecCtx = nullptr;
    }
    if (swrCtx) {
        swr_free(&swrCtx);
        swrCtx = nullptr;
    }
    
    if (avformat_open_input(&fmtCtx, filename.c_str(), nullptr, nullptr) != 0) {
        std::cerr << "Could not open audio file: " << filename << std::endl;
        return false;
    }
    if (avformat_find_stream_info(fmtCtx, nullptr) < 0) {
        std::cerr << "Could not find stream information." << std::endl;
        return false;
    }
    audioStreamIndex = -1;
    for (unsigned int i = 0; i < fmtCtx->nb_streams; i++) {
        if (fmtCtx->streams[i]->codecpar->codec_type == AVMEDIA_TYPE_AUDIO) {
            audioStreamIndex = i;
            break;
        }
    }
    if (audioStreamIndex == -1) {
        std::cerr << "No audio stream found." << std::endl;
        return false;
    }
    const AVCodec* codec = avcodec_find_decoder(fmtCtx->streams[audioStreamIndex]->codecpar->codec_id);
    if (!codec) {
        std::cerr << "Codec not found." << std::endl;
        return false;
    }
    codecCtx = avcodec_alloc_context3(codec);
    if (!codecCtx) {
        std::cerr << "Could not allocate codec context." << std::endl;
        return false;
    }
    if (avcodec_parameters_to_context(codecCtx, fmtCtx->streams[audioStreamIndex]->codecpar) < 0) {
        std::cerr << "Could not copy codec parameters." << std::endl;
        return false;
    }
    if (avcodec_open2(codecCtx, codec, nullptr) < 0) {
        std::cerr << "Could not open codec." << std::endl;
        return false;
    }
    loadedFile = filename;
    std::cout << "Audio file loaded: " << loadedFile << std::endl;
    
    SDL_AudioSpec desiredSpec;
    SDL_zero(desiredSpec);
    desiredSpec.freq = 44100;
    desiredSpec.format = AUDIO_S16SYS;
    desiredSpec.channels = 2;
    desiredSpec.samples = 1024;
    desiredSpec.callback = Player::sdlAudioCallback;
    desiredSpec.userdata = this;
    
    SDL_AudioSpec obtainedSpec;
    audioDev = SDL_OpenAudioDevice(nullptr, 0, &desiredSpec, &obtainedSpec, 0);
    if (audioDev == 0) {
        std::cerr << "SDL_OpenAudioDevice Error: " << SDL_GetError() << std::endl;
        return false;
    }
    SDL_PauseAudioDevice(audioDev, 0);
    playingAudio = true;
    return true;
}

void Player::playAudio() {
    if (audioDev != 0 && !playingAudio) {
        SDL_PauseAudioDevice(audioDev, 0);
        playingAudio = true;
    }
}

void Player::stopAudio() {
    if (audioDev != 0 && playingAudio) {
        SDL_PauseAudioDevice(audioDev, 1);
        playingAudio = false;
    }
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
            if (x >= playButton.x && x <= playButton.x + playButton.w &&
                y >= playButton.y && y <= playButton.y + playButton.h) {
                std::cout << "Play button pressed." << std::endl;
                if (!loadedFile.empty()) {
                    playAudio();
                } else {
                    std::cout << "No audio loaded!" << std::endl;
                }
            }
            if (x >= stopButton.x && x <= stopButton.x + stopButton.w &&
                y >= stopButton.y && y <= stopButton.y + stopButton.h) {
                std::cout << "Stop button pressed." << std::endl;
                stopAudio();
            }
        }
        else if (event.type == SDL_DROPFILE) {
            char* filePath = event.drop.file;
            std::cout << "File dropped: " << filePath << std::endl;
            if (!loadedFile.empty()) {
                stopAudio();
                if (fmtCtx) {
                    avformat_close_input(&fmtCtx);
                    fmtCtx = nullptr;
                }
            }
            if (!loadAudioFile(filePath)) {
                std::cerr << "Failed to load audio file." << std::endl;
            }
            SDL_free(filePath);
        }
    }
    
    SDL_SetRenderDrawColor(renderer, 50, 50, 50, 255);
    SDL_RenderClear(renderer);
    
    SDL_SetRenderDrawColor(renderer, 0, 200, 0, 255);
    SDL_RenderFillRect(renderer, &playButton);
    
    SDL_SetRenderDrawColor(renderer, 200, 0, 0, 255);
    SDL_RenderFillRect(renderer, &stopButton);
    
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
    }
    if (frame) {
        av_frame_free(&frame);
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
