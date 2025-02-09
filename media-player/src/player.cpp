// player.cpp
#include "player.h"
#include <iostream>
#include <cmath>
#include <cstdlib>
#include <algorithm>  // for std::swap

// Helper: Fisher–Yates shuffle using rand()
template<typename RandomIt>
void my_shuffle(RandomIt first, RandomIt last) {
    for (auto i = (last - first) - 1; i > 0; --i) {
        auto j = rand() % (i + 1);
        std::swap(first[i], first[j]);
    }
}

#ifdef _WIN32
#pragma comment(lib, "SDL2_image.lib")
#endif

Player::Player()
    : running(true), window(nullptr), renderer(nullptr), font(nullptr),
      fmtCtx(nullptr), codecCtx(nullptr), swrCtx(nullptr),
      audioStreamIndex(-1), packet(nullptr), frame(nullptr),
      audioBuffer(nullptr), audioBufferSize(0), audioBufferIndex(0),
      audioDev(0), playingAudio(false), loadedFile(""),
      currentTime(0), totalDuration(0), isMuted(false), isShuffled(false),
      activePlaylist(-1), shuffleIndex(0)
{
    // Set up UI rectangles
    timeBar       = { 10, 10, 780, 20 };
    prevButton    = { 10,  40, 40, 40 };
    playButton    = { 55,  40, 80, 40 };
    nextButton    = { 140, 40, 40, 40 };
    shuffleButton = { 235, 40, 80, 40 };
    muteButton    = { 320, 40, 80, 40 };
    rewindButton  = { 405, 40, 40, 40 };
    forwardButton = { 450, 40, 40, 40 };
    volumeBar     = { 545, 40, 240, 40 };

    playlistPanel = { 0,  90, 200, 510 };
    libraryPanel  = { 200, 90, 600, 510 };
    newPlaylistButton = { playlistPanel.x + 10, playlistPanel.y + 10, playlistPanel.w - 20, 30 };
    mainPanel     = {0,0,0,0};

    // Confirmation dialog for playlist deletion.
    confirmDialogRect = { 250,200,300,150 };
    confirmYesButton  = { 280,300,100,30 };
    confirmNoButton   = { 420,300,100,30 };
}

Player::~Player() {
    // NEW: Before saving state, update the last played timestamp for the current song.
    if (activePlaylist >= 0 && !loadedFile.empty()) {
        auto &pl = playlists[activePlaylist];
        if (pl.lastPositions.size() < pl.songs.size()) {
            pl.lastPositions.resize(pl.songs.size(), 0.0);
        }
        for (size_t i = 0; i < pl.songs.size(); i++) {
            if (pl.songs[i] == loadedFile) {
                pl.lastPositions[i] = currentTime;
                break;
            }
        }
    }
    savePlaylistState();
    shutdown();
}

bool Player::init() {
    // Load playlist state (if any)
    loadPlaylistState();

    if (SDL_Init(SDL_INIT_VIDEO | SDL_INIT_AUDIO) != 0) {
        std::cerr << "SDL_Init Error: " << SDL_GetError() << std::endl;
        return false;
    }
    if (TTF_Init() != 0) {
        std::cerr << "TTF_Init Error: " << TTF_GetError() << std::endl;
        return false;
    }
    if (IMG_Init(IMG_INIT_PNG | IMG_INIT_JPG) == 0) {
        std::cerr << "IMG_Init Error: " << IMG_GetError() << std::endl;
        // Not fatal—just warn.
    }

    window = SDL_CreateWindow("Twink Audio Player",
                              SDL_WINDOWPOS_CENTERED, SDL_WINDOWPOS_CENTERED,
                              800, 600, SDL_WINDOW_SHOWN);
    if (!window) {
        std::cerr << "SDL_CreateWindow Error: " << SDL_GetError() << std::endl;
        return false;
    }
    {
        SDL_Surface* icon = IMG_Load("assets/icon.ico");
        if (icon) {
            SDL_SetWindowIcon(window, icon);
            SDL_FreeSurface(icon);
        } else {
            std::cerr << "Failed to load icon: " << IMG_GetError() << std::endl;
        }
    }
    renderer = SDL_CreateRenderer(window, -1, SDL_RENDERER_ACCELERATED);
    if (!renderer) {
        std::cerr << "SDL_CreateRenderer Error: " << SDL_GetError() << std::endl;
        return false;
    }
    font = TTF_OpenFont("Arial.ttf", 12);
    if (!font) {
        std::cerr << "TTF_OpenFont Error: " << TTF_GetError() << std::endl;
        return false;
    }
    TTF_SetFontStyle(font, TTF_STYLE_BOLD);

    packet = av_packet_alloc();
    frame = av_frame_alloc();
    if (!packet || !frame) {
        std::cerr << "Failed to allocate FFmpeg packet/frame." << std::endl;
        return false;
    }
    SDL_EventState(SDL_DROPFILE, SDL_ENABLE);
    srand(static_cast<unsigned>(SDL_GetTicks()));

    std::cout << "Initialization successful.\n";
    return true;
}

