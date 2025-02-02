// player_ui.cpp
#include "player.h"
#include <iostream>

// Render text
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

// Time bar
void Player::drawTimeBar() {
    SDL_SetRenderDrawColor(renderer, 30, 30, 30, 255);
    SDL_RenderFillRect(renderer, &timeBar);

    if (totalDuration > 0.0) {
        SDL_Rect progress = timeBar;
        progress.w = (int)(timeBar.w * (currentTime / totalDuration));
        SDL_SetRenderDrawColor(renderer, 0, 255, 0, 255);
        SDL_RenderFillRect(renderer, &progress);
    }

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

    // === PREV BUTTON ("<<")
    SDL_SetRenderDrawColor(renderer, 60, 60, 60, 255);
    SDL_RenderFillRect(renderer, &prevButton);
    {
        SDL_Texture* prevText = renderText("<<", white);
        if (prevText) {
            renderButtonText(prevText, prevButton);
            SDL_DestroyTexture(prevText);
        }
    }

    // === PLAY BUTTON ("Play")
    SDL_SetRenderDrawColor(renderer, 0, 200, 0, 255);
    SDL_RenderFillRect(renderer, &playButton);
    {
        SDL_Texture* playLabel = renderText("Play", white);
        if (playLabel) {
            renderButtonText(playLabel, playButton);
            SDL_DestroyTexture(playLabel);
        }
    }

    // === NEXT BUTTON (">>")
    SDL_SetRenderDrawColor(renderer, 60, 60, 60, 255);
    SDL_RenderFillRect(renderer, &nextButton);
    {
        SDL_Texture* nextTxt = renderText(">>", white);
        if (nextTxt) {
            renderButtonText(nextTxt, nextButton);
            SDL_DestroyTexture(nextTxt);
        }
    }

    // === STOP BUTTON ("Stop")
    SDL_SetRenderDrawColor(renderer, 200, 0, 0, 255);
    SDL_RenderFillRect(renderer, &stopButton);
    {
        SDL_Texture* stopLabel = renderText("Stop", white);
        if (stopLabel) {
            renderButtonText(stopLabel, stopButton);
            SDL_DestroyTexture(stopLabel);
        }
    }

    // === SHUFFLE BUTTON ("Shuffle")
    if (isShuffled) {
        SDL_SetRenderDrawColor(renderer, 0, 200, 0, 255);
    } else {
        SDL_SetRenderDrawColor(renderer, 60, 60, 60, 255);
    }
    SDL_RenderFillRect(renderer, &shuffleButton);
    {
        SDL_Texture* shuffleLabel = renderText("Shuffle", white);
        if (shuffleLabel) {
            renderButtonText(shuffleLabel, shuffleButton);
            SDL_DestroyTexture(shuffleLabel);
        }
    }

    // === MUTE BUTTON ("Mute")
    if (isMuted) {
        SDL_SetRenderDrawColor(renderer, 200, 0, 0, 255);
    } else {
        SDL_SetRenderDrawColor(renderer, 60, 60, 60, 255);
    }
    SDL_RenderFillRect(renderer, &muteButton);
    {
        SDL_Texture* muteLabel = renderText("Mute", white);
        if (muteLabel) {
            renderButtonText(muteLabel, muteButton);
            SDL_DestroyTexture(muteLabel);
        }
    }

    // === REWIND / FORWARD
    SDL_SetRenderDrawColor(renderer, 60, 60, 60, 255);
    SDL_RenderFillRect(renderer, &rewindButton);
    SDL_RenderFillRect(renderer, &forwardButton);

    SDL_Texture* rewindText = renderText("<", white);
    if (rewindText) {
        renderButtonText(rewindText, rewindButton);
        SDL_DestroyTexture(rewindText);
    }

    SDL_Texture* forwardText = renderText(">", white);
    if (forwardText) {
        renderButtonText(forwardText, forwardButton);
        SDL_DestroyTexture(forwardText);
    }

    // === VOLUME BAR
    SDL_SetRenderDrawColor(renderer, 60, 60, 60, 255);
    SDL_RenderFillRect(renderer, &volumeBar);

    SDL_Rect volumeFill = volumeBar;
    volumeFill.w = (int)(volumeBar.w * (volume / 100.0f));
    SDL_SetRenderDrawColor(renderer, 0, 200, 0, 255);
    SDL_RenderFillRect(renderer, &volumeFill);

    // Display volume percentage
    char volumeText[16];
    snprintf(volumeText, sizeof(volumeText), "%.0f%%", volume);
    SDL_Texture* volumeTexture = renderText(volumeText, white);
    if (volumeTexture) {
        renderButtonText(volumeTexture, volumeBar);
        SDL_DestroyTexture(volumeTexture);
    }
}

