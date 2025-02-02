#include "player.h"
#include <iostream>
#include <cstring>
#include <libavutil/mathematics.h>    // For av_popcount64 if needed
#include <libavutil/channel_layout.h>  // For AVChannelLayout and av_channel_layout_default()

#include "player.h"
#include <iostream>
#include <cstring>
#include <libavutil/mathematics.h>
#include <libavutil/channel_layout.h>

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
    mainPanel = { 200, 100, 600, 500 };
    
    // Control buttons (top panel)
    prevButton = { 210, 20, 40, 40 };
    playButton = { 260, 20, 40, 40 };
    nextButton = { 310, 20, 40, 40 };
    stopButton = { 360, 20, 40, 40 };
    shuffleButton = { 410, 20, 40, 40 };
    muteButton = { 460, 20, 40, 40 };
    
    // Time bar
    timeBar = { 210, 70, 540, 20 };
    volumeBar = { 510, 20, 80, 40 };
    
    // Playlist controls
    newPlaylistButton = { 10, 60, 180, 30 };
}

Player::~Player() {
    shutdown();
}

void Player::renderButtonText(SDL_Texture* texture, const SDL_Rect& button) {
    if (texture) {
        int w, h;
        SDL_QueryTexture(texture, nullptr, nullptr, &w, &h);
        SDL_Rect dest = { 
            button.x + (button.w - w) / 2,
            button.y + (button.h - h) / 2,
            w, h 
        };
        SDL_RenderCopy(renderer, texture, nullptr, &dest);
    }
}

