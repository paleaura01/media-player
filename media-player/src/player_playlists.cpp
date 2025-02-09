#include "player.h"
#include <fstream>
#include <iostream>
#include <limits>
#include <mutex>
#include <algorithm>
#include <cctype>
#include <locale>

// Helper function to trim whitespace from both ends of a string
static inline std::string trim(const std::string &s) {
    auto start = s.begin();
    while (start != s.end() && std::isspace(static_cast<unsigned char>(*start))) {
        start++;
    }
    auto end = s.end();
    if (start != s.end()) {
        do {
            end--;
        } while (std::distance(start, end) > 0 && std::isspace(static_cast<unsigned char>(*end)));
    }
    return std::string(start, end + 1);
}

void Player::handlePlaylistCreation() {
    std::string name = "Playlist " + std::to_string(playlists.size() + 1);
    Playlist newPlaylist;
    newPlaylist.name = name;
    newPlaylist.songs = std::vector<std::string>();
    newPlaylist.playCounts = std::vector<int>();
    newPlaylist.progressTimes = std::vector<double>();
    playlists.push_back(newPlaylist);
    activePlaylist = static_cast<int>(playlists.size()) - 1;
}

void Player::savePlaylistState() {
    std::ofstream file("playlists.dat");
    if (!file) {
        SDL_ShowSimpleMessageBox(SDL_MESSAGEBOX_ERROR, "File Save Error", "Unable to open playlists.dat for writing.", nullptr);
        return;
    }
    // Save each playlist.
    for (auto& pl : playlists) {
        file << pl.name << "\n";
        file << pl.songs.size() << "\n";
        // For each song, save in the format "progress:filepath"
        for (size_t i = 0; i < pl.songs.size(); i++) {
            double prog = 0.0;
            if (i < pl.progressTimes.size())
                prog = pl.progressTimes[i];
            std::string path = pl.songs[i];
            std::replace(path.begin(), path.end(), '\\', '/'); // normalize separators
            file << prog << ":" << path << "\n";
        }
        for (auto count : pl.playCounts) {
            file << count << "\n";
        }
    }
    file << "LastTime=" << currentTime << ";File=" << loadedFile << "\n";
}

void Player::loadPlaylistState() {
    std::ifstream file("playlists.dat");
    if (!file) {
        SDL_ShowSimpleMessageBox(SDL_MESSAGEBOX_WARNING, "File Load Warning", "playlists.dat not found.", nullptr);
        return;
    }
    playlists.clear();
    std::string line;
    while (std::getline(file, line)) {
        line = trim(line);
        if (line.empty()) continue;
        if (line.substr(0,9) == "LastTime=") {
            size_t pos = line.find(";File=");
            if (pos != std::string::npos) {
                try {
                    lastPlayedTime = std::stod(trim(line.substr(9, pos - 9)));
                    loadedFile = trim(line.substr(pos + 6));
                    std::cout << "[Debug] Loaded lastPlayedTime: " << lastPlayedTime 
                              << ", loadedFile: " << loadedFile << std::endl;
                } catch (...) {
                    SDL_ShowSimpleMessageBox(SDL_MESSAGEBOX_ERROR, "Playlist Parsing Error",
                        "Error parsing LastTime marker.", nullptr);
                    lastPlayedTime = 0.0;
                    loadedFile = "";
                }
            } else {
                SDL_ShowSimpleMessageBox(SDL_MESSAGEBOX_ERROR, "Playlist Format Error",
                    "LastTime marker is not in the correct format.", nullptr);
            }
            break;
        }
        Playlist p;
        p.name = line;
        if (!std::getline(file, line))
            break;
        line = trim(line);
        size_t count = 0;
        try {
            count = std::stoul(line);
        } catch (...) {
            count = 0;
        }
        for (size_t i = 0; i < count; i++) {
            if (!std::getline(file, line))
                break;
            line = trim(line);
            size_t pos = line.find(':');
            if (pos != std::string::npos) {
                try {
                    double prog = std::stod(trim(line.substr(0, pos)));
                    std::string songPath = trim(line.substr(pos + 1));
                    p.progressTimes.push_back(prog);
                    p.songs.push_back(songPath);
                } catch (...) {
                    p.progressTimes.push_back(0.0);
                    p.songs.push_back(line);
                }
            } else {
                p.progressTimes.push_back(0.0);
                p.songs.push_back(line);
            }
        }
        for (size_t i = 0; i < count; i++) {
            if (!std::getline(file, line))
                break;
            line = trim(line);
            try {
                int cnt = std::stoi(line);
                p.playCounts.push_back(cnt);
            } catch (...) {
                p.playCounts.push_back(0);
            }
        }
        playlists.push_back(p);
    }
    activePlaylist = -1;
    if (!loadedFile.empty()) {
        for (size_t p = 0; p < playlists.size(); p++) {
            for (size_t s = 0; s < playlists[p].songs.size(); s++) {
                if (playlists[p].songs[s] == loadedFile) {
                    activePlaylist = static_cast<int>(p);
                    break;
                }
            }
            if (activePlaylist != -1)
                break;
        }
    }
    if (activePlaylist == -1 && !loadedFile.empty() && !playlists.empty()) {
        activePlaylist = 0;
    }
    if (!loadedFile.empty()) {
        std::ifstream checkFile(loadedFile);
        if (!checkFile.good()) {
            std::cout << "[Warning] Last played file not found: " << loadedFile << std::endl;
            loadedFile = "";
            lastPlayedTime = 0.0;
        }
    }
}

void Player::handleFileDrop(const char* filePath) {
    std::lock_guard<std::mutex> lock(playlistMutex);
    if (activePlaylist >= 0) {
        playlists[activePlaylist].songs.push_back(filePath);
        playlists[activePlaylist].playCounts.push_back(0);
        playlists[activePlaylist].progressTimes.push_back(0.0);
        if (playlists[activePlaylist].songs.size() == 1) {
            if (!loadAudioFile(filePath))
                SDL_ShowSimpleMessageBox(SDL_MESSAGEBOX_ERROR, "File Drop Error", "Failed to load dropped file.", window);
            else
                playAudio();
        }
    }
}
