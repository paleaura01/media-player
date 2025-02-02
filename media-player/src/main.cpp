// main.cpp
#include <iostream>
#include "player.h"

// Standard main() function.
int main(int argc, char* argv[]) {
    std::cout << "Starting Barebones Media Player..." << std::endl;
    Player player;
    if (!player.init()) {
        std::cerr << "Failed to initialize the media player!" << std::endl;
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
// Provide a Unicode entry point fallback that calls main()
// This ensures that if the CRT expects wmain(), it finds one.
int wmain(int argc, wchar_t* argv[]) {
    // For simplicity, we ignore the wide arguments.
    return main(0, nullptr);
}
#endif
