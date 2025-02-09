// player_playlists.cpp
#include "player.h"
#include <fstream>
#include <iostream>
#include <mutex>
#include <nlohmann/json.hpp>

using json = nlohmann::json;

void Player::handlePlaylistCreation() {
    std::string name = "Playlist " + std::to_string(playlists.size() + 1);
    Playlist newPlaylist;
    newPlaylist.name = name;
    playlists.push_back(newPlaylist);
    activePlaylist = static_cast<int>(playlists.size()) - 1;
}

void Player::savePlaylistState() {
    // --- NEW: Lock the playlist state while saving.
    std::lock_guard<std::recursive_mutex> lock(playlistMutex);
    
    json j = json::array();
    for (const auto& pl : playlists) {
        json playlistJson;
        playlistJson["name"] = pl.name;
        playlistJson["songs"] = pl.songs;
        playlistJson["playCounts"] = pl.playCounts;
        // Save the last played timestamp for each song.
        playlistJson["lastPositions"] = pl.lastPositions;
        // NEW: Save the index of the last played song.
        playlistJson["lastPlayedIndex"] = pl.lastPlayedIndex;
        j.push_back(playlistJson);
    }
    
    std::ofstream file("playlists.json");
    if (!file) {
        std::cerr << "Failed to open playlists.json for writing." << std::endl;
        return;
    }
    file << j.dump(4); // Pretty-print with an indent of 4 spaces
}

void Player::loadPlaylistState() {
    // --- NEW: Lock the playlist state while loading.
    std::lock_guard<std::recursive_mutex> lock(playlistMutex);
    
    std::ifstream file("playlists.json");
    if (!file) {
        // If the file does not exist, there's nothing to load.
        return;
    }
    json j;
    try {
        file >> j;
    } catch (json::parse_error& e) {
        std::cerr << "Failed to parse playlists.json: " << e.what() << std::endl;
        return;
    }
    
    playlists.clear();
    for (const auto& playlistJson : j) {
        Playlist p;
        p.name = playlistJson.value("name", "");
        p.songs = playlistJson.value("songs", std::vector<std::string>{});
        p.playCounts = playlistJson.value("playCounts", std::vector<int>{});
        // Load lastPositions if available; otherwise, initialize with zeros.
        if (playlistJson.contains("lastPositions")) {
            p.lastPositions = playlistJson.value("lastPositions", std::vector<double>{});
        } else {
            p.lastPositions.resize(p.songs.size(), 0.0);
        }
        // NEW: Load the last played song index (default to -1 if not present).
        p.lastPlayedIndex = playlistJson.value("lastPlayedIndex", -1);
        playlists.push_back(p);
    }
}
