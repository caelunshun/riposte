//
// Created by Caelum van Ispelen on 7/1/21.
//

#include "network.h"

#include <arpa/inet.h>
#include <netinet/in.h>
#include <sys/socket.h>
#include <unistd.h>
#include <iostream>

namespace rip {
    struct NetworkConnection::impl {
        std::optional<std::string> error;
        int sock;
        std::string receiveBuffer;
        std::string outputBuffer;
    };

    void NetworkConnection::setError(std::string message) {
        _impl->error = std::move(message);
    }

    NetworkConnection::NetworkConnection(const std::string &address, uint16_t port) : _impl(std::make_unique<impl>()) {
        struct sockaddr_in addr = {0};
        addr.sin_port = htons(port);
        addr.sin_family = AF_INET;
        addr.sin_addr.s_addr = inet_addr(address.c_str());

        int sock = socket(AF_INET, SOCK_STREAM, IPPROTO_TCP);
        if (sock == -1) {
            setError("failed to create socket");
            return;
        }

        if (connect(sock, (sockaddr*) &addr, sizeof(addr)) == -1) {
            setError("failed to connect to " + address + ":" + std::to_string(port));
            return;
        }

        _impl->sock = sock;
    }

    void NetworkConnection::sendMessage(const std::string &data) {
        auto &output = _impl->outputBuffer;

        // Encode length field
        output.push_back((uint8_t) data.size() >> 24);
        output.push_back((uint8_t) (data.size() >> 16));
        output.push_back((uint8_t) (data.size() >> 8));
        output.push_back((uint8_t) (data.size()));

        // Encode data
        output.append(data);

        // Write to socket
        if (!send(_impl->sock, (void*) output.data(), output.size(), 0)) {
            setError("failed to send data");
            return;
        }

        output.clear();
    }

    std::optional<uint32_t> NetworkConnection::decodeLengthField() {
        if (_impl->receiveBuffer.size() < 4) return {};

        auto *data = (unsigned char*) _impl->receiveBuffer.data();

        return (((int) data[0]) << 24) | (((int) data[1]) << 16)
            | (((int) data[2]) << 8) | ((int) data[3]);
    }

    std::optional<uint32_t> NetworkConnection::hasEnoughData() {
        const auto length = decodeLengthField();
        if (!length.has_value()) return {};

        if (_impl->receiveBuffer.size() - 4 < length) return {};

        return length;
    }

    std::string NetworkConnection::recvMessage() {
        // Wait until the four-byte length field and the remainder of the packet have been received.
        auto dataLength = hasEnoughData();
        while (!dataLength.has_value()) {
            int pos = _impl->receiveBuffer.size();
            _impl->receiveBuffer.resize(_impl->receiveBuffer.size() + 1024);

            int receivedBytes = recv(
                    _impl->sock,
                    (void*) (_impl->receiveBuffer.data() + pos),
                    1024,
                    0
                    );
            if (receivedBytes < 0) {
                setError("failed to receive data (disconnected from server)");
                std::cout << "ERROR" << strerror(errno) << std::endl;
                return "";
            }

            _impl->receiveBuffer.erase(pos + receivedBytes, std::string::npos);

            dataLength = hasEnoughData();
        }

        auto result = _impl->receiveBuffer.substr(4, *dataLength);
        _impl->receiveBuffer.erase(0, *dataLength + 4);
        return result;
    }

    std::optional<std::string> NetworkConnection::getError() {
        return _impl->error;
    }

    NetworkConnection::~NetworkConnection() {
        if (_impl->sock != 0) {
            std::cout << "closing sock " << _impl->sock;
            close(_impl->sock);
        }
    }

    NetworkConnection::NetworkConnection(NetworkConnection &&other)  noexcept {
        _impl = std::make_unique<impl>();
        _impl->receiveBuffer = std::move(other._impl->receiveBuffer);
        _impl->sock = other._impl->sock;
        _impl->error = std::move(other._impl->error);
        other._impl->sock = 0;
    }
}