void Player::drawControls() {
    SDL_Color white = {255, 255, 255, 255};
    
    // Previous button
    SDL_SetRenderDrawColor(renderer, 60, 60, 60, 255);
    SDL_RenderFillRect(renderer, &prevButton);
    SDL_Texture* prevText = renderText("<<", white);
    
    // Play button
    SDL_SetRenderDrawColor(renderer, 0, 200, 0, 255);
    SDL_RenderFillRect(renderer, &playButton);
    SDL_Texture* playText = renderText(playingAudio ? "||" : ">", white);
    
    // Next button
    SDL_SetRenderDrawColor(renderer, 60, 60, 60, 255);
    SDL_RenderFillRect(renderer, &nextButton);
    SDL_Texture* nextText = renderText(">>", white);
    
    // Stop button
    SDL_SetRenderDrawColor(renderer, 200, 0, 0, 255);
    SDL_RenderFillRect(renderer, &stopButton);
    SDL_Texture* stopText = renderText("□", white);
    
    // Shuffle button
    SDL_SetRenderDrawColor(renderer, isShuffled ? 0, 200, 0 : 60, 60, 60, 255);
    SDL_RenderFillRect(renderer, &shuffleButton);
    SDL_Texture* shuffleText = renderText("🔀", white);
    
    // Mute button
    SDL_SetRenderDrawColor(renderer, isMuted ? 200, 0, 0 : 60, 60, 60, 255);
    SDL_RenderFillRect(renderer, &muteButton);
    SDL_Texture* muteText = renderText(isMuted ? "🔇" : "🔊", white);
    
    // Render all button texts
    renderButtonText(prevText, prevButton);
    renderButtonText(playText, playButton);
    renderButtonText(nextText, nextButton);
    renderButtonText(stopText, stopButton);
    renderButtonText(shuffleText, shuffleButton);
    renderButtonText(muteText, muteButton);
    
    // Clean up
    SDL_DestroyTexture(prevText);
    SDL_DestroyTexture(playText);
    SDL_DestroyTexture(nextText);
    SDL_DestroyTexture(stopText);
    SDL_DestroyTexture(shuffleText);
    SDL_DestroyTexture(muteText);
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

void Player::handlePlaylistCreation() {
    std::string name = "Playlist " + std::to_string(playlists.size() + 1);
    playlists.push_back({name, std::vector<std::string>()});
    activePlaylist = playlists.size() - 1;
}

void Player::sdlAudioCallback(void* userdata, Uint8* stream, int len) {
    Player* player = static_cast<Player*>(userdata);
    player->audioCallback(stream, len);
}

void Player::audioCallback(Uint8* stream, int len) {
    static FILE* logFile = fopen("audio_debug.log", "w");
    fprintf(logFile, "Callback called with length: %d\n", len);
    fflush(logFile);

    int remaining = len;
    uint8_t* out = stream;
    
    while (remaining > 0) {
        if (audioBufferIndex >= audioBufferSize) {
            audioBufferSize = 0;
            audioBufferIndex = 0;
            
            bool frameDecoded = false;
            while (!frameDecoded) {
                int readResult = av_read_frame(fmtCtx, packet);
                fprintf(logFile, "av_read_frame result: %d\n", readResult);
                fflush(logFile);
                
                if (readResult < 0) {
                    fprintf(logFile, "End of file or error: %d\n", readResult);
                    fflush(logFile);
                    memset(out, 0, remaining);
                    return;
                }
                
                if (packet->stream_index == audioStreamIndex) {
                    int sendResult = avcodec_send_packet(codecCtx, packet);
                    fprintf(logFile, "avcodec_send_packet result: %d\n", sendResult);
                    fflush(logFile);
                    
                    if (sendResult >= 0) {
                        int receiveResult = avcodec_receive_frame(codecCtx, frame);
                        fprintf(logFile, "avcodec_receive_frame result: %d\n", receiveResult);
                        fflush(logFile);
                        
                        if (receiveResult >= 0) {
                            fprintf(logFile, "Frame decoded: samples=%d, channels=%d, format=%d\n", 
                                    frame->nb_samples, frame->ch_layout.nb_channels, frame->format);
                            fflush(logFile);
                            
                            if (!swrCtx) {
    fprintf(logFile, "Creating resampler context\n");
    fflush(logFile);
    
    swrCtx = swr_alloc();
    
    AVChannelLayout stereo_layout = AV_CHANNEL_LAYOUT_STEREO;
    
    // Use frame's existing channel layout for input
    av_opt_set_chlayout(swrCtx, "in_chlayout", &frame->ch_layout, 0);
    av_opt_set_int(swrCtx, "in_sample_rate", codecCtx->sample_rate, 0);
    av_opt_set_sample_fmt(swrCtx, "in_sample_fmt", (AVSampleFormat)frame->format, 0);
    
    // Set output to stereo
    av_opt_set_chlayout(swrCtx, "out_chlayout", &stereo_layout, 0);
    av_opt_set_int(swrCtx, "out_sample_rate", 44100, 0);
    av_opt_set_sample_fmt(swrCtx, "out_sample_fmt", AV_SAMPLE_FMT_S16, 0);
    
    int initResult = swr_init(swrCtx);
    fprintf(logFile, "swr_init result: %d\n", initResult);
    fflush(logFile);
}
                            
                            int dst_nb_samples = av_rescale_rnd(
                                swr_get_delay(swrCtx, codecCtx->sample_rate) + frame->nb_samples,
                                44100, codecCtx->sample_rate, AV_ROUND_UP);
                            
                            int buffer_size = av_samples_get_buffer_size(
                                nullptr, 2, dst_nb_samples, AV_SAMPLE_FMT_S16, 1);
                            
                            if (audioBuffer) {
                                av_free(audioBuffer);
                            }
                            audioBuffer = (uint8_t*)av_malloc(buffer_size);
                            
                            int convertResult = swr_convert(swrCtx, &audioBuffer, dst_nb_samples,
                                      (const uint8_t**)frame->data, frame->nb_samples);
                            fprintf(logFile, "swr_convert result: %d\n", convertResult);
                            fflush(logFile);
                            
                            audioBufferSize = buffer_size;
                            audioBufferIndex = 0;
                            frameDecoded = true;
                        }
                    }
                }
                av_packet_unref(packet);
            }
        }
        
        int bytesToCopy = audioBufferSize - audioBufferIndex;
        if (bytesToCopy > remaining)
            bytesToCopy = remaining;
        
        fprintf(logFile, "Copying %d bytes to output\n", bytesToCopy);
        fflush(logFile);
            
        memcpy(out, audioBuffer + audioBufferIndex, bytesToCopy);
        audioBufferIndex += bytesToCopy;
        remaining -= bytesToCopy;
        out += bytesToCopy;
    }
}

bool Player::loadAudioFile(const std::string &filename) {
    if (audioDev != 0) {
        SDL_CloseAudioDevice(audioDev);
        audioDev = 0;
    }
    
    // Clean up previous FFmpeg contexts
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
    
    // Find audio stream
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
    
    // Set up codec
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
    
    // Set up SDL audio
    SDL_AudioSpec desiredSpec;
    SDL_zero(desiredSpec);
    desiredSpec.freq = 44100;
    desiredSpec.format = AUDIO_S16SYS;
    desiredSpec.channels = 2;
    desiredSpec.samples = 4096;  // Increased buffer size
    desiredSpec.callback = Player::sdlAudioCallback;
    desiredSpec.userdata = this;
    
    SDL_AudioSpec obtainedSpec;
    audioDev = SDL_OpenAudioDevice(nullptr, 0, &desiredSpec, &obtainedSpec, 0);
    if (audioDev == 0) {
        std::cerr << "SDL_OpenAudioDevice Error: " << SDL_GetError() << std::endl;
        return false;
    }
    
    loadedFile = filename;
    playingAudio = false;  // Don't start playing immediately
    std::cout << "Audio file loaded: " << loadedFile << std::endl;
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
            
            // Handle playlist button
            if (x >= newPlaylistButton.x && x <= newPlaylistButton.x + newPlaylistButton.w &&
                y >= newPlaylistButton.y && y <= newPlaylistButton.y + newPlaylistButton.h) {
                handlePlaylistCreation();
            }
            
            // Handle playback controls
            if (y >= prevButton.y && y <= prevButton.y + prevButton.h) {
                if (x >= prevButton.x && x <= prevButton.x + prevButton.w) {
                    // Previous track
                } else if (x >= playButton.x && x <= playButton.x + playButton.w) {
                    if (!loadedFile.empty()) playAudio();
                } else if (x >= nextButton.x && x <= nextButton.x + nextButton.w) {
                    // Next track
                } else if (x >= stopButton.x && x <= stopButton.x + stopButton.w) {
                    stopAudio();
                } else if (x >= shuffleButton.x && x <= shuffleButton.x + shuffleButton.w) {
                    isShuffled = !isShuffled;
                } else if (x >= muteButton.x && x <= muteButton.x + muteButton.w) {
                    isMuted = !isMuted;
                    // Implement volume control
                }
            }
        }
        else if (event.type == SDL_DROPFILE) {
            char* filePath = event.drop.file;
            if (activePlaylist >= 0) {
                playlists[activePlaylist].songs.push_back(filePath);
                if (playlists[activePlaylist].songs.size() == 1) {
                    if (loadAudioFile(filePath)) {
                        calculateSongDuration();
                    }
                }
            }
            SDL_free(filePath);
        }
    }
    
    // Update current time if playing
    if (playingAudio && fmtCtx) {
        currentTime = (double)fmtCtx->pb->pos / (fmtCtx->bit_rate / 8);
    }
    
    // Clear screen
    SDL_SetRenderDrawColor(renderer, 30, 30, 30, 255);
    SDL_RenderClear(renderer);
    
    // Draw all UI components
    drawTimeBar();
    drawPlaylistPanel();
    drawControls();
    
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

