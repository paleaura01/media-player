// player_playlists.cpp
#include "player.h"
#include <fstream>
#include <iostream>

// Creates a new playlist and makes it active
void Player::handlePlaylistCreation() {
    std::string name = "Playlist " + std::to_string(playlists.size() + 1);
    playlists.push_back({ name, {} });
    activePlaylist = static_cast<int>(playlists.size()) - 1;
}

// Saves all playlists to "playlists.dat"
void Player::savePlaylistState() {
    std::ofstream file("playlists.dat");
    if (!file) return;

    for (const auto& playlist : playlists) {
        file << playlist.name << "\n";
        file << playlist.songs.size() << "\n";
        for (const auto& song : playlist.songs) {
            file << song << "\n";
        }
    }
}

// Loads playlists from "playlists.dat"
void Player::loadPlaylistState() {
    std::ifstream file("playlists.dat");
    if (!file) return;

    playlists.clear();
    std::string line;
    while (std::getline(file, line)) {
        Playlist playlist;
        playlist.name = line;
        
        size_t songCount = 0;
        file >> songCount;
        file.ignore();

        for (size_t i = 0; i < songCount; i++) {
            std::string song;
            std::getline(file, song);
            playlist.songs.push_back(song);
        }
        playlists.push_back(playlist);
    }
}

// Called after loading a file to figure out total track length
void Player::calculateSongDuration() {
    if (fmtCtx && audioStreamIndex >= 0) {
        int64_t duration = fmtCtx->duration;
        if (duration <= INT64_MAX - 5000) {
            duration += 5000;
        }
        totalDuration = static_cast<double>(duration) / AV_TIME_BASE;
    }
}

// (Optional) if you want separate logic for file drops, call here from update()
void Player::handleFileDrop(const char* filePath) {
    if (activePlaylist >= 0) {
        playlists[activePlaylist].songs.push_back(filePath);
        
        // Load first song automatically
        if (playlists[activePlaylist].songs.size() == 1) {
            if (loadAudioFile(filePath)) {
                calculateSongDuration();
            }
        }
    }
}

void Player::handleMouseClick(int x, int y) {
    // 1) Check the “New Playlist” button
    if (x >= newPlaylistButton.x && x <= newPlaylistButton.x + newPlaylistButton.w &&
        y >= newPlaylistButton.y && y <= newPlaylistButton.y + newPlaylistButton.h) 
    {
        handlePlaylistCreation();
        return;
    }

    // 2) Check if we clicked a playlist row or "X"
    for (size_t i = 0; i < playlists.size(); i++) {
        SDL_Rect del = playlistDeleteRects[i];
        if (x >= del.x && x <= del.x + del.w &&
            y >= del.y && y <= del.y + del.h)
        {
            // Deleting a playlist
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

        // Check if we clicked the main playlist row
        SDL_Rect row = playlistRects[i];
        if (x >= row.x && x <= row.x + row.w &&
            y >= row.y && y <= row.y + row.h)
        {
            activePlaylist = (int)i;
            return;
        }
    }

    // 3) Check if we clicked a song in the active playlist
    if (activePlaylist >= 0 && activePlaylist < (int)playlists.size()) {
        // We have some songs, let's see if we clicked on them
        for (size_t songIndex = 0; songIndex < songRects.size(); songIndex++) {
            SDL_Rect sRect = songRects[songIndex];
            if (x >= sRect.x && x <= sRect.x + sRect.w &&
                y >= sRect.y && y <= sRect.y + sRect.h)
            {
                // We clicked on this song
                const std::string& path = playlists[activePlaylist].songs[songIndex];
                if (loadAudioFile(path)) {
                    playAudio();
                }
                return;
            }
        }
    }

    // 4) Check if we clicked playback controls, etc.
    if (y >= prevButton.y && y <= prevButton.y + prevButton.h) {
        if (x >= prevButton.x && x <= prevButton.x + prevButton.w) {
            // Prev track
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
        }
    }
}