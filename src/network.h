//
// Created by Caelum van Ispelen on 7/1/21.
//
// Revised on 08/23/21.
//

#ifndef RIPOSTE_NETWORK_H
#define RIPOSTE_NETWORK_H

#include <riposte_networking.h>

namespace rip {
    class ConnectionHandle;

    typedef std::function<void(const RipResult&)> FnCallback;

    // C++ wrapper over riposte-c-bindings networking support.
    class NetworkingContext {
        RipNetworkingContext *inner;

    public:
        NetworkingContext();
        ~NetworkingContext();

        ConnectionHandle connectStdio();

        void waitAndInvokeCallbacks();

        RipNetworkingContext *getInner();
    };

    // C++ wrapper over a RipConnectionHandle.
    class ConnectionHandle {
        RipConnectionHandle *inner;
        RipNetworkingContext *ctx;

    public:
        ConnectionHandle(RipConnectionHandle *inner, RipNetworkingContext *ctx);
        ~ConnectionHandle();

        ConnectionHandle(ConnectionHandle &&other);
        ConnectionHandle(const ConnectionHandle &other) = delete;

        ConnectionHandle &operator=(ConnectionHandle &&other);

        void sendMessage(const std::string &data, FnCallback &callback);
        void recvMessage(FnCallback &callback);
    };
}

#endif //RIPOSTE_NETWORK_H