// In player.cpp - Add these missing implementation methods:

void Player::drawTimeBar() {
    // Background
    SDL_SetRenderDrawColor(renderer, 30, 30, 30, 255);
    SDL_RenderFillRect(renderer, &timeBar);
    
    // Progress
    if (totalDuration > 0) {
        SDL_Rect progress = timeBar;
        progress.w = (int)(timeBar.w * (currentTime / totalDuration));
        SDL_SetRenderDrawColor(renderer, 0, 255, 0, 255);
        SDL_RenderFillRect(renderer, &progress);
    }
    
    // Time text
    char timeText[32];
    int currentMinutes = (int)currentTime / 60;
    int currentSeconds = (int)currentTime % 60;
    int totalMinutes = (int)totalDuration / 60;
    int totalSeconds = (int)totalDuration % 60;
    snprintf(timeText, sizeof(timeText), "%d:%02d / %d:%02d", 
             currentMinutes, currentSeconds, totalMinutes, totalSeconds);
    
    SDL_Color white = {255, 255, 255, 255};
    SDL_Texture* timeTexture = renderText(timeText, white);
    if (timeTexture) {
        int w, h;
        SDL_QueryTexture(timeTexture, nullptr, nullptr, &w, &h);
        SDL_Rect dest = { timeBar.x + (timeBar.w - w) / 2, 
                         timeBar.y - h - 5, w, h };
        SDL_RenderCopy(renderer, timeTexture, nullptr, &dest);
        SDL_DestroyTexture(timeTexture);
    }
}

