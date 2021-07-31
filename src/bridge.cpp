//
// Created by Caelum van Ispelen on 6/20/21.
//

#include <iostream>
#include <thread>
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

    NetworkBridge::NetworkBridge(NetworkConnection conn) {
        sendQueue = std::make_shared<moodycamel::BlockingReaderWriterQueue<std::string>>();
        receiveQueue = std::make_shared<moodycamel::ReaderWriterQueue<std::string>>();

        auto con = std::make_shared<NetworkConnection>(std::move(conn));

        auto recvQueue = this->receiveQueue;
        auto sendQueue = this->sendQueue;

        std::thread recvThread([=] () {
            while (!con->getError()) {
                auto msg = con->recvMessage();
                recvQueue->emplace(std::move(msg));
            }
        });
        std::thread sendThread([=] () {
            std::string message;
            while (!con->getError()) {
                sendQueue->wait_dequeue(message);
                con->sendMessage(message);
            }
        });

        recvThread.detach();
        sendThread.detach();
    }

    std::optional<std::string> NetworkBridge::pollReceivedPacket() {
        std::string message;
        if (receiveQueue->try_dequeue(message)) {
            return message;
        } else {
            return {};
        }
    }

    void NetworkBridge::sendPacket(std::string data) {
        sendQueue->emplace(std::move(data));
    }
}
