// protocol.cpp
#include "protocol.h"

std::string Protocol::createCommand(const std::string& cmd) {
    return "{\"command\": \"" + cmd + "\"}";
}
