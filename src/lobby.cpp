//
// Created by Caelum van Ispelen on 8/23/21.
//

#include "lobby.h"

#define SEND(packet, name) { \
    rip::proto::ServerLobbyPacket p; \
    p.mutable_##name()->CopyFrom(packet); \
    sendMessage(p);\
}

namespace rip {
    void LobbyConnection::sendMessage(const proto::ServerLobbyPacket &packet) {
        std::string bytes;
        packet.AppendToString(&bytes);
        FnCallback callback = [](const RipResult &res) {};
        handle.sendMessage(bytes, callback);
    }

    void LobbyConnection::requestMoreData() {
        FnCallback callback = [&](const RipResult &res) {
            if (rip_result_is_success(&res)) {
                RipBytes bytes = rip_result_get_bytes(&res);
                this->handleReceivedData(bytes.ptr, bytes.len);
                this->requestMoreData();
            }
        };
        handle.recvMessage(callback);
    }

    void LobbyConnection::handleCreateSlot(const proto::CreateSlot &packet) {
        if (!isAdmin) return;

        proto::LobbySlot slot;
        slot.set_isai(packet.isai());
        slot.set_occupied(false);
        server->addSlot(std::move(slot));
    }

    void LobbyConnection::handleDeleteSlot(const proto::DeleteSlot &packet) {
        if (!isAdmin) return;

        server->removeSlot(packet.slotid());
    }

    void LobbyConnection::handleRequestGameStart(const proto::RequestGameStart &packet) {
        if (!isAdmin) return;
    }

    void LobbyConnection::handleMessage(const proto::ClientLobbyPacket &packet) {
        if (packet.has_createslot()) {
            handleCreateSlot(packet.createslot());
        } else if (packet.has_deleteslot()) {
            handleDeleteSlot(packet.deleteslot());
        } else if (packet.has_requestgamestart()) {
            handleRequestGameStart(packet.requestgamestart());
        }
    }

    LobbyConnection::LobbyConnection(ConnectionHandle handle, proto::UUID userID, bool isAdmin, LobbyServer *server)
        : handle(std::move(handle)), userID(userID), isAdmin(isAdmin), server(server) {
        requestMoreData();
    }

    void LobbyConnection::setSlotID(uint32_t slotID) {
        this->slotID = slotID;
    }

    void LobbyConnection::setID(LobbyConnectionID id) {
        this->id = id;
    }

    uint32_t LobbyConnection::getSlotID() const {
        return slotID;
    }

    LobbyConnectionID LobbyConnection::getID() const {
        return id;
    }

    void LobbyConnection::handleReceivedData(const unsigned char *data, size_t len) {
        proto::ClientLobbyPacket packet;
        packet.ParseFromArray(data, (int) len);
        handleMessage(packet);
    }

    void LobbyConnection::sendLobbyInfo(const std::vector<proto::LobbySlot> &slots, bool isStatic) {
        proto::LobbyInfo packet;
        packet.set_isstatic(isStatic);
        packet.set_yourslotid(slotID);
        SEND(packet, lobbyinfo);
    }

    void LobbyConnection::disconnect() {
        proto::Kicked packet;
        SEND(packet, kicked);
    }

    void LobbyConnection::onGameStarted() {
        proto::GameStarted packet;
        SEND(packet, gamestarted);
    }

    LobbyServer::LobbyServer(std::shared_ptr<NetworkingContext> networkCtx) : networkCtx(networkCtx) {

    }

    LobbyConnectionID LobbyServer::addConnection(ConnectionHandle handle, proto::UUID userID, bool isAdmin) {
        // Attempt to find a suitable slot.
        std::optional<uint32_t> slotID = {};
        // First pass: check for a slot with the same UUID
        // (higher priority)
        for (const auto &slot : slots) {
            if (slot.has_owneruuid() && slot.owneruuid().uuid() == userID.uuid()) {
                slotID = slot.id();
                break;
            }
        }

        // Second pass: find any slot.
        if (!slotID.has_value()) {
            for (const auto &slot : slots) {
                if (slot.isai()) continue;
                if (slot.occupied()) continue;
                if (slot.has_owneruuid() && slot.owneruuid().uuid() != userID.uuid()) continue;

                slotID = slot.id();
                break;
            }
        }

        if (!slotID.has_value()) return {};

        getSlot(*slotID)->set_occupied(true);
        getSlot(*slotID)->mutable_owneruuid()->CopyFrom(userID);
        getSlot(*slotID)->set_isadmin(isAdmin);

        const auto id = connections.insert(LobbyConnection(std::move(handle), std::move(userID), isAdmin, this));
        connections[id].setID(id);
        connections[id].setSlotID(*slotID);

        broadcastLobbyInfo();

        return id;
    }

    void LobbyServer::removeConnection(LobbyConnectionID id) {
        if (!connections.contains(id)) return;
        connections[id].disconnect();

        auto slot = getSlot(connections[id].getSlotID());
        if (slot) {
            slot->set_occupied(false);
        }

        connections.erase(id);

        broadcastLobbyInfo();
    }

    uint32_t LobbyServer::addSlot(proto::LobbySlot slot) {
        slot.set_id(nextSlotID++);

        slots.emplace_back(std::move(slot));

        broadcastLobbyInfo();

        return nextSlotID - 1;
    }

    void LobbyServer::removeSlot(uint32_t id) {
        for (int i = 0; i < slots.size(); i++) {
            if (slots[i].id() == id) {
                slots.erase(slots.begin() + i);
                broadcastLobbyInfo();
                return;
            }
        }
    }

    proto::LobbySlot *LobbyServer::getSlot(uint32_t id) {
        for (auto &slot : slots) {
            if (slot.id() == id) return &slot;
        }
        return nullptr;
    }

    void LobbyServer::setStatic(bool isStatic) {
        this->isStatic = isStatic;
    }

    void LobbyServer::broadcastLobbyInfo() {
        for (auto &conn : connections) {
            conn.sendLobbyInfo(slots, isStatic);
        }
    }

    void LobbyServer::run() {
        while (true) {
            networkCtx->waitAndInvokeCallbacks();
        }
    }
}
