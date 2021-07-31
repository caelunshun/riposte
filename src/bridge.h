//
// Created by Caelum van Ispelen on 6/20/21.
//

#ifndef RIPOSTE_BRIDGE_H
#define RIPOSTE_BRIDGE_H

#include <string>
#include <optional>
#include <memory>

#include <readerwriterqueue/readerwriterqueue.h>
#include "network.h"

namespace rip {
    // A bridge between client and server.
    //
    // Backed either by a network connection or
    // a queue connected to another thread.
    class Bridge {
    public:
        Bridge() = default;

        virtual ~Bridge() = default;

        Bridge(Bridge &&other) noexcept = default;

        virtual std::optional<std::string> pollReceivedPacket() = 0;

        virtual void sendPacket(std::string data) = 0;
    };

    // A bridge connected to another thread via a queue.
    class LocalBridge : public Bridge {
        std::shared_ptr<moodycamel::ReaderWriterQueue<std::string>> sendQueue;
        std::shared_ptr<moodycamel::ReaderWriterQueue<std::string>> receiveQueue;

    public:
        LocalBridge(std::shared_ptr<moodycamel::ReaderWriterQueue<std::string>> sendQueue,
                    std::shared_ptr<moodycamel::ReaderWriterQueue<std::string>> receiveQueue) : Bridge(), sendQueue(sendQueue), receiveQueue(receiveQueue) {}

        ~LocalBridge() override = default;

        std::optional<std::string> pollReceivedPacket() override;

        void sendPacket(std::string data) override;
    };

    // A bridge connected to a network connection, driven on a separate thread.
    class NetworkBridge : public Bridge {
        std::shared_ptr<moodycamel::BlockingReaderWriterQueue<std::string>> sendQueue;
        std::shared_ptr<moodycamel::ReaderWriterQueue<std::string>> receiveQueue;

    public:
        NetworkBridge(NetworkConnection conn);

        ~NetworkBridge() override = default;

        std::optional<std::string> pollReceivedPacket() override;

        void sendPacket(std::string data) override;
    };

    std::pair<std::unique_ptr<Bridge>, std::unique_ptr<Bridge>> newLocalBridgePair();
}

#endif //RIPOSTE_BRIDGE_H
