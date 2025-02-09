// player_ui.cpp
#include "player.h"
#include <iostream>

// Render text using the loaded font.
SDL_Texture* Player::renderText(const std::string &text, SDL_Color color) {
    if (!font) {
        std::cerr << "[ERROR] renderText: Font not loaded." << std::endl;
        return nullptr;
    }
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

void Player::drawTimeBar() {
    SDL_SetRenderDrawColor(renderer, 30, 30, 30, 255);
    SDL_RenderFillRect(renderer, &timeBar);
    
    if (totalDuration > 0.0) {
        SDL_Rect progress = timeBar;
        progress.w = static_cast<int>(timeBar.w * (currentTime / totalDuration));
        SDL_SetRenderDrawColor(renderer, 0, 255, 0, 255);
        SDL_RenderFillRect(renderer, &progress);
    }
    
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
        SDL_Rect dest = { timeBar.x + timeBar.w - w - 5, timeBar.y + (timeBar.h - h) / 2, w, h };
        SDL_RenderCopy(renderer, timeTexture, nullptr, &dest);
        SDL_DestroyTexture(timeTexture);
    }
}

void Player::drawControls() {
    SDL_Color white = {255, 255, 255, 255};
    
    // Prev Button
    SDL_SetRenderDrawColor(renderer, 60, 60, 60, 255);
    SDL_RenderFillRect(renderer, &prevButton);
    {
        SDL_Texture* prevText = renderText("<<", white);
        if (prevText) {
            renderButtonText(prevText, prevButton);
            SDL_DestroyTexture(prevText);
        }
    }
    
    // Play/Pause Button
    SDL_SetRenderDrawColor(renderer, 0, 200, 0, 255);
    SDL_RenderFillRect(renderer, &playButton);
    const char* label = (playingAudio) ? "Pause" : "Play";
    {
        SDL_Texture* playLabel = renderText(label, white);
        if (playLabel) {
            renderButtonText(playLabel, playButton);
            SDL_DestroyTexture(playLabel);
        }
    }
    
    // Next Button
    SDL_SetRenderDrawColor(renderer, 60, 60, 60, 255);
    SDL_RenderFillRect(renderer, &nextButton);
    {
        SDL_Texture* nextTxt = renderText(">>", white);
        if (nextTxt) {
            renderButtonText(nextTxt, nextButton);
            SDL_DestroyTexture(nextTxt);
        }
    }
    
    // Shuffle Button
    if (isShuffled)
        SDL_SetRenderDrawColor(renderer, 0, 200, 0, 255);
    else
        SDL_SetRenderDrawColor(renderer, 60, 60, 60, 255);
    SDL_RenderFillRect(renderer, &shuffleButton);
    {
        SDL_Texture* shuffleLabel = renderText("Shuffle", white);
        if (shuffleLabel) {
            renderButtonText(shuffleLabel, shuffleButton);
            SDL_DestroyTexture(shuffleLabel);
        }
    }
    
    // Mute Button
    if (isMuted)
        SDL_SetRenderDrawColor(renderer, 200, 0, 0, 255);
    else
        SDL_SetRenderDrawColor(renderer, 60, 60, 60, 255);
    SDL_RenderFillRect(renderer, &muteButton);
    {
        SDL_Texture* muteLabel = renderText("Mute", white);
        if (muteLabel) {
            renderButtonText(muteLabel, muteButton);
            SDL_DestroyTexture(muteLabel);
        }
    }
    
    // Rewind and Forward Buttons
    SDL_SetRenderDrawColor(renderer, 60, 60, 60, 255);
    SDL_RenderFillRect(renderer, &rewindButton);
    SDL_RenderFillRect(renderer, &forwardButton);
    {
        SDL_Texture* rewindText = renderText("<", white);
        if (rewindText) {
            renderButtonText(rewindText, rewindButton);
            SDL_DestroyTexture(rewindText);
        }
    }
    {
        SDL_Texture* forwardText = renderText(">", white);
        if (forwardText) {
            renderButtonText(forwardText, forwardButton);
            SDL_DestroyTexture(forwardText);
        }
    }
    
    // Volume Bar
    SDL_SetRenderDrawColor(renderer, 60, 60, 60, 255);
    SDL_RenderFillRect(renderer, &volumeBar);
    SDL_Rect volumeFill = volumeBar;
    volumeFill.w = static_cast<int>(volumeBar.w * (volume / 100.0f));
    SDL_SetRenderDrawColor(renderer, 0, 200, 0, 255);
    SDL_RenderFillRect(renderer, &volumeFill);
    char volumeText[16];
    snprintf(volumeText, sizeof(volumeText), "%.0f%%", volume);
    {
        SDL_Texture* volumeTexture = renderText(volumeText, white);
        if (volumeTexture) {
            renderButtonText(volumeTexture, volumeBar);
            SDL_DestroyTexture(volumeTexture);
        }
    }
}

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
        SDL_Rect dest = { newPlaylistButton.x + (newPlaylistButton.w - w) / 2,
                          newPlaylistButton.y + (newPlaylistButton.h - h) / 2, w, h };
        SDL_RenderCopy(renderer, newPlaylistText, nullptr, &dest);
        SDL_DestroyTexture(newPlaylistText);
    }
    
    int yOffset = newPlaylistButton.y + newPlaylistButton.h + 10;
    for (size_t i = 0; i < playlists.size(); i++) {
        SDL_Rect playlistRect = { playlistPanel.x + 5, yOffset, playlistPanel.w - 10, 25 };
        SDL_Rect deleteRect = { playlistRect.x + playlistRect.w - 25, playlistRect.y, 25, playlistRect.h };
        
        if ((int)i == activePlaylist)
            SDL_SetRenderDrawColor(renderer, 60, 100, 60, 255);
        else
            SDL_SetRenderDrawColor(renderer, 50, 50, 50, 255);
        SDL_RenderFillRect(renderer, &playlistRect);
        
        SDL_SetRenderDrawColor(renderer, 90, 30, 30, 255);
        SDL_RenderFillRect(renderer, &deleteRect);
        
        bool editingThis = (isRenaming && (int)i == renameIndex);
        if (editingThis) {
            SDL_SetRenderDrawColor(renderer, 90, 90, 90, 255);
            SDL_RenderFillRect(renderer, &playlistRect);
            SDL_Texture* editTex = renderText(renameBuffer, white);
            if (editTex) {
                int w, h;
                SDL_QueryTexture(editTex, nullptr, nullptr, &w, &h);
                SDL_Rect dest = { playlistRect.x + 5, playlistRect.y + (playlistRect.h - h) / 2, w, h };
                SDL_RenderCopy(renderer, editTex, nullptr, &dest);
                SDL_DestroyTexture(editTex);
            }
        }
        else {
            SDL_Texture* plName = renderText(playlists[i].name, white);
            if (plName) {
                int w, h;
                SDL_QueryTexture(plName, nullptr, nullptr, &w, &h);
                SDL_Rect dest = { playlistRect.x + 5, playlistRect.y + (playlistRect.h - h) / 2, w, h };
                SDL_RenderCopy(renderer, plName, nullptr, &dest);
                SDL_DestroyTexture(plName);
            }
        }
        
        SDL_Texture* xText = renderText("X", white);
        if (xText) {
            int w, h;
            SDL_QueryTexture(xText, nullptr, nullptr, &w, &h);
            SDL_Rect dest = { deleteRect.x + (deleteRect.w - w) / 2, deleteRect.y + (deleteRect.h - h) / 2, w, h };
            SDL_RenderCopy(renderer, xText, nullptr, &dest);
            SDL_DestroyTexture(xText);
        }
        
        playlistRects.push_back(playlistRect);
        playlistDeleteRects.push_back(deleteRect);
        yOffset += playlistRect.h + 5;
    }
}

