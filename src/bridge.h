//
// Created by Caelum van Ispelen on 6/20/21.
//

#ifndef RIPOSTE_BRIDGE_H
#define RIPOSTE_BRIDGE_H

#include <string>
#include <optional>
#include <memory>

#include <readerwriterqueue/readerwriterqueue.h>

namespace rip {
    // A bridge between client and server.
    //
    // Backed either by a network connection or
    // a queue connected to another thread.
    class Bridge {
    public:
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
                    std::shared_ptr<moodycamel::ReaderWriterQueue<std::string>> receiveQueue) : sendQueue(sendQueue), receiveQueue(receiveQueue) {}

        ~LocalBridge() override = default;

        std::optional<std::string> pollReceivedPacket() override;

        void sendPacket(std::string data) override;
    };

    std::pair<LocalBridge, LocalBridge> newLocalBridgePair() {
        auto queueA = std::make_shared<moodycamel::ReaderWriterQueue<std::string>>();
        auto queueB = std::make_shared<moodycamel::ReaderWriterQueue<std::string>>();
        return {
            LocalBridge(queueA, queueB),
            LocalBridge(queueB, queueA),
        };
    }
}

#endif //RIPOSTE_BRIDGE_H
