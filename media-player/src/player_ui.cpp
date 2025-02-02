// player_ui.cpp
#include "player.h"
#include <iostream>

// Render text with the current font/color
SDL_Texture* Player::renderText(const std::string &text, SDL_Color color) {
    if (!font) return nullptr;
    SDL_Surface* surface = TTF_RenderText_Solid(font, text.c_str(), color);
    if (!surface) {
        std::cerr << "TTF_RenderText_Solid Error: " << TTF_GetError() << std::endl;
        return nullptr;
    }
    SDL_Texture* texture = SDL_CreateTextureFromSurface(renderer, surface);
    SDL_FreeSurface(surface);
    return texture;
}

void Player::renderButtonText(SDL_Texture* texture, const SDL_Rect& button) {
    if (texture) {
        int w, h;
        SDL_QueryTexture(texture, nullptr, nullptr, &w, &h);
        SDL_Rect dest = { 
            button.x + (button.w - w) / 2,
            button.y + (button.h - h) / 2,
            w, h 
        };
        SDL_RenderCopy(renderer, texture, nullptr, &dest);
    }
}

// Draw the time bar
void Player::drawTimeBar() {
    SDL_SetRenderDrawColor(renderer, 30, 30, 30, 255);
    SDL_RenderFillRect(renderer, &timeBar);

    // Draw progress
    if (totalDuration > 0.0) {
        SDL_Rect progress = timeBar;
        progress.w = static_cast<int>(timeBar.w * (currentTime / totalDuration));
        SDL_SetRenderDrawColor(renderer, 0, 255, 0, 255);
        SDL_RenderFillRect(renderer, &progress);
    }
    
    // Time text
    char timeText[32];
    int curMin = (int)(currentTime / 60);
    int curSec = (int)currentTime % 60;
    int totMin = (int)(totalDuration / 60);
    int totSec = (int)totalDuration % 60;
    snprintf(timeText, sizeof(timeText), "%d:%02d / %d:%02d", curMin, curSec, totMin, totSec);

    SDL_Color white = {255, 255, 255, 255};
    SDL_Texture* timeTexture = renderText(timeText, white);
    if (timeTexture) {
        int w, h;
        SDL_QueryTexture(timeTexture, nullptr, nullptr, &w, &h);

        SDL_Rect dest = {
            timeBar.x + timeBar.w - w - 5,
            timeBar.y + (timeBar.h - h)/2,
            w, h
        };
        SDL_RenderCopy(renderer, timeTexture, nullptr, &dest);
        SDL_DestroyTexture(timeTexture);
    }
}

// Transport controls
void Player::drawControls() {
    SDL_Color white = {255, 255, 255, 255};

    // Previous
    SDL_SetRenderDrawColor(renderer, 60, 60, 60, 255);
    SDL_RenderFillRect(renderer, &prevButton);
    SDL_Texture* prevText = renderText("<<", white);

    // Play
    SDL_SetRenderDrawColor(renderer, 0, 200, 0, 255);
    SDL_RenderFillRect(renderer, &playButton);
    SDL_Texture* playText = renderText(playingAudio ? "||" : ">", white);

    // Next
    SDL_SetRenderDrawColor(renderer, 60, 60, 60, 255);
    SDL_RenderFillRect(renderer, &nextButton);
    SDL_Texture* nextText = renderText(">>", white);

    // Stop
    SDL_SetRenderDrawColor(renderer, 200, 0, 0, 255);
    SDL_RenderFillRect(renderer, &stopButton);
    SDL_Texture* stopText = renderText("□", white);

    // Shuffle
    if (isShuffled) {
        SDL_SetRenderDrawColor(renderer, 0, 200, 0, 255);
    } else {
        SDL_SetRenderDrawColor(renderer, 60, 60, 60, 255);
    }
    SDL_RenderFillRect(renderer, &shuffleButton);
    SDL_Texture* shuffleText = renderText("🔀", white);

    // Mute
    if (isMuted) {
        SDL_SetRenderDrawColor(renderer, 200, 0, 0, 255);
    } else {
        SDL_SetRenderDrawColor(renderer, 60, 60, 60, 255);
    }
    SDL_RenderFillRect(renderer, &muteButton);
    SDL_Texture* muteText = renderText(isMuted ? "🔇" : "🔊", white);

    // Render text
    renderButtonText(prevText,    prevButton);
    renderButtonText(playText,    playButton);
    renderButtonText(nextText,    nextButton);
    renderButtonText(stopText,    stopButton);
    renderButtonText(shuffleText, shuffleButton);
    renderButtonText(muteText,    muteButton);

    // Cleanup
    SDL_DestroyTexture(prevText);
    SDL_DestroyTexture(playText);
    SDL_DestroyTexture(nextText);
    SDL_DestroyTexture(stopText);
    SDL_DestroyTexture(shuffleText);
    SDL_DestroyTexture(muteText);
}

