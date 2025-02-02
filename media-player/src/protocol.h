// protocol.h
#ifndef PROTOCOL_H
#define PROTOCOL_H

#include <string>

class Protocol {
public:
    static std::string createCommand(const std::string& cmd);
};

#endif // PROTOCOL_H
