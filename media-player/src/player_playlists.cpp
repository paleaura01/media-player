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
    // Handle playlist button
    if (x >= newPlaylistButton.x && x <= newPlaylistButton.x + newPlaylistButton.w &&
        y >= newPlaylistButton.y && y <= newPlaylistButton.y + newPlaylistButton.h) {
        handlePlaylistCreation();
        return;
    }
        // Handle playback controls
    if (y >= prevButton.y && y <= prevButton.y + prevButton.h) {
        if (x >= prevButton.x && x <= prevButton.x + prevButton.w) {
            // Previous track (unimplemented)
        } else if (x >= playButton.x && x <= playButton.x + playButton.w) {
            if (!loadedFile.empty()) playAudio();
        } else if (x >= nextButton.x && x <= nextButton.x + nextButton.w) {
            // Next track (unimplemented)
        } else if (x >= stopButton.x && x <= stopButton.x + stopButton.w) {
            stopAudio();
        } else if (x >= shuffleButton.x && x <= shuffleButton.x + shuffleButton.w) {
            isShuffled = !isShuffled;
        } else if (x >= muteButton.x && x <= muteButton.x + muteButton.w) {
            isMuted = !isMuted;
        }
    }
}