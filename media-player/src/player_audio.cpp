// player_audio.cpp
#include "player.h"
#include <iostream>
#include <libavutil/mathematics.h>
#include <libavutil/channel_layout.h>

bool Player::loadAudioFile(const std::string &filename) {
    // Close the existing audio device
    if (audioDev != 0) {
        SDL_CloseAudioDevice(audioDev);
        audioDev = 0;
    }
    
    // Clean up any existing FFmpeg contexts
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
    
    // Open input
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
    desiredSpec.freq     = 44100;
    desiredSpec.format   = AUDIO_S16SYS;
    desiredSpec.channels = 2;
    desiredSpec.samples  = 4096;
    desiredSpec.callback = Player::sdlAudioCallback;
    desiredSpec.userdata = this;
    
    SDL_AudioSpec obtainedSpec;
    audioDev = SDL_OpenAudioDevice(nullptr, 0, &desiredSpec, &obtainedSpec, 0);
    if (audioDev == 0) {
        std::cerr << "SDL_OpenAudioDevice Error: " << SDL_GetError() << std::endl;
        return false;
    }
    
    loadedFile   = filename;
    playingAudio = false; 
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

// The static callback dispatches to our member function
void Player::sdlAudioCallback(void* userdata, Uint8* stream, int len) {
    Player* player = static_cast<Player*>(userdata);
    player->audioCallback(stream, len);
}

void Player::audioCallback(Uint8* stream, int len) {
    static FILE* logFile = fopen("audio_debug.log", "w");
    if (!logFile) return;

    fprintf(logFile, "Callback called with length: %d\n", len);
    fflush(logFile);

    int remaining = len;
    uint8_t* out = stream;

    while (remaining > 0) {
        // Need new data?
        if (audioBufferIndex >= audioBufferSize) {
            audioBufferSize  = 0;
            audioBufferIndex = 0;
            bool frameDecoded = false;

            while (!frameDecoded) {
                int readResult = av_read_frame(fmtCtx, packet);
                fprintf(logFile, "av_read_frame result: %d\n", readResult);
                fflush(logFile);

                if (readResult < 0) {
                    // EOF or error
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
                            fprintf(logFile, "Frame decoded: samples=%d, channels=%d\n",
                                    frame->nb_samples, frame->ch_layout.nb_channels);
                            fflush(logFile);

                            if (!swrCtx) {
                                swrCtx = swr_alloc();
                                AVChannelLayout stereo_layout = AV_CHANNEL_LAYOUT_STEREO;
                                av_opt_set_chlayout(swrCtx, "in_chlayout",   &frame->ch_layout, 0);
                                av_opt_set_int(swrCtx, "in_sample_rate",     codecCtx->sample_rate, 0);
                                av_opt_set_sample_fmt(swrCtx, "in_sample_fmt",
                                                      (AVSampleFormat)frame->format, 0);

                                av_opt_set_chlayout(swrCtx, "out_chlayout",  &stereo_layout, 0);
                                av_opt_set_int(swrCtx, "out_sample_rate",   44100, 0);
                                av_opt_set_sample_fmt(swrCtx, "out_sample_fmt",
                                                      AV_SAMPLE_FMT_S16, 0);

                                int initRes = swr_init(swrCtx);
                                fprintf(logFile, "swr_init result: %d\n", initRes);
                                fflush(logFile);
                            }
                            int dst_nb_samples = av_rescale_rnd(
                                swr_get_delay(swrCtx, codecCtx->sample_rate) + frame->nb_samples,
                                44100, codecCtx->sample_rate, AV_ROUND_UP
                            );

                            int buffer_size = av_samples_get_buffer_size(
                                nullptr, 2, dst_nb_samples, AV_SAMPLE_FMT_S16, 1
                            );

                            if (audioBuffer) {
                                av_free(audioBuffer);
                            }
                            audioBuffer = (uint8_t*)av_malloc(buffer_size);

                            int convertResult = swr_convert(swrCtx, &audioBuffer, dst_nb_samples,
                                                           (const uint8_t**)frame->data, frame->nb_samples);
                            fprintf(logFile, "swr_convert result: %d\n", convertResult);
                            fflush(logFile);

                            audioBufferSize  = buffer_size;
                            audioBufferIndex = 0;
                            frameDecoded = true;
                        }
                    }
                }
                av_packet_unref(packet);
            }
        }
        // Copy from audioBuffer to output
        int bytesToCopy = audioBufferSize - audioBufferIndex;
        if (bytesToCopy > remaining) {
            bytesToCopy = remaining;
        }
        fprintf(logFile, "Copying %d bytes to output\n", bytesToCopy);
        fflush(logFile);

        memcpy(out, audioBuffer + audioBufferIndex, bytesToCopy);
        audioBufferIndex += bytesToCopy;
        remaining        -= bytesToCopy;
        out             += bytesToCopy;
    }
}
