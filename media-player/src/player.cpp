#include "player.h"
#include <iostream>
#include <cmath> // llround
#include <cstdlib> // for srand, rand

Player::Player()

    : currentPlayLevel(0), running(true), window(nullptr), renderer(nullptr), font(nullptr),
      fmtCtx(nullptr), codecCtx(nullptr), swrCtx(nullptr),
      audioStreamIndex(-1), packet(nullptr), frame(nullptr),
      audioBuffer(nullptr), audioBufferSize(0), audioBufferIndex(0),
      audioDev(0), playingAudio(false), loadedFile(""),
      currentTime(0), totalDuration(0), isMuted(false), isShuffled(false),
      activePlaylist(-1),
      isConfirmingDeletion(false), deleteCandidateIndex(-1)
{
    timeBar = { 10, 10, 780, 20 };
    // Transport buttons
    prevButton    = { 10,  40, 40, 40 };
    playButton    = { 55,  40, 80, 40 };
    nextButton    = { 140, 40, 40, 40 };
    shuffleButton = { 235, 40, 80, 40 };
    muteButton    = { 320, 40, 80, 40 };
    rewindButton  = { 405, 40, 40, 40 };
    forwardButton = { 450, 40, 40, 40 };
    volumeBar     = { 545, 40, 240, 40 };

    // Panels
    playlistPanel = { 0,  90, 200, 510 };
    libraryPanel  = { 200, 90, 600, 510 };
    newPlaylistButton = { 
        playlistPanel.x + 10,
        playlistPanel.y + 10,
        playlistPanel.w - 20,
        30
    };
    mainPanel = {0,0,0,0};

    // Confirm Deletion
    confirmDialogRect = { 250,200,300,150 };
    confirmYesButton  = { 280,300,100,30 };
    confirmNoButton   = { 420,300,100,30 };
}

Player::~Player() {
    savePlaybackState();
    savePlaylistState();
    shutdown();
}

