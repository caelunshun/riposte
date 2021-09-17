//
// Created by Caelum van Ispelen on 7/1/21.
//
// Revised on 08/23/21.
//

#include "network.h"

#include <arpa/inet.h>
#include <netinet/in.h>
#include <sys/socket.h>
#include <unistd.h>
#include <iostream>

namespace rip {
    NetworkingContext::NetworkingContext() {
        inner = networkctx_create();
    }

    NetworkingContext::~NetworkingContext() {
        networkctx_free(inner);
    }

    ConnectionHandle NetworkingContext::connectStdio() {
        auto *handle = networkctx_connect_stdio(inner);
        return ConnectionHandle(handle, inner);
    }

    HubServerConnection NetworkingContext::connectToHub(const std::string &authToken) {
        auto *handle = networkctx_create_game(inner, (const uint8_t*) authToken.data(), authToken.size());
        return HubServerConnection(handle, inner);
    }

    void NetworkingContext::waitAndInvokeCallbacks() {
        networkctx_wait(inner);
    }

    RipNetworkingContext *NetworkingContext::getInner() {
        return inner;
    }
   
    void *callbackToUserdata(FnCallback callback) {
        auto *c = new FnCallback();
        *c = std::move(callback);
        
        return (void*) c;
    }
    
    void callbackFunction(void *userdata, const RipResult *result) {
        FnCallback &callback = *((FnCallback*) userdata);
        callback(*result);
        delete &callback;
    }

    ConnectionHandle::ConnectionHandle(RipConnectionHandle *inner, RipNetworkingContext *ctx) : inner(inner), ctx(ctx) {}

    ConnectionHandle::~ConnectionHandle() {
        if (inner) {
            networkctx_conn_free(ctx, inner);
        }
    }

    void ConnectionHandle::sendMessage(const std::string &data, FnCallback &callback) {
        networkctx_conn_send_data(ctx, inner, RipBytes {
            .len = data.size(),
            .ptr = (const unsigned char*) data.data(),
        }, callbackFunction, callbackToUserdata(std::move(callback)));
    }

    void ConnectionHandle::recvMessage(FnCallback &callback) {
        networkctx_conn_recv_data(ctx, inner, callbackFunction, callbackToUserdata(std::move(callback)));
    }

    ConnectionHandle::ConnectionHandle(ConnectionHandle &&other) {
        ctx = other.ctx;
        inner = other.inner;
        other.inner = nullptr;
    }

    ConnectionHandle &ConnectionHandle::operator=(ConnectionHandle &&other) {
        ctx = other.ctx;
        inner = other.inner;
        other.inner = nullptr;
        return *this;
    }

    HubServerConnection::HubServerConnection(RipHubServerConnection *inner, RipNetworkingContext *ctx) : inner(inner), ctx(ctx) {}

    void HubServerConnection::getNewConnection(FnCallback &callback) {
        hubconn_get_new_connection(ctx, inner, callbackFunction, callbackToUserdata(std::move(callback)));
    }
}