void Player::drawSongPanel() {
    songRects.clear();
    SDL_SetRenderDrawColor(renderer, 30, 30, 30, 255);
    SDL_RenderFillRect(renderer, &libraryPanel);
    
    if (activePlaylist >= 0 && activePlaylist < (int)playlists.size()) {
        SDL_Color white = {255, 255, 255, 255};
        int yOffset = libraryPanel.y + 10;
        for (size_t i = 0; i < playlists[activePlaylist].songs.size(); i++) {
            SDL_Rect songRect = { libraryPanel.x + 10, yOffset, libraryPanel.w - 20, 25 };
            
            // Highlight the currently loaded song.
            if (playlists[activePlaylist].songs[i] == loadedFile)
                SDL_SetRenderDrawColor(renderer, 0, 100, 0, 255);
            else
                SDL_SetRenderDrawColor(renderer, 45, 45, 45, 255);
            SDL_RenderFillRect(renderer, &songRect);
            songRects.push_back(songRect);
            
            // Prepare display text with play count.
            std::string filename = playlists[activePlaylist].songs[i];
            size_t pos = filename.find_last_of("/\\");
            if (pos != std::string::npos)
                filename = filename.substr(pos + 1);
            int plays = playlists[activePlaylist].playCounts[i];
            std::string displayName = "(played " + std::to_string(plays) + ") " + filename;
            
            // Clip text if it exceeds the song row width.
            int textW, textH;
            TTF_SizeText(font, displayName.c_str(), &textW, &textH);
            const int maxWidth = songRect.w - 10; // 10px padding.
            if (textW > maxWidth) {
                while (displayName.size() > 3) {
                    displayName = displayName.substr(0, displayName.size()-1);
                    std::string testStr = displayName + "...";
                    TTF_SizeText(font, testStr.c_str(), &textW, &textH);
                    if (textW <= maxWidth) {
                        displayName = testStr;
                        break;
                    }
                }
            }
            
            SDL_Texture* songTex = renderText(displayName, white);
            if (songTex) {
                int w, h;
                SDL_QueryTexture(songTex, nullptr, nullptr, &w, &h);
                SDL_Rect dest = { songRect.x + 5, songRect.y + (songRect.h - h) / 2, w, h };
                SDL_RenderCopy(renderer, songTex, nullptr, &dest);
                SDL_DestroyTexture(songTex);
            }
            
            // If this song row is hovered, draw a red "X" button on its right.
            if (static_cast<int>(i) == hoveredSongIndex) {
                SDL_Rect deleteRect = { songRect.x + songRect.w - 30, songRect.y, 25, songRect.h };
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

void Player::drawConfirmDialog() {
    if (!isConfirmingDeletion)
        return;
    
    SDL_SetRenderDrawBlendMode(renderer, SDL_BLENDMODE_BLEND);
    SDL_SetRenderDrawColor(renderer, 0, 0, 0, 150);
    SDL_Rect fullScreen = { 0, 0, 800, 600 };
    SDL_RenderFillRect(renderer, &fullScreen);
    SDL_SetRenderDrawBlendMode(renderer, SDL_BLENDMODE_NONE);
    
    SDL_SetRenderDrawColor(renderer, 60, 60, 60, 255);
    SDL_RenderFillRect(renderer, &confirmDialogRect);
    
    SDL_Color white = {255, 255, 255, 255};
    SDL_Texture* msg = renderText("Are you sure?", white);
    if (msg) {
        int w, h;
        SDL_QueryTexture(msg, nullptr, nullptr, &w, &h);
        SDL_Rect msgRect = { confirmDialogRect.x + (confirmDialogRect.w - w) / 2,
                             confirmDialogRect.y + 20, w, h };
        SDL_RenderCopy(renderer, msg, nullptr, &msgRect);
        SDL_DestroyTexture(msg);
    }
    
    SDL_SetRenderDrawColor(renderer, 80, 140, 80, 255);
    SDL_RenderFillRect(renderer, &confirmYesButton);
    SDL_Texture* yesText = renderText("Yes", white);
    if (yesText) {
        renderButtonText(yesText, confirmYesButton);
        SDL_DestroyTexture(yesText);
    }
    
    SDL_SetRenderDrawColor(renderer, 140, 80, 80, 255);
    SDL_RenderFillRect(renderer, &confirmNoButton);
    SDL_Texture* noText = renderText("No", white);
    if (noText) {
        renderButtonText(noText, confirmNoButton);
        SDL_DestroyTexture(noText);
    }
}