void Player::drawPlaylistPanel() {
    // Draw panel background
    SDL_SetRenderDrawColor(renderer, 40, 40, 40, 255);
    SDL_RenderFillRect(renderer, &playlistPanel);
    
    // Draw "New Playlist" button
    SDL_SetRenderDrawColor(renderer, 60, 60, 60, 255);
    SDL_RenderFillRect(renderer, &newPlaylistButton);
    SDL_Color white = {255, 255, 255, 255};
    SDL_Texture* newPlaylistText = renderText("New Playlist", white);
    if (newPlaylistText) {
        int w, h;
        SDL_QueryTexture(newPlaylistText, nullptr, nullptr, &w, &h);
        SDL_Rect dest = { newPlaylistButton.x + (newPlaylistButton.w - w) / 2,
                         newPlaylistButton.y + (newPlaylistButton.h - h) / 2, w, h };
        SDL_RenderCopy(renderer, newPlaylistText, nullptr, &dest);
        SDL_DestroyTexture(newPlaylistText);
    }
    
    // Draw playlists
    int yOffset = newPlaylistButton.y + newPlaylistButton.h + 10;
    for (size_t i = 0; i < playlists.size(); i++) {
        SDL_Rect playlistRect = { playlistPanel.x + 5, yOffset, 
                                 playlistPanel.w - 10, 25 };
        if ((int)i == activePlaylist) {
            SDL_SetRenderDrawColor(renderer, 60, 100, 60, 255);
        } else {
            SDL_SetRenderDrawColor(renderer, 50, 50, 50, 255);
        }
        SDL_RenderFillRect(renderer, &playlistRect);
        
        SDL_Texture* playlistName = renderText(playlists[i].name, white);
        if (playlistName) {
            int w, h;
            SDL_QueryTexture(playlistName, nullptr, nullptr, &w, &h);
            SDL_Rect dest = { playlistRect.x + 5, 
                             playlistRect.y + (playlistRect.h - h) / 2, w, h };
            SDL_RenderCopy(renderer, playlistName, nullptr, &dest);
            SDL_DestroyTexture(playlistName);
        }
        yOffset += playlistRect.h + 5;
    }
}

void Player::calculateSongDuration() {
    if (fmtCtx && audioStreamIndex >= 0) {
        int64_t duration = fmtCtx->duration + (fmtCtx->duration <= INT64_MAX - 5000 ? 5000 : 0);
        totalDuration = duration / AV_TIME_BASE;
    }
}

bool Player::isRunning() const {
    return running;
}
