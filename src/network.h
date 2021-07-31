//
// Created by Caelum van Ispelen on 7/1/21.
//

#ifndef RIPOSTE_NETWORK_H
#define RIPOSTE_NETWORK_H

#include <memory>
#include <optional>

namespace rip {
    // A TCP connection to a server.
    //
    // Implements a simple length-prefixing codec,
    // where each message is prefixed with a 4-byte, big-endian length field.
    class NetworkConnection {
        class impl;
        std::unique_ptr<impl> _impl;

        void setError(std::string message);

        std::optional<uint32_t> decodeLengthField();
        std::optional<uint32_t> hasEnoughData();

    public:
        NetworkConnection(const std::string &address, uint16_t port);
        ~NetworkConnection();

        NetworkConnection(const NetworkConnection &other) = delete;
        NetworkConnection(NetworkConnection &&other) noexcept;

        // Sends a message. _Blocks_.
        //
        // You should call getError() after calling this method
        // to determine whether an error occurred.
        void sendMessage(const std::string &data);

        // Waits for a message to be received and returns it. _Blocks_.
        //
        // You should call getError() after calling this method
        // to determine whether an error occurred. If so,
        // the returned message should be ignored.
        std::string recvMessage();

        std::optional<std::string> getError();
    };
}

#endif //RIPOSTE_NETWORK_H
