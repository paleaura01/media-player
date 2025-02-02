// player_playlists.cpp

#include "player.h"
#include <fstream>
#include <iostream>

void Player::handlePlaylistCreation() {
    std::string name = "Playlist " + std::to_string(playlists.size() + 1);
    playlists.push_back({ name, {} });
    activePlaylist = (int)playlists.size() - 1;
}

// Saves all playlists
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
        for (size_t i = 0; i < count; i++) {
            std::string s;
            std::getline(file, s);
            p.songs.push_back(s);
        }
        playlists.push_back(p);
    }
}

void Player::handleFileDrop(const char* filePath) {
    if (activePlaylist >= 0) {
        playlists[activePlaylist].songs.push_back(filePath);
        if (playlists[activePlaylist].songs.size() == 1) {
            if (loadAudioFile(filePath)) {
                calculateSongDuration();
            }
        }
    }
}

void Player::handleMouseClick(int x, int y) {
    // 1) If the confirmation dialog is open, only handle Yes/No
    if (isConfirmingDeletion) {
        // Check Yes button
        if (x >= confirmYesButton.x && x <= confirmYesButton.x + confirmYesButton.w &&
            y >= confirmYesButton.y && y <= confirmYesButton.y + confirmYesButton.h)
        {
            // Confirm deletion
            if (deleteCandidateIndex >= 0 && deleteCandidateIndex < (int)playlists.size()) {
                playlists.erase(playlists.begin() + deleteCandidateIndex);
                playlistRects.erase(playlistRects.begin() + deleteCandidateIndex);
                playlistDeleteRects.erase(playlistDeleteRects.begin() + deleteCandidateIndex);

                if (deleteCandidateIndex == activePlaylist) {
                    activePlaylist = -1;
                } else if (deleteCandidateIndex < activePlaylist) {
                    activePlaylist--;
                }
            }
            isConfirmingDeletion = false;
            deleteCandidateIndex  = -1;
            return;
        }
        // Check No button
        if (x >= confirmNoButton.x && x <= confirmNoButton.x + confirmNoButton.w &&
            y >= confirmNoButton.y && y <= confirmNoButton.y + confirmNoButton.h)
        {
            // Cancel
            isConfirmingDeletion = false;
            deleteCandidateIndex = -1;
            return;
        }

        // Else clicked elsewhere while dialog is open, do nothing
        return;
    }

    // 2) No dialog open -> normal logic

    // Check "New Playlist"
    if (x >= newPlaylistButton.x && x <= newPlaylistButton.x + newPlaylistButton.w &&
        y >= newPlaylistButton.y && y <= newPlaylistButton.y + newPlaylistButton.h)
    {
        handlePlaylistCreation();
        return;
    }

    // Check if we clicked a playlist row or "X"
    for (size_t i = 0; i < playlists.size(); i++) {
        // "X" button
        SDL_Rect del = playlistDeleteRects[i];
        if (x >= del.x && x <= del.x + del.w &&
            y >= del.y && y <= del.y + del.h)
        {
            // Instead of immediate deletion, prompt
            isConfirmingDeletion = true;
            deleteCandidateIndex  = (int)i;
            return;
        }

        // Main row
        SDL_Rect row = playlistRects[i];
        if (x >= row.x && x <= row.x + row.w &&
            y >= row.y && y <= row.y + row.h)
        {
            // Double-click detection
            Uint32 now = SDL_GetTicks();
            const Uint32 DBLCLICK_TIME = 400; // ms

            if ((int)i == lastPlaylistClickIndex &&
                (now - lastPlaylistClickTime) < DBLCLICK_TIME)
            {
                // Double-click => rename
                std::cout << "[Debug] Double-click => rename: " << playlists[i].name << "\n";
                isRenaming   = true;
                renameIndex  = (int)i;
                renameBuffer = playlists[i].name;
                SDL_StartTextInput();
            }
            else {
                // Single-click => activate
                std::cout << "[Debug] Single-click => activate: " << playlists[i].name << "\n";
                activePlaylist = (int)i;
            }
            lastPlaylistClickIndex = (int)i;
            lastPlaylistClickTime  = now;
            return;
        }
    }

    // 3) Check if clicked a song
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
            if (hoveredSongIndex >= 0 && 
            x >= songRects[hoveredSongIndex].x + songRects[hoveredSongIndex].w - 30 && 
            x <= songRects[hoveredSongIndex].x + songRects[hoveredSongIndex].w) {
            // Remove song
            playlists[activePlaylist].songs.erase(
                playlists[activePlaylist].songs.begin() + hoveredSongIndex
            );
            return;
        }
        }
    }

    // 4) Check transport controls
    if (y >= prevButton.y && y <= prevButton.y + prevButton.h) {
        if (x >= prevButton.x && x <= prevButton.x + prevButton.w) {
            if (activePlaylist >= 0 && !playlists[activePlaylist].songs.empty()) {
                for (size_t i = 0; i < playlists[activePlaylist].songs.size(); i++) {
                    if (playlists[activePlaylist].songs[i] == loadedFile && i > 0) {
                        if (loadAudioFile(playlists[activePlaylist].songs[i-1])) {
                            playAudio();
                        }
                        break;
                    }
                }
            }
        } else if (x >= playButton.x && x <= playButton.x + playButton.w) {
            if (!loadedFile.empty()) playAudio();
        } else if (x >= nextButton.x && x <= nextButton.x + nextButton.w) {
            if (activePlaylist >= 0 && !playlists[activePlaylist].songs.empty()) {
                for (size_t i = 0; i < playlists[activePlaylist].songs.size(); i++) {
                    if (playlists[activePlaylist].songs[i] == loadedFile && 
                        i < playlists[activePlaylist].songs.size() - 1) {
                        if (loadAudioFile(playlists[activePlaylist].songs[i+1])) {
                            playAudio();
                        }
                        break;
                    }
                }
            }
        } else if (x >= stopButton.x && x <= stopButton.x + stopButton.w) {
            stopAudio();
        } else if (x >= shuffleButton.x && x <= shuffleButton.x + shuffleButton.w) {
            isShuffled = !isShuffled;
        } else if (x >= muteButton.x && x <= muteButton.x + muteButton.w) {
            isMuted = !isMuted;
        } else if (x >= rewindButton.x && x <= rewindButton.x + rewindButton.w) {
            seekTo(currentTime - 10.0);
        } else if (x >= forwardButton.x && x <= forwardButton.x + forwardButton.w) {
            seekTo(currentTime + 10.0);
        } else if (x >= volumeBar.x && x <= volumeBar.x + volumeBar.w) {
            volume = ((float)(x - volumeBar.x) / volumeBar.w) * 100.0f;
            if (volume < 0) volume = 0;
            if (volume > 100) volume = 100;
        }
    }

    // Handle time bar clicks
    if (y >= timeBar.y && y <= timeBar.y + timeBar.h &&
        x >= timeBar.x && x <= timeBar.x + timeBar.w) 
    {
        if (totalDuration > 0.0) {
            double fraction = double(x - timeBar.x) / double(timeBar.w);
            if (fraction < 0) fraction = 0;
            if (fraction > 1) fraction = 1;
            double newTime = fraction * totalDuration;
            seekTo(newTime);
        }
        
        return;
    }
}