// Left panel playlists
void Player::drawPlaylistPanel() {
    playlistRects.clear();
    playlistDeleteRects.clear();

    SDL_SetRenderDrawColor(renderer, 40, 40, 40, 255);
    SDL_RenderFillRect(renderer, &playlistPanel);

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

        if ((int)i == activePlaylist) {
            SDL_SetRenderDrawColor(renderer, 60, 100, 60, 255);
        } else {
            SDL_SetRenderDrawColor(renderer, 50, 50, 50, 255);
        }
        SDL_RenderFillRect(renderer, &playlistRect);

        // "X" area
        SDL_SetRenderDrawColor(renderer, 90, 30, 30, 255);
        SDL_RenderFillRect(renderer, &deleteRect);

        bool editingThis = (isRenaming && (int)i == renameIndex);
        if (editingThis) {
            SDL_SetRenderDrawColor(renderer, 90, 90, 90, 255);
            SDL_RenderFillRect(renderer, &playlistRect);

            // Show renameBuffer
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

        // Draw "X"
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

        playlistRects.push_back(playlistRect);
        playlistDeleteRects.push_back(deleteRect);
        yOffset += playlistRect.h + 5;
    }
}

// Right panel songs
void Player::drawSongPanel() {
    songRects.clear();
    SDL_SetRenderDrawColor(renderer, 30, 30, 30, 255);
    SDL_RenderFillRect(renderer, &libraryPanel);

    if (activePlaylist >= 0 && activePlaylist < (int)playlists.size()) {
        SDL_Color white = {255, 255, 255, 255};
        int yOffset = libraryPanel.y + 10;

        for (size_t i = 0; i < playlists[activePlaylist].songs.size(); i++) {
            auto& song = playlists[activePlaylist].songs[i];
            SDL_Rect songRect = {
                libraryPanel.x + 10,
                yOffset,
                libraryPanel.w - 20,
                25
            };
            
            // Highlight if this song is currently loaded
            if (song == loadedFile) {
                SDL_SetRenderDrawColor(renderer, 0, 100, 0, 255);
            } else {
                SDL_SetRenderDrawColor(renderer, 45, 45, 45, 255);
            }
            SDL_RenderFillRect(renderer, &songRect);

            songRects.push_back(songRect);

            // Extract just the filename
            std::string filename = song.substr(song.find_last_of("/\\") + 1);

            // Build a display string like "(played 3) MySong.mp3"
            int plays = playlists[activePlaylist].playCounts[i];
            std::string displayName = "(played " + std::to_string(plays) + ") " + filename;

            // Render that text
            SDL_Texture* songTex = renderText(displayName, white);
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

            // If hovered, draw the X area on the right side
            if ((int)i == hoveredSongIndex) {
                SDL_Rect deleteRect = {
                    songRect.x + songRect.w - 30,
                    songRect.y,
                    25,
                    songRect.h
                };
                SDL_SetRenderDrawColor(renderer, 90, 30, 30, 255);
                SDL_RenderFillRect(renderer, &deleteRect);
                
                SDL_Texture* xText = renderText("X", white);
                if (xText) {
                    renderButtonText(xText, deleteRect);
                    SDL_DestroyTexture(xText);
                }
            }

            yOffset += songRect.h + 2;
        }
    }
}

// Confirmation Dialog
void Player::drawConfirmDialog() {
    if (!isConfirmingDeletion) return;

    // Dim background behind the dialog
    SDL_SetRenderDrawBlendMode(renderer, SDL_BLENDMODE_BLEND);
    SDL_SetRenderDrawColor(renderer, 0, 0, 0, 150);
    SDL_Rect fullScreen = { 0, 0, 800, 600 };
    SDL_RenderFillRect(renderer, &fullScreen);
    SDL_SetRenderDrawBlendMode(renderer, SDL_BLENDMODE_NONE);

    // Draw the dialog box
    SDL_SetRenderDrawColor(renderer, 60, 60, 60, 255);
    SDL_RenderFillRect(renderer, &confirmDialogRect);

    // "Are you sure?" text
    SDL_Color white = {255, 255, 255, 255};
    SDL_Texture* msg = renderText("Are you sure?", white);
    if (msg) {
        int w, h;
        SDL_QueryTexture(msg, nullptr, nullptr, &w, &h);
        SDL_Rect msgRect = {
            confirmDialogRect.x + (confirmDialogRect.w - w)/2,
            confirmDialogRect.y + 20,
            w, h
        };
        SDL_RenderCopy(renderer, msg, nullptr, &msgRect);
        SDL_DestroyTexture(msg);
    }

    // Yes button
    SDL_SetRenderDrawColor(renderer, 80, 140, 80, 255);
    SDL_RenderFillRect(renderer, &confirmYesButton);
    SDL_Texture* yesText = renderText("Yes", white);
    if (yesText) {
        renderButtonText(yesText, confirmYesButton);
        SDL_DestroyTexture(yesText);
    }

    // No button
    SDL_SetRenderDrawColor(renderer, 140, 80, 80, 255);
    SDL_RenderFillRect(renderer, &confirmNoButton);
    SDL_Texture* noText = renderText("No", white);
    if (noText) {
        renderButtonText(noText, confirmNoButton);
        SDL_DestroyTexture(noText);
    }
}