void Player::update() {
    SDL_Event event;
    while (SDL_PollEvent(&event)) {
        if (event.type == SDL_MOUSEBUTTONDOWN) {
            int mx = event.button.x;
            int my = event.button.y;
            if (my >= timeBar.y && my <= timeBar.y + timeBar.h &&
                mx >= timeBar.x && mx <= timeBar.x + timeBar.w) {
                if (totalDuration > 0.0) {
                    double fraction = double(mx - timeBar.x) / double(timeBar.w);
                    if (fraction < 0) fraction = 0;
                    if (fraction > 1) fraction = 1;
                    double newTime = fraction * totalDuration;
                    seekTo(newTime);  // Now returns bool but we don't need to check it here
                }
                continue;
            }
        }
        if (event.type == SDL_QUIT)
            running = false;
        else if (event.type == SDL_MOUSEMOTION) {
            int mx = event.motion.x;
            int my = event.motion.y;
            hoveredSongIndex = -1;
            for (size_t i = 0; i < songRects.size(); i++) {
                SDL_Rect r = songRects[i];
                if (mx >= r.x && mx <= r.x + r.w &&
                    my >= r.y && my <= r.y + r.h) {
                    hoveredSongIndex = static_cast<int>(i);
                    break;
                }
            }
        }
        else if (event.type == SDL_MOUSEBUTTONDOWN)
            handleMouseClick(event.button.x, event.button.y);
        else if (event.type == SDL_DROPFILE) {
            handleFileDrop(event.drop.file);
            SDL_free(event.drop.file);
        }
        else if (event.type == SDL_TEXTINPUT && isRenaming)
            renameBuffer += event.text.text;
        else if (event.type == SDL_KEYDOWN && isRenaming) {
            if (event.key.keysym.sym == SDLK_RETURN) {
                if (renameIndex >= 0 && renameIndex < (int)playlists.size())
                    playlists[renameIndex].name = renameBuffer;
                isRenaming = false;
                SDL_StopTextInput();
            } else if (event.key.keysym.sym == SDLK_BACKSPACE && !renameBuffer.empty())
                renameBuffer.pop_back();
            else if (event.key.keysym.sym == SDLK_ESCAPE) {
                isRenaming = false;
                SDL_StopTextInput();
            }
        }
    }

    if (playingAudio) {
        currentTime = lastPTS.load(std::memory_order_relaxed);
        if ((currentTime >= totalDuration && totalDuration > 0) ||
            reachedEOF.load(std::memory_order_relaxed)) {
            incrementFinishedTrack();
            reachedEOF.store(false, std::memory_order_relaxed);
            playNextTrack();
        }
    }

    SDL_SetRenderDrawColor(renderer, 30, 30, 30, 255);
    SDL_RenderClear(renderer);

    drawPlaylistPanel();
    drawSongPanel();
    drawControls();
    drawTimeBar();
    drawConfirmDialog();

    SDL_RenderPresent(renderer);
    SDL_Delay(16);
}

void Player::incrementFinishedTrack() {
    if (activePlaylist < 0 || playlists[activePlaylist].songs.empty())
        return;
    for (size_t i = 0; i < playlists[activePlaylist].songs.size(); i++) {
        if (playlists[activePlaylist].songs[i] == loadedFile) {
            if (i < playlists[activePlaylist].playCounts.size())
                playlists[activePlaylist].playCounts[i]++;
            break;
        }
    }
    savePlaylistState();
}

