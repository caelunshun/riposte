//
// Created by Caelum van Ispelen on 6/20/21.
//

#include "bridge.h"

namespace rip {
    std::optional<std::string> LocalBridge::pollReceivedPacket() {
        std::string val;
        if (receiveQueue->try_dequeue(val)) {
            return val;
        } else {
            return {};
        }
    }

    void LocalBridge::sendPacket(std::string data) {
        sendQueue->emplace(std::move(data));
    }
}
