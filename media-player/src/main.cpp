// main.cpp
#include <iostream>
#include "player.h"

int main(int argc, char* argv[]) {
    (void)argc;
    (void)argv;
    std::cout << "Starting Twink Audio Player..." << std::endl;
    
    Player player;
    if (!player.init()) {
        std::cerr << "Failed to initialize the audio player!" << std::endl;
        return 1;
    }
    
    while (player.isRunning()) {
        player.update();
    }
    
    player.shutdown();
    return 0;
}

#ifdef _WIN32
#include <windows.h>
// Fallback WinMain in case the linker looks for it.
int WINAPI WinMain(HINSTANCE, HINSTANCE, LPSTR, int) {
    return main(__argc, __argv);
}
#endif