// Draw the playlists panel on the left
void Player::drawPlaylistPanel() {
    playlistRects.clear();
    playlistDeleteRects.clear();

    // Panel background
    SDL_SetRenderDrawColor(renderer, 40, 40, 40, 255);
    SDL_RenderFillRect(renderer, &playlistPanel);

    // "New Playlist" button
    SDL_SetRenderDrawColor(renderer, 60, 60, 60, 255);
    SDL_RenderFillRect(renderer, &newPlaylistButton);

    SDL_Color white = {255, 255, 255, 255};
    SDL_Texture* newPlaylistText = renderText("New Playlist", white);
    if (newPlaylistText) {
        int w, h;
        SDL_QueryTexture(newPlaylistText, nullptr, nullptr, &w, &h);
        SDL_Rect dest = { 
            newPlaylistButton.x + (newPlaylistButton.w - w)/2,
            newPlaylistButton.y + (newPlaylistButton.h - h)/2,
            w, h
        };
        SDL_RenderCopy(renderer, newPlaylistText, nullptr, &dest);
        SDL_DestroyTexture(newPlaylistText);
    }

    int yOffset = newPlaylistButton.y + newPlaylistButton.h + 10;
    for (size_t i = 0; i < playlists.size(); i++) {
        // Full row for the playlist
        SDL_Rect playlistRect = {
            playlistPanel.x + 5,
            yOffset,
            playlistPanel.w - 10,
            25
        };
        SDL_Rect deleteRect = {
            playlistRect.x + playlistRect.w - 25,
            playlistRect.y,
            25,
            playlistRect.h
        };

        // Decide color
        if ((int)i == activePlaylist) {
            SDL_SetRenderDrawColor(renderer, 60, 100, 60, 255);
        } else {
            SDL_SetRenderDrawColor(renderer, 50, 50, 50, 255);
        }
        SDL_RenderFillRect(renderer, &playlistRect);

        // "X" button
        SDL_SetRenderDrawColor(renderer, 90, 30, 30, 255);
        SDL_RenderFillRect(renderer, &deleteRect);

        // =============== Inline Rename vs Normal Display ===============
        bool editingThis = (isRenaming && (int)i == renameIndex);

        if (editingThis) {
            // Lighten a bit more to show editing
            SDL_SetRenderDrawColor(renderer, 90, 90, 90, 255);
            SDL_RenderFillRect(renderer, &playlistRect);

            // Render renameBuffer
            SDL_Texture* editTex = renderText(renameBuffer, white);
            if (editTex) {
                int w, h;
                SDL_QueryTexture(editTex, nullptr, nullptr, &w, &h);
                SDL_Rect dest = {
                    playlistRect.x + 5,
                    playlistRect.y + (playlistRect.h - h)/2,
                    w, h
                };
                SDL_RenderCopy(renderer, editTex, nullptr, &dest);
                SDL_DestroyTexture(editTex);
            }
        }
        else {
            // Normal playlist name
            SDL_Texture* plName = renderText(playlists[i].name, white);
            if (plName) {
                int w, h;
                SDL_QueryTexture(plName, nullptr, nullptr, &w, &h);
                SDL_Rect dest = {
                    playlistRect.x + 5,
                    playlistRect.y + (playlistRect.h - h)/2,
                    w, h
                };
                SDL_RenderCopy(renderer, plName, nullptr, &dest);
                SDL_DestroyTexture(plName);
            }
        }

        // Render the "X"
        SDL_Texture* xText = renderText("X", white);
        if (xText) {
            int w, h;
            SDL_QueryTexture(xText, nullptr, nullptr, &w, &h);
            SDL_Rect dest = {
                deleteRect.x + (deleteRect.w - w)/2,
                deleteRect.y + (deleteRect.h - h)/2,
                w, h
            };
            SDL_RenderCopy(renderer, xText, nullptr, &dest);
            SDL_DestroyTexture(xText);
        }

        // Keep track of these rects for click detection
        playlistRects.push_back(playlistRect);
        playlistDeleteRects.push_back(deleteRect);

        yOffset += playlistRect.h + 5;
    }
}

// Draw the songs in the active playlist (right pane)
void Player::drawSongPanel() {
    // Clear old song rects
    songRects.clear();

    SDL_SetRenderDrawColor(renderer, 30, 30, 30, 255);
    SDL_RenderFillRect(renderer, &libraryPanel);

    if (activePlaylist >= 0 && activePlaylist < (int)playlists.size()) {
        SDL_Color white = {255, 255, 255, 255};
        int yOffset = libraryPanel.y + 10;

        for (auto& song : playlists[activePlaylist].songs) {
            SDL_Rect songRect = {
                libraryPanel.x + 10,
                yOffset,
                libraryPanel.w - 20,
                25
            };
            SDL_SetRenderDrawColor(renderer, 45, 45, 45, 255);
            SDL_RenderFillRect(renderer, &songRect);

            songRects.push_back(songRect);

            // Filename only
            std::string filename = song.substr(song.find_last_of("/\\")+1);
            SDL_Texture* songTex = renderText(filename, white);
            if (songTex) {
                int w, h;
                SDL_QueryTexture(songTex, nullptr, nullptr, &w, &h);
                SDL_Rect dest = {
                    songRect.x + 5,
                    songRect.y + (songRect.h - h)/2,
                    w, h
                };
                SDL_RenderCopy(renderer, songTex, nullptr, &dest);
                SDL_DestroyTexture(songTex);
            }
            yOffset += songRect.h + 2;
        }
    }
}
