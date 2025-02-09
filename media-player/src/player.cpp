#ifdef _WIN32
#include <windows.h>
#include <string>
// Helper function to load an ICO file and convert it into an SDL_Surface.
// Renamed to avoid overloading conflicts.
SDL_Surface* LoadIconFromICOHelper(const char* path) {
    int len = MultiByteToWideChar(CP_UTF8, 0, path, -1, NULL, 0);
    if (len == 0) return nullptr;
    std::wstring wpath(len, L'\0');
    MultiByteToWideChar(CP_UTF8, 0, path, -1, &wpath[0], len);
    HICON hIcon = (HICON)LoadImageW(NULL, wpath.c_str(), IMAGE_ICON, 32, 32, LR_LOADFROMFILE);
    if (!hIcon) {
        return nullptr;
    }
    ICONINFO iconInfo;
    if (!GetIconInfo(hIcon, &iconInfo)) {
        DestroyIcon(hIcon);
        return nullptr;
    }
    BITMAP bmp;
    GetObject(iconInfo.hbmColor, sizeof(BITMAP), &bmp);
    int width = bmp.bmWidth;
    int height = bmp.bmHeight;
    SDL_Surface* surface = SDL_CreateRGBSurface(0, width, height, 32,
        0x00FF0000, 0x0000FF00, 0x000000FF, 0xFF000000);
    if (!surface) {
        DeleteObject(iconInfo.hbmColor);
        DeleteObject(iconInfo.hbmMask);
        DestroyIcon(hIcon);
        return nullptr;
    }
    BITMAPINFO bmi;
    ZeroMemory(&bmi, sizeof(bmi));
    bmi.bmiHeader.biSize = sizeof(bmi.bmiHeader);
    bmi.bmiHeader.biWidth = width;
    bmi.bmiHeader.biHeight = -height;
    bmi.bmiHeader.biPlanes = 1;
    bmi.bmiHeader.biBitCount = 32;
    bmi.bmiHeader.biCompression = BI_RGB;
    HDC hdc = GetDC(NULL);
    GetDIBits(hdc, iconInfo.hbmColor, 0, height, surface->pixels, &bmi, DIB_RGB_COLORS);
    ReleaseDC(NULL, hdc);
    DeleteObject(iconInfo.hbmColor);
    DeleteObject(iconInfo.hbmMask);
    DestroyIcon(hIcon);
    return surface;
}
#endif

#include "player.h"
#include <iostream>
#include <cmath>
#include <cstdlib>
#include <algorithm>
#include <fstream>

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
      activePlaylist(-1), shuffleIndex(0), lastPlayedTime(0.0)
{
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

    confirmDialogRect = { 250,200,300,150 };
    confirmYesButton  = { 280,300,100,30 };
    confirmNoButton   = { 420,300,100,30 };
}

Player::~Player() {
    savePlaylistState();
    shutdown();
}

bool Player::init() {
    loadPlaylistState();

    if (SDL_Init(SDL_INIT_VIDEO | SDL_INIT_AUDIO) != 0) {
        std::cerr << "SDL_Init Error: " << SDL_GetError() << std::endl;
        SDL_ShowSimpleMessageBox(SDL_MESSAGEBOX_ERROR, "SDL Init Error", SDL_GetError(), nullptr);
        return false;
    }
    if (TTF_Init() != 0) {
        std::cerr << "TTF_Init Error: " << TTF_GetError() << std::endl;
        SDL_ShowSimpleMessageBox(SDL_MESSAGEBOX_ERROR, "TTF Init Error", TTF_GetError(), nullptr);
        return false;
    }
 
    window = SDL_CreateWindow("Twink Audio Player",
                              SDL_WINDOWPOS_CENTERED, SDL_WINDOWPOS_CENTERED,
                              800, 600, SDL_WINDOW_SHOWN);
    if (!window) {
        std::cerr << "SDL_CreateWindow Error: " << SDL_GetError() << std::endl;
        SDL_ShowSimpleMessageBox(SDL_MESSAGEBOX_ERROR, "Window Creation Error", SDL_GetError(), nullptr);
        return false;
    }
    {
        char* basePath = SDL_GetBasePath();
        std::string base;
        if (basePath) {
            base = std::string(basePath);
            SDL_free(basePath);
        } else {
            base = "";
        }
        size_t pos = base.rfind("dist");
        if (pos != std::string::npos) {
            base = base.substr(0, pos);
        }
        std::string iconPath = base + "assets\\icon.png";
        std::cout << "[Debug] Loading icon from: " << iconPath << "\n";
        SDL_Surface* icon = IMG_Load(iconPath.c_str());
#ifdef _WIN32
        if (!icon) {
            std::cerr << "[Debug] PNG icon not found or failed to load, attempting ICO fallback.\n";
            std::string icoPath = base + "assets\\icon.ico";
            icon = LoadIconFromICOHelper(icoPath.c_str());
        }
#endif
        if (icon) {
            SDL_SetWindowIcon(window, icon);
            SDL_FreeSurface(icon);
        } else {
            std::string err = std::string("Failed to load icon: ") + IMG_GetError();
            std::cerr << err << std::endl;
            SDL_ShowSimpleMessageBox(SDL_MESSAGEBOX_ERROR, "Icon Load Error", ("Failed to load icon from: " + iconPath).c_str(), window);
        }
    }
    renderer = SDL_CreateRenderer(window, -1, SDL_RENDERER_ACCELERATED);
    if (!renderer) {
        std::cerr << "SDL_CreateRenderer Error: " << SDL_GetError() << std::endl;
        SDL_ShowSimpleMessageBox(SDL_MESSAGEBOX_ERROR, "Renderer Creation Error", SDL_GetError(), window);
        return false;
    }
    font = TTF_OpenFont("Arial.ttf", 12);
    if (!font) {
        font = TTF_OpenFont("C:\\Windows\\Fonts\\arial.ttf", 12);
        if (!font) {
            std::cerr << "TTF_OpenFont Error: " << TTF_GetError() << std::endl;
            SDL_ShowSimpleMessageBox(SDL_MESSAGEBOX_ERROR, "Font Load Error", TTF_GetError(), window);
            return false;
        }
    }
    TTF_SetFontStyle(font, TTF_STYLE_BOLD);

    packet = av_packet_alloc();
    frame  = av_frame_alloc();
    if (!packet || !frame) {
        std::cerr << "Failed to allocate FFmpeg packet/frame." << std::endl;
        SDL_ShowSimpleMessageBox(SDL_MESSAGEBOX_ERROR, "FFmpeg Allocation Error", "Could not allocate packet or frame.", window);
        return false;
    }
    SDL_EventState(SDL_DROPFILE, SDL_ENABLE);
    srand(static_cast<unsigned>(SDL_GetTicks()));

    // Check if last played file exists
    if (!loadedFile.empty()) {
        std::ifstream checkFile(loadedFile);
        if (!checkFile.good()) {
            std::cout << "[Warning] Last played file not found: " << loadedFile << std::endl;
            loadedFile = "";
            lastPlayedTime = 0.0;
        }
    }

    // Resume playback if a last played file was saved.
    if (!loadedFile.empty() && activePlaylist != -1) {
        std::cout << "[Debug] Attempting to resume: " << loadedFile 
                  << " at " << lastPlayedTime << " seconds\n";
        if (loadAudioFile(loadedFile)) {
            SDL_Delay(100);
            if (lastPlayedTime > 0 && lastPlayedTime < totalDuration) {
                seekTo(lastPlayedTime);
                SDL_Delay(100);
            }
            playAudio();
        } else {
            std::cout << "[Error] Failed to load last played file\n";
            loadedFile = "";
            lastPlayedTime = 0.0;
        }
    }

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
                    seekTo(newTime);
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
            updateProgressForCurrentSong(totalDuration);
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
            if (i < playlists[activePlaylist].progressTimes.size())
                playlists[activePlaylist].progressTimes[i] = totalDuration;
            break;
        }
    }
    savePlaylistState();
}

