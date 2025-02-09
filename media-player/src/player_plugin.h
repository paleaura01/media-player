// player_plugin.h
#ifndef PLAYER_PLUGIN_H
#define PLAYER_PLUGIN_H

#define PLAYER_API extern "C" __declspec(dllexport)

PLAYER_API void update_ui(); // Functions you want to hot-reload

#endif // PLAYER_PLUGIN_H
