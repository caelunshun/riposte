//
// Created by Caelum van Ispelen on 7/30/21.
//

// Asynchronous networking for Lua.
// Sockets are spawned on a separate thread
// and communicate with the main thread through a pair of queues.

#include <slot_map.h>
#include <network.h>

#include <thread>
#include <utility>
#include "lua_networking.h"
#include <readerwriterqueue/readerwriterqueue.h>
#include <bridge.h>

using namespace moodycamel;

namespace rip {
    typedef uint32_t CompletionID;

    enum MessageType {
        Connected,
        WaitForMessage,
        ReceivedMessage,
        SendMessage,
        Errored,
        ConvertToBridge,
        ConvertedToBridge,
    };

    struct Message {
        MessageType type;
        CompletionID completionID;
        std::string message;
        std::unique_ptr<Bridge> bridge;

        Message() {}

        Message(MessageType type, CompletionID completionId, std::string message) : type(type),
                                                                                           completionID(completionId),
                                                                                           message(std::move(message)) {}
    };

    struct ConnectionHandle {
        std::shared_ptr<ReaderWriterQueue<Message>> threadToMain;
        std::shared_ptr<BlockingReaderWriterQueue<Message>> mainToThread;
        CompletionID connectCompletion;
        ID id;

        ConnectionHandle(std::shared_ptr<ReaderWriterQueue<Message>> threadToMain,
                         std::shared_ptr<BlockingReaderWriterQueue<Message>> mainToThread) : threadToMain(std::move(threadToMain)),
                                                                                            mainToThread(std::move(
                                                                                                    mainToThread)) {}
    };

    std::array<ConnectionHandle, 2> createConnHandlePair() {
        auto threadToMain = std::make_shared<ReaderWriterQueue<Message>>();
        auto mainToThread = std::make_shared<BlockingReaderWriterQueue<Message>>();

        ConnectionHandle conn(threadToMain, mainToThread);

        return {
            conn,
            conn
        };
    }

    // Keeps track of connections and queues used to communicate
    // with connection threads.
    class AsyncNetworkingBridge {
        slot_map<ConnectionHandle> connectionHandles;
        CompletionID nextCompletionID = 0;

    public:
        CompletionID getNextCompletionID() {
            return nextCompletionID++;
        }

        CompletionID connectAsync(const std::string &address, uint16_t port) {
            auto connHandles = createConnHandlePair();

            auto threadHandle = connHandles[0];
            auto mainHandle = connHandles[1];

            const auto completionID = getNextCompletionID();

            // Launch the networking thread.
            std::thread connectionThread([=] () {
                 NetworkConnection conn(address, port);
                 if (conn.getError()) {
                     threadHandle.threadToMain->emplace(MessageType::Errored, completionID, *conn.getError());
                     return;
                 }

                 threadHandle.threadToMain->emplace(MessageType::Connected, completionID, "");

                 while(true) {
                     Message message;
                     threadHandle.mainToThread->wait_dequeue(message);

                     std::string data;
                     switch (message.type) {
                         case MessageType::SendMessage:
                             conn.sendMessage(message.message);
                             break;
                         case MessageType::WaitForMessage:
                             data = conn.recvMessage();
                             if (conn.getError()) {
                                 threadHandle.threadToMain->emplace(MessageType::Errored, message.completionID, *conn.getError());
                                 return;
                             }

                             threadHandle.threadToMain->emplace(MessageType::ReceivedMessage, message.completionID, std::move(data));
                             break;
                         case MessageType::ConvertToBridge:
                             auto bridge = std::make_unique<NetworkBridge>(std::move(conn));

                             Message response(MessageType::ConvertedToBridge, message.completionID, "");
                             response.bridge = std::move(bridge);

                             threadHandle.threadToMain->emplace(std::move(response));

                             return;
                     }
                 }
            });
            connectionThread.detach();

            mainHandle.connectCompletion = completionID;
            auto id = connectionHandles.insert(std::move(mainHandle));
            connectionHandles[id].id = id;

            return completionID;
        }

        CompletionID sendMessageAsync(ID connHandleID, std::string payload) {
            const auto completionID = getNextCompletionID();
            connectionHandles[connHandleID].mainToThread->emplace(MessageType::SendMessage, completionID, std::move(payload));
            return completionID;
        }

        CompletionID recvMessageAsync(ID connHandleID) {
            const auto completionID = getNextCompletionID();
            connectionHandles[connHandleID].mainToThread->emplace(MessageType::WaitForMessage, completionID, "");
            return completionID;
        }

        CompletionID makeBridgeFromConn(ID connHandleID) {
            const auto completionID = getNextCompletionID();
            connectionHandles[connHandleID].mainToThread->emplace(MessageType::ConvertToBridge, completionID, "");
            return completionID;
        }

        // generates a table of completion ID to {payload, error}
        sol::table pollCompletions(sol::state &lua) {
            auto result = lua.create_table();

            for (auto &connHandle : connectionHandles) {
                Message msg;
                while (connHandle.threadToMain->try_dequeue(msg)) {
                    sol::table payload;
                    switch (msg.type) {
                        case MessageType::ReceivedMessage:
                            payload = lua.create_table_with("contents", msg.message);
                            break;
                        case MessageType::Errored:
                            payload = lua.create_table_with("error", msg.message);
                            break;
                        case MessageType::Connected:
                            payload = lua.create_table_with("contents", connHandle.id);
                            break;
                        case MessageType::ConvertedToBridge:
                            payload = lua.create_table_with("contents", std::move(msg.bridge));
                            break;
                    }
                    result[msg.completionID] = payload;
                }
            }

            return result;
        }
    };

    void registerNetworkBindings(std::shared_ptr<sol::state> &lua) {
        auto bridge = std::make_shared<AsyncNetworkingBridge>();

        (*lua)["networkingConnectAsync"] = [=](std::string ip, uint16_t port) {
            return bridge->connectAsync(ip, port);
        };
        (*lua)["networkingSendAsync"] = [=](ID connHandle, std::string payload) {
            return bridge->sendMessageAsync(connHandle, std::move(payload));
        };
        (*lua)["networkingRecvAsync"] = [=](ID connHandle) {
            return bridge->recvMessageAsync(connHandle);
        };
        (*lua)["networkingConvertToBridge"] = [=] (ID connHandle) {
            return bridge->makeBridgeFromConn(connHandle);
        };
        (*lua)["networkingPoll"] = [=] () {
            return bridge->pollCompletions(*lua);
        };
    }
}
