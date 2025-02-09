// player_playlists.cpp
#include "player.h"
#include <fstream>
#include <iostream>
#include <limits>
#include <mutex>

void Player::handlePlaylistCreation() {
    std::string name = "Playlist " + std::to_string(playlists.size() + 1);
    Playlist newPlaylist;
    newPlaylist.name = name;
    playlists.push_back(newPlaylist);
    activePlaylist = static_cast<int>(playlists.size()) - 1;
}

void Player::savePlaylistState() {
    std::ofstream file("playlists.dat");
    if (!file) return;
    for (auto& pl : playlists) {
        file << pl.name << "\n";
        file << pl.songs.size() << "\n";
        for (auto& song : pl.songs) {
            file << song << "\n";
        }
        for (auto count : pl.playCounts) {
            file << count << "\n";
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
        file.ignore(std::numeric_limits<std::streamsize>::max(), '\n');
        for (size_t i = 0; i < count; i++) {
            std::string s;
            std::getline(file, s);
            p.songs.push_back(s);
        }
        for (size_t i = 0; i < count; i++) {
            int c;
            file >> c;
            file.ignore(std::numeric_limits<std::streamsize>::max(), '\n');
            p.playCounts.push_back(c);
        }
        playlists.push_back(p);
    }
}
