//
// Created by Caelum van Ispelen on 8/23/21.
//

#ifndef RIPOSTE_LOBBY_H
#define RIPOSTE_LOBBY_H

#include <memory>
#include <riposte.pb.h>
#include "network.h"
#include "slot_map.h"
#include "registry.h"
#include "rng.h"

namespace rip {
    typedef ID LobbyConnectionID;

    class LobbyServer;

    class LobbyConnection {
        ConnectionHandle handle;
        uint32_t slotID = 0;
        LobbyConnectionID id;
        LobbyServer *server;
        proto::UUID userID;
        bool isAdmin;

        void handleCreateSlot(const proto::CreateSlot &packet);
        void handleDeleteSlot(const proto::DeleteSlot &packet);
        void handleRequestGameStart(const proto::RequestGameStart &packet);

        void sendMessage(const proto::ServerLobbyPacket &packet);
        void handleMessage(const proto::ClientLobbyPacket &packet);

        void requestMoreData();

    public:
        LobbyConnection(ConnectionHandle handle, proto::UUID userID, bool isAdmin, LobbyServer *server);

        void setSlotID(uint32_t slotID);
        void setID(LobbyConnectionID id);

        uint32_t getSlotID() const;
        LobbyConnectionID getID() const;

        void handleReceivedData(const unsigned char *data, size_t len);

        void sendLobbyInfo(const std::vector<proto::LobbySlot> &slots, bool isStatic);
        void disconnect();
        void onGameStarted();
    };

    // The lobby server.
    //
    // Keeps track of player slots in the lobby
    // and does connection IO for the lobby state.
    //
    // Switches into the Game state by creating a Server
    // once the lobby state is ended (because the host sent RequestStartGame).
    class LobbyServer {
        std::shared_ptr<NetworkingContext> networkCtx;
        slot_map<std::shared_ptr<LobbyConnection>> connections;
        std::vector<proto::LobbySlot> slots;
        uint32_t nextSlotID = 0;
        bool isStatic = false;
        Rng rng;
        std::shared_ptr<Registry> registry;

    public:
        LobbyServer(std::shared_ptr<NetworkingContext> networkCtx, std::shared_ptr<Registry> registry);

        // Adds a new connection.
        //
        // Will attempt to find a slot for the new player.
        // If there is no available slot, the connection is dropped.
        LobbyConnectionID addConnection(ConnectionHandle handle, proto::UUID userID, bool isAdmin);
        // Removes a connection and its associated slot.
        void removeConnection(LobbyConnectionID id);

        // Adds a new slot.
        uint32_t addSlot(proto::LobbySlot slot);
        // Removes the slot with the given ID, if it exists.
        void removeSlot(uint32_t id);
        // May return null if the ID is invalid.
        proto::LobbySlot *getSlot(uint32_t id);

        // Puts the lobby in static mode, preventing
        // new slots from being created.
        //
        // This is used when the game is being loaded from an existing save file.
        void setStatic(bool isStatic);

        // Broadcasts LobbyInfo to all connections.
        void broadcastLobbyInfo();

        void run();
    };
}

#endif //RIPOSTE_LOBBY_H