void Player::playNextTrack() {
    if (activePlaylist < 0 || playlists[activePlaylist].songs.empty())
        return;

    const auto &songs = playlists[activePlaylist].songs;
    if (isShuffled) {
        if (songs.size() == 1) {
            if (loadAudioFile(songs[0]))
                playAudio();
            return;
        }
        // Use playCounts: select among those with the minimum play count.
        const auto &counts = playlists[activePlaylist].playCounts;
        int minCount = counts[0];
        for (size_t i = 1; i < counts.size(); i++) {
            if (counts[i] < minCount)
                minCount = counts[i];
        }
        std::vector<int> candidates;
        int currentIndex = -1;
        for (size_t i = 0; i < songs.size(); i++) {
            if (songs[i] == loadedFile)
                currentIndex = static_cast<int>(i);
            if (counts[i] == minCount) {
                if (static_cast<int>(i) == currentIndex && songs.size() > 1)
                    continue;
                candidates.push_back(static_cast<int>(i));
            }
        }
        if (candidates.empty()) {
            for (size_t i = 0; i < songs.size(); i++) {
                if (counts[i] == minCount)
                    candidates.push_back(static_cast<int>(i));
            }
        }
        int chosenIndex = candidates[rand() % candidates.size()];
        if (loadAudioFile(songs[chosenIndex]))
            playAudio();
    } else {
        int currentIndex = -1;
        for (size_t i = 0; i < songs.size(); i++) {
            if (songs[i] == loadedFile) {
                currentIndex = static_cast<int>(i);
                break;
            }
        }
        int nextIndex = (currentIndex + 1) % songs.size();
        if (loadAudioFile(songs[nextIndex]))
            playAudio();
    }
}

