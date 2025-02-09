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
    json j = json::array();
    for (const auto& pl : playlists) {
        json playlistJson;
        playlistJson["name"] = pl.name;
        playlistJson["songs"] = pl.songs;
        playlistJson["playCounts"] = pl.playCounts;
        // NEW: Save the last played timestamp for each song.
        playlistJson["lastPositions"] = pl.lastPositions;
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
        // NEW: Load lastPositions if available; otherwise, initialize with zeros.
        if (playlistJson.contains("lastPositions")) {
            p.lastPositions = playlistJson.value("lastPositions", std::vector<double>{});
        } else {
            p.lastPositions.resize(p.songs.size(), 0.0);
        }
        playlists.push_back(p);
    }
}
