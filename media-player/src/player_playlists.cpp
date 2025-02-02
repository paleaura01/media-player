// player_playlists.cpp

#include "player.h"
#include <fstream>
#include <iostream>

// Creates a new playlist
void Player::handlePlaylistCreation() {
    std::string name = "Playlist " + std::to_string(playlists.size() + 1);
    playlists.push_back({ name, {} });
    activePlaylist = (int)playlists.size() - 1;
}

// Save playlists
void Player::savePlaylistState() {
    std::ofstream file("playlists.dat");
    if (!file) return;

    for (auto& pl : playlists) {
        file << pl.name << "\n";
        file << pl.songs.size() << "\n";
        for (auto& song : pl.songs) {
            file << song << "\n";
        }
    }
}

// Load playlists
void Player::loadPlaylistState() {
    std::ifstream file("playlists.dat");
    if (!file) return;

    playlists.clear();
    std::string line;
    while (std::getline(file, line)) {
        Playlist p;
        p.name = line;
        size_t count = 0;
        file >> count;
        file.ignore();
        for (size_t i=0; i<count; i++) {
            std::string s;
            std::getline(file, s);
            p.songs.push_back(s);
        }
        playlists.push_back(p);
    }
}

void Player::calculateSongDuration() {
    if (fmtCtx && audioStreamIndex >= 0) {
        int64_t duration = fmtCtx->duration;
        if (duration <= INT64_MAX - 5000) {
            duration += 5000;
        }
        totalDuration = (double)duration / AV_TIME_BASE;
    }
}

void Player::handleFileDrop(const char* filePath) {
    if (activePlaylist >= 0) {
        playlists[activePlaylist].songs.push_back(filePath);
        // If first song, load immediately
        if (playlists[activePlaylist].songs.size() == 1) {
            if (loadAudioFile(filePath)) {
                calculateSongDuration();
            }
        }
    }
}

// handleMouseClick with double-click for rename
void Player::handleMouseClick(int x, int y) {
    // (1) New Playlist button?
    if (x >= newPlaylistButton.x && x <= newPlaylistButton.x + newPlaylistButton.w &&
        y >= newPlaylistButton.y && y <= newPlaylistButton.y + newPlaylistButton.h)
    {
        handlePlaylistCreation();
        return;
    }

    // (2) Check if clicked a playlist row or "X"
    for (size_t i = 0; i < playlists.size(); i++) {
        // Delete "X" button
        SDL_Rect del = playlistDeleteRects[i];
        if (x >= del.x && x <= del.x + del.w &&
            y >= del.y && y <= del.y + del.h)
        {
            // Delete playlist
            playlists.erase(playlists.begin() + i);
            playlistRects.erase(playlistRects.begin() + i);
            playlistDeleteRects.erase(playlistDeleteRects.begin() + i);

            if ((int)i == activePlaylist) {
                activePlaylist = -1;
            } else if ((int)i < activePlaylist) {
                activePlaylist--;
            }
            return;
        }

        // Main row
        SDL_Rect row = playlistRects[i];
        if (x >= row.x && x <= row.x + row.w &&
            y >= row.y && y <= row.y + row.h)
        {
            // Check double-click
            Uint32 now = SDL_GetTicks();
            const Uint32 DBLCLICK_TIME = 400; // ms

            if ((int)i == lastPlaylistClickIndex &&
                (now - lastPlaylistClickTime) < DBLCLICK_TIME)
            {
                // Double-click => inline rename
                std::cout << "[Debug] Double-click => Start rename for " 
                          << playlists[i].name << "\n";
                isRenaming   = true;
                renameIndex  = (int)i;
                renameBuffer = playlists[i].name;
                SDL_StartTextInput();
            }
            else {
                // Single-click => activate
                std::cout << "[Debug] Single-click => activate " 
                          << playlists[i].name << "\n";
                activePlaylist = (int)i;
            }
            // Update double-click memory
            lastPlaylistClickIndex = (int)i;
            lastPlaylistClickTime  = now;
            return;
        }
    }

    // (3) Check if clicked a song
    if (activePlaylist >= 0 && activePlaylist < (int)playlists.size()) {
        for (size_t s = 0; s < songRects.size(); s++) {
            SDL_Rect r = songRects[s];
            if (x >= r.x && x <= r.x + r.w &&
                y >= r.y && y <= r.y + r.h)
            {
                // clicked a song
                const std::string& path = playlists[activePlaylist].songs[s];
                if (loadAudioFile(path)) {
                    playAudio();
                }
                return;
            }
        }
    }

    // (4) Check if clicked transport controls, e.g. stop, play, etc.
    if (y >= prevButton.y && y <= prevButton.y + prevButton.h) {
        if (x >= prevButton.x && x <= prevButton.x + prevButton.w) {
            // ...
        } else if (x >= playButton.x && x <= playButton.x + playButton.w) {
            if (!loadedFile.empty()) playAudio();
        } else if (x >= nextButton.x && x <= nextButton.x + nextButton.w) {
            // ...
        } else if (x >= stopButton.x && x <= stopButton.x + stopButton.w) {
            stopAudio();
        } else if (x >= shuffleButton.x && x <= shuffleButton.x + shuffleButton.w) {
            isShuffled = !isShuffled;
        } else if (x >= muteButton.x && x <= muteButton.x + muteButton.w) {
            isMuted = !isMuted;
        }
    }
}