void Player::updateProgressForCurrentSong(double time) {
    if (activePlaylist < 0)
        return;
    if (playlists[activePlaylist].progressTimes.size() < playlists[activePlaylist].songs.size())
        playlists[activePlaylist].progressTimes.resize(playlists[activePlaylist].songs.size(), 0.0);
    for (size_t i = 0; i < playlists[activePlaylist].songs.size(); i++) {
        if (playlists[activePlaylist].songs[i] == loadedFile) {
            playlists[activePlaylist].progressTimes[i] = time;
            break;
        }
    }
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

    if (isConfirmingDeletion) {
        if (x >= confirmYesButton.x && x <= confirmYesButton.x + confirmYesButton.w &&
            y >= confirmYesButton.y && y <= confirmYesButton.y + confirmYesButton.h) {
            playlists.erase(playlists.begin() + deleteCandidateIndex);
            playlistRects.erase(playlistRects.begin() + deleteCandidateIndex);
            playlistDeleteRects.erase(playlistDeleteRects.begin() + deleteCandidateIndex);
            if (deleteCandidateIndex == activePlaylist)
                activePlaylist = -1;
            else if (deleteCandidateIndex < activePlaylist)
                activePlaylist--;
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
            // Fixed: Erase from playCounts using the correct vector iterator.
            playlists[activePlaylist].playCounts.erase(playlists[activePlaylist].playCounts.begin() + hoveredSongIndex);
            playlists[activePlaylist].progressTimes.erase(playlists[activePlaylist].progressTimes.begin() + hoveredSongIndex);
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

void Player::seekTo(double seconds) {
    std::lock_guard<std::mutex> lock(audioMutex);
    if (!fmtCtx || audioStreamIndex < 0)
        return;
    if (seconds < 0)
        seconds = 0;
    if (totalDuration > 0 && seconds > totalDuration)
        seconds = totalDuration;
    int64_t target = static_cast<int64_t>(llround(seconds * AV_TIME_BASE));
    if (av_seek_frame(fmtCtx, -1, target, AVSEEK_FLAG_BACKWARD) >= 0) {
        avcodec_flush_buffers(codecCtx);
        audioBufferSize = 0;
        audioBufferIndex = 0;
        currentTime = seconds;
        lastPTS.store(seconds, std::memory_order_relaxed);
        updateProgressForCurrentSong(seconds);
        std::cout << "[Debug] Seeked to " << seconds << " sec.\n";
    } else {
        std::cerr << "[Warn] av_seek_frame failed.\n";
        SDL_ShowSimpleMessageBox(SDL_MESSAGEBOX_ERROR, "Seek Error", "av_seek_frame failed.", window);
    }    
}

void Player::shutdown() {
    if (activePlaylist >= 0 && !loadedFile.empty()) {
        for (size_t i = 0; i < playlists[activePlaylist].songs.size(); i++) {
            if (playlists[activePlaylist].songs[i] == loadedFile) {
                playlists[activePlaylist].progressTimes[i] = currentTime;
                break;
            }
        }
    }
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
