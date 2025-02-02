// player_ui.cpp
#include "player.h"
#include <iostream>

// Renders text with the current font/color
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

// Renders text centered in a button
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

// Draw the progress/time bar
void Player::drawTimeBar() {
    // Background
    SDL_SetRenderDrawColor(renderer, 30, 30, 30, 255);
    SDL_RenderFillRect(renderer, &timeBar);
    
    // Progress
    if (totalDuration > 0.0) {
        SDL_Rect progress = timeBar;
        progress.w = static_cast<int>(timeBar.w * (currentTime / totalDuration));
        SDL_SetRenderDrawColor(renderer, 0, 255, 0, 255);
        SDL_RenderFillRect(renderer, &progress);
    }
    
    // Text
    char timeText[32];
    int curMin = static_cast<int>(currentTime / 60);
    int curSec = static_cast<int>(currentTime) % 60;
    int totMin = static_cast<int>(totalDuration / 60);
    int totSec = static_cast<int>(totalDuration) % 60;
    snprintf(timeText, sizeof(timeText), "%d:%02d / %d:%02d", curMin, curSec, totMin, totSec);
    
    SDL_Color white = {255, 255, 255, 255};
    SDL_Texture* timeTexture = renderText(timeText, white);
    if (timeTexture) {
        int w, h;
        SDL_QueryTexture(timeTexture, nullptr, nullptr, &w, &h);
        // Right-align above the bar
        SDL_Rect dest = {
            timeBar.x + timeBar.w - w - 5,
            timeBar.y - h - 5,
            w, h
        };
        SDL_RenderCopy(renderer, timeTexture, nullptr, &dest);
        SDL_DestroyTexture(timeTexture);
    }
}

// Draw playback/transport controls
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

// Draw the playlist panel on the left side
void Player::drawPlaylistPanel() {
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
            newPlaylistButton.x + (newPlaylistButton.w - w) / 2,
            newPlaylistButton.y + (newPlaylistButton.h - h) / 2,
            w, h
        };
        SDL_RenderCopy(renderer, newPlaylistText, nullptr, &dest);
        SDL_DestroyTexture(newPlaylistText);
    }

    // Show each playlist
    int yOffset = newPlaylistButton.y + newPlaylistButton.h + 10;
    for (size_t i = 0; i < playlists.size(); i++) {
        SDL_Rect playlistRect = {
            playlistPanel.x + 5, yOffset,
            playlistPanel.w - 10, 25
        };
        if (static_cast<int>(i) == activePlaylist) {
            SDL_SetRenderDrawColor(renderer, 60, 100, 60, 255);
        } else {
            SDL_SetRenderDrawColor(renderer, 50, 50, 50, 255);
        }
        SDL_RenderFillRect(renderer, &playlistRect);

        SDL_Texture* playlistName = renderText(playlists[i].name, white);
        if (playlistName) {
            int w, h;
            SDL_QueryTexture(playlistName, nullptr, nullptr, &w, &h);
            SDL_Rect dest = {
                playlistRect.x + 5,
                playlistRect.y + (playlistRect.h - h) / 2,
                w, h
            };
            SDL_RenderCopy(renderer, playlistName, nullptr, &dest);
            SDL_DestroyTexture(playlistName);
        }
        yOffset += playlistRect.h + 5;

        // If it's the active playlist, draw the songs
        if (static_cast<int>(i) == activePlaylist) {
            for (const auto& song : playlists[i].songs) {
                SDL_Rect songRect = {
                    playlistPanel.x + 15, yOffset,
                    playlistPanel.w - 20, 20
                };
                SDL_SetRenderDrawColor(renderer, 45, 45, 45, 255);
                SDL_RenderFillRect(renderer, &songRect);

                // Show filename only
                std::string filename = song.substr(song.find_last_of("/\\") + 1);
                SDL_Texture* songText = renderText(filename, white);
                if (songText) {
                    int w, h;
                    SDL_QueryTexture(songText, nullptr, nullptr, &w, &h);
                    SDL_Rect dest = {
                        songRect.x + 5,
                        songRect.y + (songRect.h - h) / 2,
                        w, h
                    };
                    SDL_RenderCopy(renderer, songText, nullptr, &dest);
                    SDL_DestroyTexture(songText);
                }
                yOffset += songRect.h + 2;
            }
            yOffset += 10;
        }
    }
}