bool Player::init() {
    loadPlaylistState();  // Load playlists first

    if (SDL_Init(SDL_INIT_VIDEO | SDL_INIT_AUDIO) != 0) {
        std::cerr << "SDL_Init Error: " << SDL_GetError() << std::endl;
        return false;
    }
    if (TTF_Init() != 0) {
        std::cerr << "TTF_Init Error: " << TTF_GetError() << std::endl;
        return false;
    }

    font = TTF_OpenFont("Arial.ttf", 14);
    if (!font) {
        std::cerr << "TTF_OpenFont Error: " << TTF_GetError() << std::endl;
        return false;
    }

    window = SDL_CreateWindow("Barebones Audio Player",
                            SDL_WINDOWPOS_CENTERED, SDL_WINDOWPOS_CENTERED,
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

    packet = av_packet_alloc();
    frame = av_frame_alloc();
    if (!packet || !frame) {
        std::cerr << "Failed to allocate FFmpeg packet/frame." << std::endl;
        return false;
    }

    SDL_EventState(SDL_DROPFILE, SDL_ENABLE);
    
    // Seed random generator
    srand(static_cast<unsigned>(SDL_GetTicks())); 

    // Now that everything is initialized, load the playback state
    loadPlaybackState();  // Move this here, after all initialization

    std::cout << "Initialization successful.\n";
    return true;
}

void Player::shuffleCurrentPlaylist() {
    if (activePlaylist < 0 || activePlaylist >= (int)playlists.size()) return;
    
    auto& songs = playlists[activePlaylist].songs;
    auto& counts = playlists[activePlaylist].playCounts;
    auto& sessionCounts = sessionPlayCounts;

    // Create indices array
    std::vector<size_t> indices(songs.size());
    for (size_t i = 0; i < indices.size(); i++) {
        indices[i] = i;
    }
    
    // Fisher-Yates shuffle
    for (size_t i = indices.size() - 1; i > 0; i--) {
        size_t j = rand() % (i + 1);
        std::swap(indices[i], indices[j]);
    }
    
    // Apply shuffle using temporary vectors
    std::vector<std::string> tempSongs = songs;
    std::vector<int> tempCounts = counts;
    std::vector<int> tempSessionCounts = sessionCounts;
    
    for (size_t i = 0; i < indices.size(); i++) {
        songs[i] = tempSongs[indices[i]];
        counts[i] = tempCounts[indices[i]];
        sessionCounts[i] = tempSessionCounts[indices[i]];
    }
}

void Player::update() {
    static Uint32 lastSaveTime = 0;
    Uint32 currentTicks = SDL_GetTicks();
    
    // Save playback state every 5 seconds if playing
    if (playingAudio && currentTicks - lastSaveTime > 5000) {
        savePlaybackState();
        lastSaveTime = currentTicks;
    }

    // Poll SDL events
    SDL_Event event;
    while (SDL_PollEvent(&event)) {
        if (event.type == SDL_QUIT) {
            running = false;
        }
        else if (event.type == SDL_MOUSEWHEEL) {
            int mouseX, mouseY;
            SDL_GetMouseState(&mouseX, &mouseY);
            
            // Check if mouse is over song panel
            if (mouseX >= libraryPanel.x && mouseX <= libraryPanel.x + libraryPanel.w &&
                mouseY >= libraryPanel.y && mouseY <= libraryPanel.y + libraryPanel.h) 
            {
                songScrollOffset -= event.wheel.y;
                
                // Clamp scrolling
                if (activePlaylist >= 0) {
                    int maxOffset = (int)playlists[activePlaylist].songs.size() - visibleSongRows;
                    if (maxOffset < 0) maxOffset = 0;
                    if (songScrollOffset < 0) songScrollOffset = 0;
                    if (songScrollOffset > maxOffset) songScrollOffset = maxOffset;
                }
            }
        }
        else if (event.type == SDL_MOUSEBUTTONDOWN) {
            handleMouseClick(event.button.x, event.button.y);
        }
        else if (event.type == SDL_DROPFILE) {
            handleFileDrop(event.drop.file);
            SDL_free(event.drop.file);
        }
        else if (event.type == SDL_TEXTINPUT && isRenaming) {
            renameBuffer += event.text.text;
        }
        else if (event.type == SDL_KEYDOWN && isRenaming) {
            if (event.key.keysym.sym == SDLK_RETURN) {
                if (renameIndex >= 0 && renameIndex < (int)playlists.size()) {
                    playlists[renameIndex].name = renameBuffer;
                }
                isRenaming = false;
                SDL_StopTextInput();
            }
            else if (event.key.keysym.sym == SDLK_BACKSPACE && !renameBuffer.empty()) {
                renameBuffer.pop_back();
            }
            else if (event.key.keysym.sym == SDLK_ESCAPE) {
                isRenaming = false;
                SDL_StopTextInput();
            }
        }
        if (event.type == SDL_MOUSEMOTION) {
            if (activePlaylist >= 0) {
                hoveredSongIndex = -1;  // Reset first
                for (size_t i = 0; i < songRects.size(); i++) {
                    if (event.motion.x >= songRects[i].x &&
                        event.motion.x <= songRects[i].x + songRects[i].w &&
                        event.motion.y >= songRects[i].y &&
                        event.motion.y <= songRects[i].y + songRects[i].h)
                    {
                        hoveredSongIndex = i + songScrollOffset;  // Add scroll offset
                        break;
                    }
                }
            }
        }
    }

    // If audio is playing, check for track finish
    if (playingAudio) {
        currentTime = lastPTS.load(std::memory_order_relaxed);

        // If track is finished
        if ((currentTime >= totalDuration && totalDuration > 0) ||
            reachedEOF.load(std::memory_order_relaxed))
        {
            std::cout << "[Debug] Track finished. Moving to next song...\n";
            
            // 1) Increment counts for the track that truly finished
            incrementFinishedTrack();

            // 2) Then pick next track
            playNextTrack();

            reachedEOF.store(false, std::memory_order_relaxed);
        }
    }

    // UI Rendering
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

//
// 1) We only call this when the track actually finishes.
//
void Player::incrementFinishedTrack() {
    // If there's no loaded file, or no active playlist, do nothing
    if (loadedFile.empty() || activePlaylist < 0
        || activePlaylist >= (int)playlists.size()) 
    {
        return;
    }

    // Which index was that file in the playlist?
    auto &pl = playlists[activePlaylist];
    for (size_t i = 0; i < pl.songs.size(); i++) {
        if (pl.songs[i] == loadedFile) {
            // Overall count
            pl.playCounts[i]++;
            // Per-session count
            if (i < sessionPlayCounts.size()) {
                sessionPlayCounts[i]++;
            }
            savePlaylistState();
            break;
        }
    }
}

//
// 2) This picks from unplayed tracks only, resetting if needed.
//    No increment logic here—only selection.
//
void Player::playNextTrack() {
    std::lock_guard<std::mutex> lock(playlistMutex);

    if (activePlaylist < 0 || activePlaylist >= (int)playlists.size() ||
        playlists[activePlaylist].songs.empty())
    {
        return;
    }

    auto &pl = playlists[activePlaylist];
    sessionPlayCounts.resize(pl.songs.size(), 0);

    size_t nextIndex;
    if (isShuffled) {
        // Keep random selection for shuffle mode
        std::vector<size_t> unplayed;
        for (size_t i = 0; i < pl.songs.size(); i++) {
            if (sessionPlayCounts[i] == 0) {
                unplayed.push_back(i);
            }
        }
        if (unplayed.empty()) {
            for (size_t i = 0; i < sessionPlayCounts.size(); i++) {
                sessionPlayCounts[i] = 0;
            }
            unplayed = std::vector<size_t>(pl.songs.size());
            for (size_t i = 0; i < pl.songs.size(); i++) {
                unplayed[i] = i;
            }
        }
        nextIndex = unplayed[rand() % unplayed.size()];
    } else {
        // Find current track and play next one
        size_t currentIndex = 0;
        for (size_t i = 0; i < pl.songs.size(); i++) {
            if (pl.songs[i] == loadedFile) {
                currentIndex = i;
                break;
            }
        }
        nextIndex = (currentIndex + 1) % pl.songs.size();
    }

    if (loadAudioFile(pl.songs[nextIndex])) {
        playAudio();
    }
}

//
// 3) Called when user clicks a song in handleMouseClick() to ensure
//    we obey the "one round" rule. If the track is already played
//    and there are still unplayed tracks left, return false.
//
bool Player::canPlayThisTrack(size_t index) {
    // If this track is unplayed, fine
    if (sessionPlayCounts[index] == 0) {
        return true;
    }
    // Otherwise, check if ANY track is still unplayed
    for (size_t i = 0; i < sessionPlayCounts.size(); i++) {
        if (sessionPlayCounts[i] == 0) {
            // Means we do have some unplayed track left,
            // so we can't replay this one yet.
            return false;
        }
    }
    // If we get here, everything was played, so reset them all
    for (size_t i = 0; i < sessionPlayCounts.size(); i++) {
        sessionPlayCounts[i] = 0;
    }
    return true; // now it's allowed
}

void Player::seekTo(double seconds) {
    std::lock_guard<std::mutex> lock(audioMutex);
    
    if (!fmtCtx || audioStreamIndex < 0 || isLoading.load()) {
        return;
    }

    if (seconds < 0) seconds = 0;
    if (seconds > totalDuration) seconds = totalDuration;

    try {
        int64_t target = (int64_t)llround(seconds * AV_TIME_BASE);
        if (av_seek_frame(fmtCtx, -1, target, AVSEEK_FLAG_BACKWARD) >= 0) {
            avcodec_flush_buffers(codecCtx);
            currentTime = seconds;
            lastPTS.store(seconds, std::memory_order_relaxed);
        }
    } catch (...) {
        std::cerr << "Error during seek operation" << std::endl;
    }
}

void Player::shutdown() {
    if (!loadedFile.empty()) {
        savePlaybackState();  // Save before cleanup
    }
    
    std::lock_guard<std::mutex> lock1(audioMutex);
    std::lock_guard<std::mutex> lock2(playlistMutex);
    
    running = false;
    cleanup_audio_resources();
    
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
    SDL_Quit();
}

bool Player::isRunning() const {
    return running;
}

void Player::savePlaybackState() {
    std::ofstream file("playback_state.dat");
    if (!file) return;
    
    file << loadedFile << "\n";
    file << currentTime << "\n";
    file << activePlaylist << "\n";
}

void Player::loadPlaybackState() {
    std::ifstream file("playback_state.dat");
    if (!file) return;
    
    std::string savedFile;
    double savedTime;
    int savedPlaylist;
    
    std::getline(file, savedFile);
    file >> savedTime;
    file >> savedPlaylist;
    
    if (!savedFile.empty() && savedTime >= 0) {
        if (savedPlaylist >= 0 && savedPlaylist < (int)playlists.size()) {
            activePlaylist = savedPlaylist;
            sessionPlayCounts.resize(playlists[activePlaylist].songs.size(), 0);
            currentPlayLevel = 0;
        }
        
        if (loadAudioFile(savedFile)) {
            SDL_Delay(100);  // Give audio system time to initialize
            seekTo(savedTime);
            playAudio();
        }
    }
}