void Player::handleMouseClick(int x, int y) {
    std::lock_guard<std::mutex> lock(playlistMutex);

    // (The progress bar click is handled in update(), so here we only process other areas.)
    if (isConfirmingDeletion) {
        if (x >= confirmYesButton.x && x <= confirmYesButton.x + confirmYesButton.w &&
            y >= confirmYesButton.y && y <= confirmYesButton.y + confirmYesButton.h) {
            if (deleteCandidateIndex >= 0 && deleteCandidateIndex < (int)playlists.size()) {
                playlists.erase(playlists.begin() + deleteCandidateIndex);
                playlistRects.erase(playlistRects.begin() + deleteCandidateIndex);
                playlistDeleteRects.erase(playlistDeleteRects.begin() + deleteCandidateIndex);
                if (deleteCandidateIndex == activePlaylist)
                    activePlaylist = -1;
                else if (deleteCandidateIndex < activePlaylist)
                    activePlaylist--;
            }
            isConfirmingDeletion = false;
            deleteCandidateIndex = -1;
            return;
        }
        if (x >= confirmNoButton.x && x <= confirmNoButton.x + confirmNoButton.w &&
            y >= confirmNoButton.y && y <= confirmNoButton.y + confirmNoButton.h) {
            isConfirmingDeletion = false;
            deleteCandidateIndex = -1;
            return;
        }
        return;
    }

    if (activePlaylist >= 0 && activePlaylist < (int)playlists.size() &&
        hoveredSongIndex >= 0 && hoveredSongIndex < (int)songRects.size()) {
        SDL_Rect songR = songRects[hoveredSongIndex];
        if (x >= songR.x + songR.w - 30 && x <= songR.x + songR.w) {
            playlists[activePlaylist].songs.erase(playlists[activePlaylist].songs.begin() + hoveredSongIndex);
            playlists[activePlaylist].playCounts.erase(playlists[activePlaylist].playCounts.begin() + hoveredSongIndex);
            return;
        }
    }

    if (x >= newPlaylistButton.x && x <= newPlaylistButton.x + newPlaylistButton.w &&
        y >= newPlaylistButton.y && y <= newPlaylistButton.y + newPlaylistButton.h) {
        handlePlaylistCreation();
        return;
    }

    for (size_t i = 0; i < playlists.size(); i++) {
        SDL_Rect del = playlistDeleteRects[i];
        if (x >= del.x && x <= del.x + del.w &&
            y >= del.y && y <= del.y + del.h) {
            isConfirmingDeletion = true;
            deleteCandidateIndex = static_cast<int>(i);
            return;
        }
        SDL_Rect row = playlistRects[i];
        if (x >= row.x && x <= row.x + row.w &&
            y >= row.y && y <= row.y + row.h) {
            Uint32 now = SDL_GetTicks();
            const Uint32 DBLCLICK_TIME = 400;
            if ((int)i == lastPlaylistClickIndex && (now - lastPlaylistClickTime) < DBLCLICK_TIME) {
                isRenaming = true;
                renameIndex = static_cast<int>(i);
                renameBuffer = playlists[i].name;
                SDL_StartTextInput();
            } else {
                activePlaylist = static_cast<int>(i);
            }
            lastPlaylistClickIndex = static_cast<int>(i);
            lastPlaylistClickTime = now;
            return;
        }
    }

    if (activePlaylist >= 0 && activePlaylist < (int)playlists.size()) {
        for (size_t s = 0; s < songRects.size(); s++) {
            SDL_Rect r = songRects[s];
            if (x >= r.x && x <= r.x + r.w &&
                y >= r.y && y <= r.y + r.h) {
                const std::string& path = playlists[activePlaylist].songs[s];
                if (!path.empty() && loadAudioFile(path))
                    playAudio();
                return;
            }
        }
    }

    if (y >= prevButton.y && y <= prevButton.y + prevButton.h &&
        x >= prevButton.x && x <= prevButton.x + prevButton.w) {
        if (activePlaylist >= 0 && !playlists[activePlaylist].songs.empty()) {
            int foundIndex = -1;
            for (size_t i = 0; i < playlists[activePlaylist].songs.size(); i++) {
                if (playlists[activePlaylist].songs[i] == loadedFile) {
                    foundIndex = static_cast<int>(i);
                    break;
                }
            }
            if (foundIndex <= 0) {
                if (loadAudioFile(playlists[activePlaylist].songs.back()))
                    playAudio();
            } else {
                if (loadAudioFile(playlists[activePlaylist].songs[foundIndex - 1]))
                    playAudio();
            }
        }
    }
    else if (y >= nextButton.y && y <= nextButton.y + nextButton.h &&
             x >= nextButton.x && x <= nextButton.x + nextButton.w) {
        if (activePlaylist >= 0 && !playlists[activePlaylist].songs.empty())
            playNextTrack();
    }
    else if (x >= playButton.x && x <= playButton.x + playButton.w) {
        if (!loadedFile.empty()) {
            if (playingAudio)
                stopAudio();
            else
                playAudio();
        }
    }
    else if (x >= shuffleButton.x && x <= shuffleButton.x + shuffleButton.w) {
        isShuffled = !isShuffled;
    }
    else if (x >= muteButton.x && x <= muteButton.x + muteButton.w) {
        isMuted = !isMuted;
    }
    else if (x >= rewindButton.x && x <= rewindButton.x + rewindButton.w) {
        seekTo(currentTime - 10.0);
    }
    else if (x >= forwardButton.x && x <= forwardButton.x + forwardButton.w) {
        seekTo(currentTime + 10.0);
    }
    else if (x >= volumeBar.x && x <= volumeBar.x + volumeBar.w) {
        volume = ((float)(x - volumeBar.x) / volumeBar.w) * 100.0f;
        if (volume < 0) volume = 0;
        if (volume > 100) volume = 100;
    }
}


void Player::shutdown() {
    // CHANGE #1: Lock the audio mutex so the callback can't run while we free resources.
    std::lock_guard<std::mutex> lock(audioMutex);

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
        packet = nullptr;
    }
    if (frame) {
        av_frame_free(&frame);
        frame = nullptr;
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
    TTF_Quit();
    IMG_Quit();
    SDL_Quit();

    std::cout << "Player shutdown.\n";
    running = false;
}

bool Player::isRunning() const {
    return running;
}

void Player::handleFileDrop(const char* filePath) {
    std::lock_guard<std::mutex> lock(playlistMutex);
    if (activePlaylist >= 0) {
        playlists[activePlaylist].songs.push_back(filePath);
        playlists[activePlaylist].playCounts.push_back(0);
        if (playlists[activePlaylist].songs.size() == 1) {
            if (loadAudioFile(filePath))
                playAudio();
        }
    }
}
