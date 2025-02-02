// player_audio.cpp
#include "player.h"
#include <iostream>
#include <libavutil/mathematics.h>
#include <libavutil/channel_layout.h>
#include <mutex>

bool Player::loadAudioFile(const std::string &filename) {
    // Close old device
    if (audioDev != 0) {
        SDL_CloseAudioDevice(audioDev);
        audioDev = 0;
    }
    // Close old contexts
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
        std::cerr << "Could not find stream info.\n";
        return false;
    }

    audioStreamIndex = -1;
    for (unsigned int i=0; i < fmtCtx->nb_streams; i++) {
        if (fmtCtx->streams[i]->codecpar->codec_type == AVMEDIA_TYPE_AUDIO) {
            audioStreamIndex = i;
            break;
        }
    }
    if (audioStreamIndex < 0) {
        std::cerr << "No audio stream found.\n";
        return false;
    }

    // Setup codec
    const AVCodec* codec = avcodec_find_decoder(fmtCtx->streams[audioStreamIndex]->codecpar->codec_id);
    if (!codec) {
        std::cerr << "Codec not found.\n";
        return false;
    }
    codecCtx = avcodec_alloc_context3(codec);
    if (!codecCtx) {
        std::cerr << "Could not allocate codec context.\n";
        return false;
    }
    if (avcodec_parameters_to_context(codecCtx, fmtCtx->streams[audioStreamIndex]->codecpar) < 0) {
        std::cerr << "Could not copy codec parameters.\n";
        return false;
    }
    if (avcodec_open2(codecCtx, codec, nullptr) < 0) {
        std::cerr << "Could not open codec.\n";
        return false;
    }

    // Setup SDL audio
    SDL_AudioSpec desired;
    SDL_zero(desired);
    desired.freq     = 44100;
    desired.format   = AUDIO_S16SYS;
    desired.channels = 2;
    desired.samples  = 4096;
    desired.callback = Player::sdlAudioCallback;
    desired.userdata = this;

    SDL_AudioSpec obtained;
    audioDev = SDL_OpenAudioDevice(nullptr, 0, &desired, &obtained, 0);
    if (audioDev == 0) {
        std::cerr << "SDL_OpenAudioDevice Error: " << SDL_GetError() << std::endl;
        return false;
    }

    loadedFile   = filename;
    playingAudio = false;
    std::cout << "Audio file loaded: " << loadedFile << "\n";
    return true;
}

void Player::calculateSongDuration() {
    if (fmtCtx && audioStreamIndex >= 0) {
        AVStream* stream = fmtCtx->streams[audioStreamIndex];
        int64_t size = avio_size(fmtCtx->pb);
        if (size > 0 && stream->codecpar->bit_rate > 0) {
            totalDuration = (double)size * 8.0 / stream->codecpar->bit_rate;
            std::cout << "Calculated duration: " << totalDuration << " seconds" << std::endl;
        }
    }
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

void Player::sdlAudioCallback(void* userdata, Uint8* stream, int len) {
    Player* player = static_cast<Player*>(userdata);
    player->audioCallback(stream, len);
}

void Player::audioCallback(Uint8* stream, int len) {
    std::lock_guard<std::mutex> lock(audioMutex);
    if (!fmtCtx || !codecCtx || !stream) {
        memset(stream, 0, len);
        return;
    }

    int remaining = len;
    uint8_t* out = stream;
    

    while (remaining > 0) {
        if (audioBufferIndex >= audioBufferSize) {
            audioBufferSize  = 0;
            audioBufferIndex = 0;
            bool frameDecoded = false;

            while (!frameDecoded) {
                int readResult = av_read_frame(fmtCtx, packet);
                if (readResult < 0) {
                    // EOF or error
                    memset(out, 0, remaining);
                    return;
                }
                if (packet->stream_index == audioStreamIndex) {
                    int send = avcodec_send_packet(codecCtx, packet);
                    if (send >= 0) {
                        int receive = avcodec_receive_frame(codecCtx, frame);
                        if (receive >= 0) {
                            // ~~~~~ Store PTS if valid ~~~~~
                            if (frame->pts != AV_NOPTS_VALUE) {
                                double base = av_q2d(fmtCtx->streams[audioStreamIndex]->time_base);
                                lastPTS.store(frame->pts * base, std::memory_order_relaxed);
                                totalDuration = base * fmtCtx->streams[audioStreamIndex]->duration;
                            }
                            // ~~~~~ End PTS store ~~~~~

                            // SWR init if needed
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
                                swr_init(swrCtx);
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

                            swr_convert(swrCtx, &audioBuffer, dst_nb_samples,
                                        (const uint8_t**)frame->data, frame->nb_samples);

                            audioBufferSize  = buffer_size;
                            audioBufferIndex = 0;
                            frameDecoded = true;
                        }
                    }
                }
                av_packet_unref(packet);
            }
        }

        int bytesToCopy = audioBufferSize - audioBufferIndex;
        if (bytesToCopy > remaining) {
            bytesToCopy = remaining;
        }

        // Apply volume control
        if (!isMuted) {
            int16_t* samples = (int16_t*)(audioBuffer + audioBufferIndex);
            int numSamples = bytesToCopy / 2;  // 2 bytes per sample
            float volumeScale = volume / 100.0f;
            
            for (int i = 0; i < numSamples; i++) {
                samples[i] = (int16_t)(samples[i] * volumeScale);
            }
            memcpy(out, audioBuffer + audioBufferIndex, bytesToCopy);
        } else {
            memset(out, 0, bytesToCopy);
        }

        audioBufferIndex += bytesToCopy;
        remaining -= bytesToCopy;
        out += bytesToCopy;
    }
}