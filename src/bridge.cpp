//
// Created by Caelum van Ispelen on 6/20/21.
//

#include <iostream>
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

    std::pair<std::unique_ptr<Bridge>, std::unique_ptr<Bridge>> newLocalBridgePair() {
        auto queueA = std::make_shared<moodycamel::ReaderWriterQueue<std::string>>();
        auto queueB = std::make_shared<moodycamel::ReaderWriterQueue<std::string>>();
        return {
                std::make_unique<LocalBridge>(queueA, queueB),
                std::make_unique<LocalBridge>(queueB, queueA),
        };
    }
}
